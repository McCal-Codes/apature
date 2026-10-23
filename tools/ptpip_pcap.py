#!/usr/bin/env python3
"""Decode a PTP/IP capture (tcpdump -w, port 15740) into readable packets.

    python3 -I tools/ptpip_pcap.py /tmp/wtu-transfer.pcap

Prints each TCP stream's packets in order: init requests and acks, every
operation with its code and parameters, responses with their codes, data
phase sizes and the first bytes, and events. Enough to see how a host asks
a camera for a frame.
"""
import struct
import sys

PT = {1: "Init_Command_Request", 2: "Init_Command_Ack", 3: "Init_Event_Request", 4: "Init_Event_Ack", 5: "Init_Fail",
      6: "Cmd_Request", 7: "Cmd_Response", 8: "Event", 9: "Start_Data", 10: "Data", 11: "Cancel", 12: "End_Data", 13: "Ping", 14: "Pong"}


def streams_of(path):
    f = open(path, "rb").read()
    magic = struct.unpack("<I", f[:4])[0]
    e = "<" if magic in (0xA1B2C3D4, 0xA1B23C4D) else ">"
    linktype = struct.unpack(e + "I", f[20:24])[0]
    off = 24
    streams = {}
    order = []
    while off + 16 <= len(f):
        _, _, incl, _ = struct.unpack(e + "IIII", f[off:off + 16])
        off += 16
        pkt = f[off:off + incl]
        off += incl
        ip = pkt[14:] if linktype == 1 else pkt[4:] if linktype == 0 else pkt
        if len(ip) < 20 or (ip[0] >> 4) != 4 or ip[9] != 6:
            continue
        ihl = (ip[0] & 15) * 4
        src = ".".join(map(str, ip[12:16]))
        dst = ".".join(map(str, ip[16:20]))
        tcp = ip[ihl:]
        if len(tcp) < 20:
            continue
        sp, dp = struct.unpack(">HH", tcp[:4])
        doff = (tcp[12] >> 4) * 4
        payload = tcp[doff:]
        if not payload:
            continue
        key = (src, sp, dst, dp)
        if key not in streams:
            streams[key] = b""
            order.append(key)
        streams[key] += payload
    return [(k, streams[k]) for k in order]


def describe(t, body):
    if t == 1:
        guid = body[:16].hex()
        name = body[16:-4].split(b"\x00\x00")[0].decode("utf-16-le", "ignore")
        return f"identity {guid} name {name!r}"
    if t == 2:
        conn = struct.unpack("<I", body[:4])[0]
        name = body[20:-4].split(b"\x00\x00")[0].decode("utf-16-le", "ignore")
        return f"connection {conn} camera {name!r}"
    if t == 3:
        return f"connection {struct.unpack('<I', body[:4])[0]}"
    if t == 5:
        return f"reason {struct.unpack('<I', body[:4])[0]:#x}"
    if t == 6:
        _, op, tid = struct.unpack("<IHI", body[:10])
        params = [struct.unpack("<I", body[i:i + 4])[0] for i in range(10, len(body) - 3, 4)]
        return f"op {op:#06x} tid {tid} params {[f'{p:#x}' for p in params]}"
    if t == 7:
        code, tid = struct.unpack("<HI", body[:6])
        params = [struct.unpack("<I", body[i:i + 4])[0] for i in range(6, len(body) - 3, 4)]
        return f"code {code:#06x} tid {tid} params {[f'{p:#x}' for p in params]}"
    if t == 8:
        code, tid = struct.unpack("<HI", body[:6])
        params = [struct.unpack("<I", body[i:i + 4])[0] for i in range(6, len(body) - 3, 4)]
        return f"event {code:#06x} tid {tid} params {[f'{p:#x}' for p in params]}"
    if t == 9:
        tid = struct.unpack("<I", body[:4])[0]
        total = struct.unpack("<Q", body[4:12])[0]
        return f"tid {tid} total {total} bytes"
    if t in (10, 12):
        tid = struct.unpack("<I", body[:4])[0]
        return f"tid {tid} {len(body) - 4} bytes, head {body[4:28].hex()}"
    return f"{len(body)} bytes"


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(2)
    for key, data in streams_of(sys.argv[1]):
        print(f"== {key[0]}:{key[1]} -> {key[2]}:{key[3]}")
        i = 0
        while i + 8 <= len(data):
            ln, t = struct.unpack("<II", data[i:i + 8])
            if ln < 8:
                print("  (desync)")
                break
            body = data[i + 8:i + ln]
            print(f"  {PT.get(t, t):22} {describe(t, body)}")
            i += ln


if __name__ == "__main__":
    main()
