#!/usr/bin/env python3
"""PTP/IP receiver spike: be the paired computer for a Nikon Z over Wi-Fi.

    python3 -I tools/ptpip_spike.py CAMERA_IP GUID_HEX "HOST NAME" OUT_DIR

The camera (Connect to PC, Auto send) accepts only the host identity that
was paired through Nikon's Wireless Transmitter Utility: a 16-byte GUID and
a name. Captured from one utility session (tcpdump on port 15740), they can
be replayed; the camera then treats this script as the paired computer.

What it does, read-only on the camera:
  1. Init_Command_Request with the GUID + name → expects Init_Command_Ack.
  2. Init_Event_Request on a second socket with the connection number.
  3. GetDeviceInfo (0x1001), OpenSession (0x1002).
  4. Waits for events; on ObjectAdded (0x4002) it does GetObjectInfo (0x1008)
     and GetObject (0x1009) and writes the file into OUT_DIR.
Ctrl-C closes the session. Nothing is deleted on the camera.
"""
import os
import socket
import struct
import sys
import time

PT = {1: "InitCmdReq", 2: "InitCmdAck", 3: "InitEvtReq", 4: "InitEvtAck", 5: "InitFail",
      6: "CmdReq", 7: "CmdResp", 8: "Event", 9: "StartData", 10: "Data", 11: "Cancel", 12: "EndData", 13: "Ping", 14: "Pong"}


def pkt(t, payload=b""):
    return struct.pack("<II", 8 + len(payload), t) + payload


def read_pkt(s):
    hdr = b""
    while len(hdr) < 8:
        c = s.recv(8 - len(hdr))
        if not c:
            raise ConnectionError("closed")
        hdr += c
    ln, t = struct.unpack("<II", hdr)
    body = b""
    while len(body) < ln - 8:
        c = s.recv(min(65536, ln - 8 - len(body)))
        if not c:
            raise ConnectionError("closed mid-packet")
        body += c
    return t, body


class Camera:
    def __init__(self, ip, guid, name):
        self.ip, self.guid, self.name = ip, guid, name
        self.tid = 0
        self.cmd = None
        self.evt = None

    def connect(self):
        self.cmd = socket.create_connection((self.ip, 15740), timeout=20)
        body = self.guid + self.name.encode("utf-16-le") + b"\x00\x00" + struct.pack("<I", 0x00010000)
        self.cmd.sendall(pkt(1, body))
        t, b = read_pkt(self.cmd)
        if t != 2:
            reason = struct.unpack("<I", b[:4])[0] if len(b) >= 4 else -1
            raise RuntimeError(f"camera refused the hello: {PT.get(t, t)} reason {reason:#x}")
        conn = struct.unpack("<I", b[:4])[0]
        cam_name = b[20:].split(b"\x00\x00")[0].decode("utf-16-le", "ignore")
        print(f"hello accepted · connection {conn} · camera {cam_name!r}")
        self.evt = socket.create_connection((self.ip, 15740), timeout=20)
        self.evt.sendall(pkt(3, struct.pack("<I", conn)))
        t, _ = read_pkt(self.evt)
        if t != 4:
            raise RuntimeError(f"event channel refused: {PT.get(t, t)}")
        self.evt.settimeout(None)

    def op(self, code, *params, want_data=False):
        self.tid += 1
        body = struct.pack("<IHI", 1, code, self.tid) + b"".join(struct.pack("<I", p) for p in params)
        self.cmd.sendall(pkt(6, body))
        data = b""
        while True:
            t, b = read_pkt(self.cmd)
            if t == 9:
                total = struct.unpack("<Q", b[4:12])[0]
            elif t in (10, 12):
                data += b[4:]
                if t == 12:
                    pass
            elif t == 7:
                rc = struct.unpack("<H", b[:2])[0]
                if rc != 0x2001:
                    raise RuntimeError(f"op {code:#06x} failed: {rc:#06x}")
                return data
            else:
                print("  unexpected", PT.get(t, t), len(b))

    def close(self):
        try:
            self.op(0x1003)
        except Exception:
            pass
        for s in (self.cmd, self.evt):
            try:
                s.close()
            except Exception:
                pass


