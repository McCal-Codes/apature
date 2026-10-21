# Apature Pocket, Phase A: USB receiver proof of concept

**Experimental / not field-ready.** This folder is intentionally independent of the root Rust workspace: no Apature application files or APIs are changed. Python 3.10+ standard library and the `gphoto2` executable (for camera mode) are its only runtime prerequisites. The production receiver should eventually be a Rust/Linux service using Apature's verified-ingest code.

## What exists today

- Simulated camera intake with hashes, append-only manifests, no overwrites, and the original downloaded files preserved.
- A `gphoto2 --capture-tethered` supervisor, with a generated post-download hook that enqueues each fully downloaded file. One camera connection at a time.
- A storage archive that preserves Nikon RAW/JPEG filenames and stores each pair in the same session directory.
- Conservative failure behavior: partial copies remain named `.partial` only until cleaned up; a previously committed file with the wrong hash produces a hard error; collisions with different bytes are refused.
- Restart processing of queued events. Successful event paths are cached in RAM to avoid repeated multi-gigabyte hashing while running; an event-log replay after reboot may need to reverify existing archived files.
- Nine isolated tests using dummy files; none prove Z6 II compatibility or memory-card retention.

## Try without a camera (safe)

```bash
mkdir -p /tmp/apature-demo/camera /tmp/apature-demo/data
printf 'simulated data' > /tmp/apature-demo/camera/DSC_0001.NEF
python3 experiments/apature-pocket/pocket.py simulate \
  --source /tmp/apature-demo/camera \
  --root /tmp/apature-demo/data \
  --session test_20261020 --camera Nikon_Z6II
python3 -m unittest discover -s experiments/apature-pocket -p 'test_*.py' -v
```

Output files: `data/Sessions/test_20261020/Nikon_Z6II/` and an append-only JSONL record in `data/Manifests/`. The digest verifies **spool versus archive**, not camera memory card versus archive. Do not claim end-to-end camera original verification.

## Nikon Z6 II lab test (do not run during an assignment)

Install a recent gphoto2 and libgphoto2, connect the camera via a proper USB data cable, use test images, and close Nikon transfer applications / macOS Photos that could hold the PTP device. First, run *read-only* tests:

```bash
gphoto2 --auto-detect
gphoto2 --summary
gphoto2 --wait-event=15s
```

Confirm that the camera's **physical shutter** still works and pictures stay on its card while connected. Do not begin automatic downloads until that is verified. For the download experiment, on a Raspberry Pi / Linux computer with an ext4 data disk mounted at `/mnt/apature-data`, run:

```bash
python3 experiments/apature-pocket/pocket.py camera \
  --root /mnt/apature-data \
  --session lab_20261020 --camera Nikon_Z6II \
  --ack-untested-camera-card-behavior
```

The `--ack` flag indicates that the camera-card behavior is **unverified**; it is not a safety bypass for paid work. Verify directly on the camera after a few shutter presses that all originals remain on its memory card. Exit with Ctrl+C (the tool asks gphoto2 to end the tether session gracefully).

Notes:

- **No Wi-Fi, Bluetooth, or internet** is used during image reception. Camera to Linux host is USB. The Pocket computer must remain powered.
- gphoto2's `--keep` semantics are documented for some capture modes; they are *not* proof that Z6 II physical-shutter tethering keeps originals on-card. This is a mandatory hardware acceptance test.
- This prototype needs an **ext4 or other hardlink-capable destination filesystem**: it uses `os.link` to atomically publish a verified file without overwriting. exFAT may not support it. Do not silently substitute an unsafe rename.
- The prototype does not provide USB device passthrough, camera configuration changes, network sync, battery status, LEDs, auto-start services, queue compaction, or a GUI.
- No automatic gphoto2 reconnect is attempted after failure. Stop, inspect logs and card content, then restart manually.
- Manifests and event journals are append-only; if a power loss tears the last JSONL line, new writes fail rather than silently creating a corrupt journal. Back up the damaged file and repair that partial last line before restarting. A production revision should replace the journal with SQLite or an atomic transactional format.
- This prototype verifies *downloaded files against their copies*. A broken download that is already incomplete before `ACTION=download` requires hardware/source validation to detect.
- Do not automatically delete, format, unmount, or erase camera storage.

## Next engineering steps

1. Verify Nikon Z6 II events, memory-card retention, RAW+JPEG behavior, and uninterrupted physical shutter operation using a reproducible test matrix.
2. Replace the Python experimental service with an isolated Rust `apature-camera` backend and `apature-pocket` receiver once the transport behavior is proven.
3. Integrate the resulting session format with the desktop Apature ingest UX; validate a real Pi, SSD power budget, USB disconnects, and power-loss recovery.
4. Add systemd, status LEDs, safe shutdown, and hardware compatibility profiles only after successful camera tests.