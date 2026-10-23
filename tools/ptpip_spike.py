#!/usr/bin/env python3
"""Apature Wi-Fi receiver prototype for Nikon Z bodies (PTP/IP).

    python3 -I tools/ptpip_spike.py OUT_DIR [--camera IP] [--name "Apature"]

Finds the camera over Bonjour (_ptp._tcp), reads its mode from the TXT
record, and then:

  pairing mode  (apps=PAIR)  pairs with a fresh identity, which the camera
                             confirms on its own screen, and saves it to
                             ~/.apature-camera-id so later runs reuse it
  transfer mode (apps=WT)    opens a session with the saved identity, waits
                             for the camera to announce frames, and fetches
                             each one into OUT_DIR with its own file name

If Nikon's utility already paired this Mac, its system identity at
/Library/ptpitcp.guid is used when no saved identity exists. Nothing on the
camera is changed or deleted. Ctrl-C closes the session cleanly.
"""
import os
import re
import socket
import struct
import subprocess
import sys
import time
import uuid

PORT = 15740
ID_FILE = os.path.expanduser("~/.apature-camera-id")
PT = {1: "InitCmdReq", 2: "InitCmdAck", 3: "InitEvtReq", 4: "InitEvtAck", 5: "InitFail",
      6: "CmdReq", 7: "CmdResp", 8: "Event", 9: "StartData", 10: "Data", 11: "Cancel", 12: "EndData", 13: "Ping", 14: "Pong"}
OP_GET_DEVICE_INFO, OP_OPEN_SESSION, OP_CLOSE_SESSION = 0x1001, 0x1002, 0x1003
OP_GET_OBJECT_INFO, OP_GET_OBJECT = 0x1008, 0x1009
EV_OBJECT_ADDED = 0x4002
RC_OK = 0x2001


def log(msg):
    print(time.strftime("%H:%M:%S"), msg, flush=True)


# ---- discovery ---------------------------------------------------------------

def run_for(cmd, seconds):
    """Run a dns-sd command that never exits on its own; return its output."""
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    try:
        return p.communicate(timeout=seconds)[0]
    except subprocess.TimeoutExpired:
        p.kill()
        return p.communicate()[0]


def discover(seconds=5):
    """(ip, mode, instance) for the first camera advertising _ptp._tcp, or None."""
    out = run_for(["dns-sd", "-B", "_ptp._tcp", "local"], seconds)
    names = re.findall(r"_ptp\._tcp\.\s+(\S+)\s*$", out, flags=re.M)
    if not names:
        return None
    name = names[0]
    text = run_for(["dns-sd", "-L", name, "_ptp._tcp", "local"], seconds)
    host = re.search(r"can be reached at (\S+?):(\d+)", text)
    mode = re.search(r"apps=(\w+)", text)
    ip = None
    if host:
        # Nikon encodes the address in the host name: 38e86a91-010-064-144-040.local.
        m = re.search(r"-(\d{3})-(\d{3})-(\d{3})-(\d{3})\.local", host.group(1))
        if m:
            ip = ".".join(str(int(x)) for x in m.groups())
        else:
            try:
                ip = socket.gethostbyname(host.group(1).rstrip("."))
            except OSError:
                ip = None
    return ip, (mode.group(1) if mode else "?"), name


# ---- identity --------------------------------------------------------------------

def load_identity():
    for path in (ID_FILE, "/Library/ptpitcp.guid"):
        try:
            b = open(path, "rb").read()[:16]
            if len(b) == 16:
                return b, path
        except OSError:
            continue
    return None, None


def save_identity(guid):
    with open(ID_FILE, "wb") as f:
        f.write(guid)
    os.chmod(ID_FILE, 0o600)


# ---- transport -------------------------------------------------------------------

def pkt(t, payload=b""):
    return struct.pack("<II", 8 + len(payload), t) + payload


def read_pkt(s):
    hdr = b""
    while len(hdr) < 8:
        c = s.recv(8 - len(hdr))
        if not c:
            raise ConnectionError("camera closed the connection")
        hdr += c
    ln, t = struct.unpack("<II", hdr)
    body = b""
    while len(body) < ln - 8:
        c = s.recv(min(1 << 16, ln - 8 - len(body)))
        if not c:
            raise ConnectionError("camera closed mid-packet")
        body += c
    return t, body