def ptp_string(b, off):
    n = b[off]
    s = b[off + 1: off + 1 + n * 2].decode("utf-16-le", "ignore").rstrip("\x00")
    return s, off + 1 + n * 2


def parse_device_info(b):
    # StandardVersion u16, VendorExtensionID u32, VendorExtensionVersion u16, VendorExtensionDesc str,
    # FunctionalMode u16, OperationsSupported u16[], EventsSupported u16[], DevicePropertiesSupported u16[],
    # CaptureFormats u16[], ImageFormats u16[], Manufacturer str, Model str, DeviceVersion str, SerialNumber str
    off = 8
    _, off = ptp_string(b, off)
    off += 2
    arrays = []
    for _ in range(5):
        n = struct.unpack("<I", b[off:off + 4])[0]
        arrays.append(struct.unpack("<%dH" % n, b[off + 4: off + 4 + 2 * n]))
        off += 4 + 2 * n
    manu, off = ptp_string(b, off)
    model, off = ptp_string(b, off)
    ver, off = ptp_string(b, off)
    serial, off = ptp_string(b, off)
    return manu, model, ver, serial, arrays[0], arrays[1]


def object_info(b):
    # StorageID u32, ObjectFormat u16, ProtectionStatus u16, ObjectCompressedSize u32, ... Filename str at a fixed offset chain
    size = struct.unpack("<I", b[8:12])[0]
    off = 52
    name, _ = ptp_string(b, off)
    return size, name


def main():
    if len(sys.argv) < 5:
        print(__doc__)
        sys.exit(2)
    ip, guid_hex, name, out = sys.argv[1:5]
    os.makedirs(out, exist_ok=True)
    cam = Camera(ip, bytes.fromhex(guid_hex), name)
    cam.connect()
    info = cam.op(0x1001, want_data=True)
    manu, model, ver, serial, ops, events = parse_device_info(info)
    print(f"device · {manu} {model} · firmware {ver} · serial {serial}")
    print(f"  operations: {' '.join(f'{o:#06x}' for o in ops)}")
    print(f"  events: {' '.join(f'{e:#06x}' for e in events)}")
    cam.op(0x1002, 1)
    # What Nikon's utility does next, from the capture: a vendor query
    # (0x952b, returns 8 bytes) and a vendor set (0x935a with 0x2001), which
    # is presumably "the computer is ready; send as you shoot".
    try:
        r = cam.op(0x952b, want_data=True)
        print(f"  0x952b -> {r.hex()}")
    except Exception as e:
        print(f"  0x952b failed: {e}")
    try:
        cam.op(0x935a, 0x2001)
        print("  0x935a(0x2001) ok")
    except Exception as e:
        print(f"  0x935a failed: {e}")
    print("session open · waiting for frames (Ctrl-C to stop)")
    try:
        while True:
            t, b = read_pkt(cam.evt)
            if t == 8:
                code = struct.unpack("<H", b[:2])[0]
                params = [struct.unpack("<I", b[6 + 4 * i: 10 + 4 * i])[0] for i in range((len(b) - 6) // 4)]
                print(f"event {code:#06x} {params}")
                if code == 0x4002 and params:
                    handle = params[0]
                    oi = cam.op(0x1008, handle, want_data=True)
                    size, fname = object_info(oi)
                    print(f"  new object {handle:#x} {fname} {size} bytes; fetching")
                    t0 = time.time()
                    data = cam.op(0x1009, handle, want_data=True)
                    path = os.path.join(out, fname or f"{handle:08x}.bin")
                    with open(path, "wb") as f:
                        f.write(data)
                    dt = time.time() - t0
                    print(f"  wrote {path} ({len(data)} bytes, {len(data) / 1e6 / max(dt, 0.001):.1f} MB/s)")
            elif t == 13:
                cam.evt.sendall(pkt(14))
            else:
                print(f"packet {PT.get(t, t)} len {len(b)} head {b[:24].hex()}")
    except KeyboardInterrupt:
        pass
    finally:
        cam.close()
        print("closed")


if __name__ == "__main__":
    main()
