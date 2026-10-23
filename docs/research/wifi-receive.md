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
