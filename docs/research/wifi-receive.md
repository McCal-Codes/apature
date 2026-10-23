# Receiving photos over Wi-Fi: the phone as a WT-7

2026-10-23. What works today, what does not, and why Apature has an FTP
server in it.

## The question

Can a camera send to Apature over a phone's hotspot, with no WT-7?

## What the Z 6 II can and cannot do

- **No FTP client.** The body's Wi-Fi speaks Nikon's own protocol (PTP/IP)
  to SnapBridge on a phone or to Wireless Transmitter Utility on a computer.
  FTP from the body needs the WT-7 (about $900) or a CamRanger.
- **SnapBridge auto-download works and is reliable.** Every frame can be sent
  to the phone as it is shot: 2 MP by default, or the full-size JPEG
  ("Original") for JPEG and RAW+JPEG. RAW files are manual, one at a time.
- **"Connect to computer" over Wi-Fi** sends to Wireless Transmitter Utility
  only. It is PTP/IP, not FTP; Apature does not speak it yet (see below).

## The chain that works now

```
Z 6 II  --SnapBridge (Bluetooth + Wi-Fi)-->  phone  --FTP over the hotspot-->  Apature  -->  Live Ingest
```

1. **Phone**: turn on Personal Hotspot. The Mac joins it. (The camera does
   not need the hotspot; SnapBridge uses its own link to the body.)
2. **SnapBridge**: Auto download on, size Original for a desk-ready JPEG.
3. **A transfer app on the phone** watches the camera roll and uploads new
   photos by FTP to the Mac. PhotoSync (iOS and Android) does this with
   "Autotransfer" to an FTP target; FolderSync on Android as well.
4. **Apature**: Ingest › Plan › **Receive over Wi-Fi**. The row shows
   `ftp://<the Mac's hotspot address>:2121`, user `apature`, and a six-digit
   password made up for the session. Put those three into the phone app.
5. Every file that lands is picked up by Live Ingest once its size has held
   for two polls: hashed, named from the template, ledgered.

Expect one to three seconds per full-size JPEG through SnapBridge, then the
hotspot hop. Enough for a game or a presser; not for a 20 fps burst.

## Cameras with an FTP client

Sony (α7 IV and up), Canon (R5, R6 II, R3) and Nikon with a WT-7 can upload
straight to Apature with the same address, user and password. Set the camera
to FTP, passive mode, no TLS (the camera menus call it FTP, not FTPS).

## Why FTP and not something newer

Every camera and phone transfer app speaks FTP; almost none speak SFTP or
WebDAV in a way you can configure on a camera body. It runs only on a
private network the photographer controls, with a password nobody can
guess from the window across the room. The server is libunftp, in Rust,
inside the app: nothing to install, nothing listening when the chip is off.

## Not built yet

- **PTP/IP responder** so the Z 6 II's own "Connect to computer" mode sends
  straight to Apature without the phone. Nikon's dialect is undocumented;
  gphoto2 has a partial initiator, not a responder. Worth a spike on a quiet
  week, since it would remove the phone from the chain for Nikon bodies.
- **Bonjour / mDNS** advertising so phone apps find Apature without typing
  the address.
- **TLS (FTPS)** for cameras that support it (Sony does): the deliver crate
  already has native-tls; the receive side needs a certificate story.

## Verified

- Round-trip test in `crates/apature-receive/tests`: start, log in, upload
  10 KB, read it back, refuse a wrong password.
- Headless run on the Mac: the server reports `ftp://192.168.1.175:2121`
  and stops cleanly. A phone has not been tested yet; that needs Caleb's
  phone, SnapBridge and a transfer app on the same hotspot.

## SnapBridge as a pseudo-bridge: can Apature talk to the camera the way the phone does? (2026-10-23)

Looked for an open implementation of what SnapBridge does so the Mac could
stand in for the phone. Findings:

- **No open reimplementation of SnapBridge's pairing exists.** Reviews and
  Nikon's manuals describe the layers (Bluetooth LE for the always-on link
  and small transfers, Wi-Fi for full-size images and live view) but nobody
  has published the BLE pairing handshake, and the Wi-Fi hand-off is
  negotiated over that BLE link. Reverse-engineering it is a project, not a
  task.
- **PTP/IP over Wi-Fi is open and works for Z bodies.** gphoto2 drives a Z6
  over `ptpip:` once the computer is on the camera's network (camera as
  access point, or both on the same router or hotspot); a German test did
  exactly that from a Raspberry Pi. The weak spot is the camera's
  "Connect to computer" pairing wizard, which waits for Nikon's Wireless
  Transmitter Utility to answer; a Z50 report shows gphoto2 identifying the
  camera and the camera then refusing to save the connection. Nothing
  documents the pairing exchange.
- **Conclusion.** Two chains, both without a WT-7:
  1. *Today, tested in the lab*: camera → SnapBridge → phone → transfer app →
     Apature's FTP server (above). Phone required.
  2. *Worth a spike*: Apature as a PTP/IP initiator on the camera's Wi-Fi,
     pulling new frames as they are shot (the same `FILEADDED` loop the
     Pocket prototype uses over USB), so the Mac replaces both the phone and
     the WT-7. Needs: the camera in access-point mode or on the hotspot, a
     Rust PTP/IP client (none mature; port the needed subset from
     libgphoto2's `ptpip.c`), and a test of whether the Z 6 II accepts a
     session from something other than WTU. Budget it as a research week,
     not a feature.

Sources: dpreview's SnapBridge test, Digital Trends on the BLE/Wi-Fi
hand-off, Nikon's D6 and Z5 manuals on the app, the PIXLS Z50 wireless
thread, PC-Welt's gphoto2 Raspberry Pi article, pypi `ptpip`.

## Correction (2026-10-23, from a research note Caleb shared): the camera can send to the Mac by itself

Nikon's own **Wireless Transmitter Utility** (WTU, free, Mac) pairs with the
Z 6 II over the body's built-in Wi-Fi (Setup › Connect to PC: the camera as
access point, or both on one network such as a phone hotspot), takes an
authentication code shown on the camera, and then receives files into a
folder of your choice. With *Connect to PC › Options › Auto send* on, every
frame goes as it is written to the card, full size, RAW+JPEG or JPEG only
(*Send file as*). No phone, no WT-7. The WT-7 is only needed for FTP from
the body and for wireless tethered control.

So the first-choice chain is:

```
Z 6 II  --built-in Wi-Fi, WTU pairing-->  a folder on the Mac  -->  Live Ingest
```

Apature already covers the right-hand half: pick the WTU folder as the
Ingest source and turn Live Ingest on. The FTP receiver stays for phones and
for cameras that speak FTP. Not yet tested on Caleb's body; the test plan
is in the note's "direct PTP/IP experiment" (pair with WTU first, then
`gphoto2 --port ptpip:<camera ip> --summary`, then `--list-files`).

The note also corrects my BLE finding: two open projects (furble, MIT; Nikon
Smart GPS, AGPL-3.0) have reverse-engineered parts of the SnapBridge
smart-device Bluetooth pairing, with Z7 II, Z50 II and Z8 reports. Image
transfer over that path is still undemonstrated. Licence check before any
reuse. Recommended staging: (1) WTU folder + Live Ingest, (2) a PTP/IP
spike with the `gphoto2` crate (a libgphoto2 wrapper with `wait_event` and
`NewFile`), (3) BLE research as its own track.
