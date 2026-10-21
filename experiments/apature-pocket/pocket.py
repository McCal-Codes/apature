#!/usr/bin/env python3
"""Apature Pocket Phase A prototype, standalone and stdlib-only.

This is NOT a camera-original verification or a field-ready backup system.
The camera controls remain unverified for Nikon Z6 II until hardware testing.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path

ALLOWED = {'.nef', '.nrw', '.jpg', '.jpeg', '.tif', '.tiff', '.xmp', '.wav'}
SAFE_NAME = re.compile(r'^[a-zA-Z0-9_-]{1,64}$')
BUFFER = 1024 * 1024


def utcnow() -> str:
    return datetime.now(timezone.utc).isoformat()


def require_name(name: str) -> str:
    if not SAFE_NAME.fullmatch(name):
        raise ValueError('session and camera IDs must be 1..64 ASCII letters, digits, _ or -')
    return name


def hash_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as source:
        while chunk := source.read(BUFFER):
            digest.update(chunk)
    return digest.hexdigest()


def sync_dir(path: Path) -> None:
    # Directory fsync works on ext4; any failure is visible to the caller.
    fd = os.open(path, os.O_RDONLY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def append_jsonl(path: Path, record: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = (json.dumps(record, ensure_ascii=True, sort_keys=True) + '\n').encode('utf-8')
    fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        end = os.lseek(fd, 0, os.SEEK_END)
        if end:
            os.lseek(fd, -1, os.SEEK_END)
            if os.read(fd, 1) != b'\n':
                raise RuntimeError(f'incomplete journal tail must be repaired: {path}')
        os.write(fd, payload)
        os.fsync(fd)
    finally:
        os.close(fd)


def records(path: Path):
    if not path.exists():
        return
    with path.open('rb') as source:
        for line in source:
            if not line.endswith(b'\n'):
                break  # Crash during append: incomplete tail ignored for recovery.
            try:
                yield json.loads(line)
            except (ValueError, UnicodeDecodeError):
                # Malformed full line is not safely recoverable. Do not skip it.
                raise RuntimeError(f'corrupt manifest or queue: {path}')


def verified_copy(source: Path, target: Path) -> tuple[int, str]:
    """Compare the *downloaded spool file* to our copy; not to camera media.

    Atomic hard-link commit deliberately requires a hardlink-capable data FS.
    It NEVER overwrites a destination; do not silently fall back to rename.
    """
    target.parent.mkdir(parents=True, exist_ok=True)
    tmp = target.with_name(f'.{target.name}.{uuid.uuid4().hex}.partial')
    digest = hashlib.sha256()
    total = 0
    try:
        with source.open('rb') as read, tmp.open('xb') as out:
            while chunk := read.read(BUFFER):
                out.write(chunk)
                digest.update(chunk)
                total += len(chunk)
            out.flush()
            os.fsync(out.fileno())
        expected = digest.hexdigest()
        if hash_file(tmp) != expected or hash_file(source) != expected:
            raise OSError('source changed while copying or destination hash mismatch')
        os.link(tmp, target)  # Atomic, refuses any existing target, unlike os.replace.
        sync_dir(target.parent)
        return total, expected
    finally:
        tmp.unlink(missing_ok=True)


class Receiver:
    def __init__(self, root: Path, session: str, camera: str, spool: Path):
        self.root = root.expanduser().resolve()
        self.session = require_name(session)
        self.camera = require_name(camera)
        self.spool = spool.expanduser().resolve()
        self.archive = self.root / 'Sessions' / session / camera
        self.manifest = self.root / 'Manifests' / f'{session}-{camera}.jsonl'
        self.archive.mkdir(parents=True, exist_ok=True)
        self.spool.mkdir(parents=True, exist_ok=True)
        if self.root == self.spool or self.root.is_relative_to(self.spool) or self.spool.is_relative_to(self.archive):
            raise ValueError('spool and archive must not overlap')
        self.completed = {row['input']: row for row in records(self.manifest) if row.get('status') == 'copied'}

    def ingest_one(self, path: Path) -> str:
        path = path.expanduser().resolve(strict=True)
        if not path.is_relative_to(self.spool) or not path.is_file():
            raise ValueError('input must be a regular file inside the spool')
        if path.suffix.lower() not in ALLOWED:
            raise ValueError(f'unsupported extension: {path.suffix}')
        rel = path.relative_to(self.spool).as_posix()
        digest = hash_file(path)
        prior = self.completed.get(rel)
        if prior:
            target = self.root / prior['archive']
            if prior['sha256'] == digest and target.is_file() and hash_file(target) == digest:
                return 'already-verified'
            if prior['sha256'] != digest:
                raise RuntimeError(f'source path changed after ingest: {rel}')
            # Previously recorded destination is missing/corrupt: never silently certify it.
            raise RuntimeError(f'previously committed archive is missing or corrupt: {target}')
        dest = self.archive / path.name
        if dest.exists():
            if dest.is_file() and hash_file(dest) == digest:
                status = 'recovered-identical'
            else:
                raise RuntimeError(f'archive name collision with different bytes: {dest}')
        else:
            _, written_hash = verified_copy(path, dest)
            if written_hash != digest:
                raise RuntimeError('unexpected copy digest disagreement')
            status = 'copied'
        entry = {
            'status': 'copied', 'input': rel,
            'archive': dest.relative_to(self.root).as_posix(),
            'bytes': path.stat().st_size, 'sha256': digest,
            'time_utc': utcnow(), 'recovery': status == 'recovered-identical',
            'source_verified': 'spool-only; camera originals not verified',
        }
        append_jsonl(self.manifest, entry)
        self.completed[rel] = entry
        return status

    def ingest_folder(self) -> dict:
        summary = {'copied': 0, 'already-verified': 0, 'failed': 0}
        for path in sorted(self.spool.rglob('*')):
            if not path.is_file() or path.suffix.lower() not in ALLOWED:
                continue
            try:
                state = self.ingest_one(path)
                if state == 'recovered-identical':
                    state = 'already-verified'
                summary[state] += 1
                print(f'{state}: {path.name}', flush=True)
            except (OSError, ValueError, RuntimeError) as exc:
                summary['failed'] += 1
                print(f'FAILED {path}: {exc}', file=sys.stderr, flush=True)
        return summary


def hook_event(queue: Path, argument: str) -> None:
    # Called only after gphoto2 announces ACTION=download (file saved locally).
    # Handle awkward/newline filenames via JSON, not unescaped newline-delimited paths.
    append_jsonl(queue, {'path': str(Path(argument).absolute()), 'time_utc': utcnow()})


def consume_events(queue: Path, receiver: Receiver, seen: set[str] | None = None) -> int:
    """Replay full queue on each tick; manifest makes retries idempotent.

    Errors remain retryable; bad events are printed and the process continues.
    """
    count = 0
    for item in records(queue):
        try:
            path = Path(item['path'])
            if seen is not None and str(path) in seen:
                continue
            status = receiver.ingest_one(path)
            if seen is not None:
                seen.add(str(path))
            if status != 'already-verified':
                print(f'{status}: {path.name}', flush=True)
                count += 1
        except (KeyError, TypeError, ValueError, RuntimeError, OSError) as exc:
            print(f'PENDING/FAILED event: {exc}', file=sys.stderr, flush=True)
    return count


def camera_loop(root: Path, session: str, camera: str, wait: float, acknowledge: bool):
    if not acknowledge:
        raise ValueError('camera mode requires --ack-untested-camera-card-behavior')
    if shutil.which('gphoto2') is None:
        raise RuntimeError('gphoto2 executable not found; install gphoto2 and libgphoto2')
    run_id = uuid.uuid4().hex
    spool = root / 'Incoming' / session / camera / run_id
    queue = root / 'Queue' / f'{run_id}.jsonl'
    receiver = Receiver(root, session, camera, spool)
    # Dynamically generate executable hook: GitHub text files need no executable bit.
    hook = spool / 'on-download.sh'
    cmd = f'exec {shlex.quote(sys.executable)} {shlex.quote(str(Path(__file__).resolve()))} hook --queue {shlex.quote(str(queue))} --file "$ARGUMENT"'
    hook.write_text('#!/bin/sh\n[ "$ACTION" = download ] || exit 0\n' + cmd + '\n')
    hook.chmod(0o700)
    print('CAMERA EXPERIMENT: verify card retention and shutter responsiveness on camera.', flush=True)
    print(f'Incoming: {spool}\nArchive: {receiver.archive}', flush=True)
    processed: set[str] = set()
    proc = subprocess.Popen([
        'gphoto2', '--capture-tethered', '--keep',
        '--filename', '%f.%C', '--hook-script', str(hook),
    ], cwd=spool)
    try:
        while proc.poll() is None:
            consume_events(queue, receiver, processed)
            time.sleep(wait)
        consume_events(queue, receiver, processed)
        if proc.returncode:
            raise RuntimeError(f'gphoto2 ended with exit code {proc.returncode}; reconnect is not automated')
    except KeyboardInterrupt:
        if proc.poll() is None:
            proc.send_signal(signal.SIGUSR2)  # gphoto's documented graceful tether stop.
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.terminate()
                proc.wait(timeout=5)
        consume_events(queue, receiver, processed)
        print('Stopped; camera memory card was never accessed for deletion.')


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description='Apature Pocket Phase A experimental receiver')
    sp = p.add_subparsers(dest='command', required=True)
    s = sp.add_parser('simulate', help='verified ingest from a mock camera/download folder')
    s.add_argument('--source', type=Path, required=True)
    s.add_argument('--root', type=Path, required=True)
    s.add_argument('--session', required=True)
    s.add_argument('--camera', default='Nikon_Z6II')
    c = sp.add_parser('camera', help='EXPERIMENTAL gphoto2 tether, test on spare photos only')
    c.add_argument('--root', type=Path, required=True)
    c.add_argument('--session', required=True)
    c.add_argument('--camera', default='Nikon_Z6II')
    c.add_argument('--poll-seconds', type=float, default=0.5)
    c.add_argument('--ack-untested-camera-card-behavior', action='store_true')
    h = sp.add_parser('hook', help=argparse.SUPPRESS)
    h.add_argument('--queue', type=Path, required=True)
    h.add_argument('--file', required=True)
    args = p.parse_args(argv)
    try:
        if args.command == 'hook':
            hook_event(args.queue, args.file)
        elif args.command == 'simulate':
            result = Receiver(args.root, args.session, args.camera, args.source).ingest_folder()
            print(json.dumps(result, sort_keys=True))
            return 1 if result['failed'] else 0
        elif args.command == 'camera':
            if args.poll_seconds < 0.1:
                raise ValueError('poll-seconds must be >= 0.1')
            camera_loop(args.root.expanduser().resolve(), require_name(args.session), require_name(args.camera), args.poll_seconds, args.ack_untested_camera_card_behavior)
    except (OSError, RuntimeError, ValueError) as exc:
        print(f'apature-pocket: {exc}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())