class Camera:
    def __init__(self, ip, guid, name):
        self.ip, self.guid, self.name = ip, guid, name
        self.tid = 0
        self.cmd = self.evt = None
        self.camera_name = ""

    def connect(self):
        self.cmd = socket.create_connection((self.ip, PORT), timeout=20)
        body = self.guid + self.name.encode("utf-16-le") + b"\x00\x00" + struct.pack("<I", 0x00010000)
        self.cmd.sendall(pkt(1, body))
        t, b = read_pkt(self.cmd)
        if t != 2:
            reason = struct.unpack("<I", b[:4])[0] if len(b) >= 4 else -1
            raise PermissionError(f"camera refused this identity ({PT.get(t, t)}, reason {reason})")
        conn = struct.unpack("<I", b[:4])[0]
        self.camera_name = b[20:].split(b"\x00\x00")[0].decode("utf-16-le", "ignore")
        self.evt = socket.create_connection((self.ip, PORT), timeout=20)
        self.evt.sendall(pkt(3, struct.pack("<I", conn)))
        t, _ = read_pkt(self.evt)
        if t != 4:
            raise ConnectionError(f"event channel refused ({PT.get(t, t)})")
        self.evt.settimeout(None)

    def op(self, code, *params):
        """Run one operation; returns the data phase bytes (possibly empty)."""
        self.tid += 1
        body = struct.pack("<IHI", 1, code, self.tid) + b"".join(struct.pack("<I", p) for p in params)
        self.cmd.sendall(pkt(6, body))
        data = b""
        while True:
            t, b = read_pkt(self.cmd)
            if t in (10, 12):
                data += b[4:]
            elif t == 7:
                rc = struct.unpack("<H", b[:2])[0]
                if rc != RC_OK:
                    raise RuntimeError(f"operation {code:#06x} answered {rc:#06x}")
                return data
            elif t != 9:
                log(f"unexpected {PT.get(t, t)} on the command channel ({len(b)} bytes)")

    def close(self):
        try:
            self.op(OP_CLOSE_SESSION)
        except Exception:
            pass
        for s in (self.cmd, self.evt):
            try:
                s.close()
            except Exception:
                pass


# ---- PTP structures --------------------------------------------------------------

def ptp_string(b, off):
    n = b[off]
    return b[off + 1: off + 1 + n * 2].decode("utf-16-le", "ignore").rstrip("\x00"), off + 1 + n * 2


def device_info(b):
    off = 8
    _, off = ptp_string(b, off)
    off += 2
    arrays = []
    for _ in range(5):
        n = struct.unpack("<I", b[off:off + 4])[0]
        arrays.append(set(struct.unpack("<%dH" % n, b[off + 4: off + 4 + 2 * n])))
        off += 4 + 2 * n
    manu, off = ptp_string(b, off)
    model, off = ptp_string(b, off)
    ver, _ = ptp_string(b, off)
    return manu, model, ver, arrays[0]


def object_info(b):
    size = struct.unpack("<I", b[8:12])[0]
    name, _ = ptp_string(b, 52)
    return size, name


# ---- the two modes -------------------------------------------------------------------

def pair(ip, name):
    guid = uuid.uuid4().bytes
    cam = Camera(ip, guid, name)
    cam.connect()
    manu, model, ver, _ = device_info(cam.op(OP_GET_DEVICE_INFO))
    log(f"{manu} {model}, firmware {ver}: pairing")
    cam.op(OP_OPEN_SESSION, 1)
    # The camera shows the pairing on its own screen and confirms it there.
    cam.op(0x952b)
    cam.op(0x935a, 0x2001)
    save_identity(guid)
    log(f"paired; identity saved to {ID_FILE}")
    cam.close()


def receive(ip, name, out):
    guid, source = load_identity()
    if guid is None:
        raise SystemExit("no identity yet: put the camera in its connection wizard and run again to pair")
    cam = Camera(ip, guid, name)
    cam.connect()
    manu, model, ver, ops = device_info(cam.op(OP_GET_DEVICE_INFO))
    log(f"{manu} {model}, firmware {ver} (identity from {source})")
    if OP_GET_OBJECT not in ops:
        raise SystemExit("the camera is not in transfer mode; finish the wizard on the camera and run again")
    cam.op(OP_OPEN_SESSION, 1)
    log(f"ready; frames go to {out}")
    try:
        while True:
            t, b = read_pkt(cam.evt)
            if t == 13:
                cam.evt.sendall(pkt(14))
                continue
            if t != 8:
                continue
            code = struct.unpack("<H", b[:2])[0]
            params = [struct.unpack("<I", b[6 + 4 * i: 10 + 4 * i])[0] for i in range((len(b) - 6) // 4)]
            if code != EV_OBJECT_ADDED or not params:
                continue
            handle = params[0]
            size, fname = object_info(cam.op(OP_GET_OBJECT_INFO, handle))
            t0 = time.time()
            data = cam.op(OP_GET_OBJECT, handle)
            path = os.path.join(out, fname or f"{handle:08x}.bin")
            tmp = path + ".part"
            with open(tmp, "wb") as f:
                f.write(data)
            os.replace(tmp, path)
            dt = max(time.time() - t0, 0.001)
            log(f"{fname}: {len(data) / 1e6:.1f} MB in {dt:.1f} s ({len(data) / 1e6 / dt:.1f} MB/s)")
    except KeyboardInterrupt:
        log("stopping")
    finally:
        cam.close()


def main():
    args = sys.argv[1:]
    if not args or args[0].startswith("-"):
        print(__doc__)
        sys.exit(2)
    out = args[0]
    os.makedirs(out, exist_ok=True)
    name = args[args.index("--name") + 1] if "--name" in args else "Apature"
    ip = args[args.index("--camera") + 1] if "--camera" in args else None
    mode = "?"
    if ip is None:
        found = discover()
        if not found or not found[0]:
            raise SystemExit("no camera advertising on this network; join the same Wi-Fi and start Connect to PC on the camera")
        ip, mode, inst = found
        log(f"found {inst} at {ip}, mode {mode}")
    if mode == "PAIR":
        pair(ip, name)
        log("now finish the wizard on the camera; when it shows connected, run again to receive")
    else:
        receive(ip, name, out)


if __name__ == "__main__":
    main()
