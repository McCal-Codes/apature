# Threat model

What can hurt a photographer's work through Apature, where it comes from, and
which test or rule stops it. Status is honest: *covered* has a test in the
repository; *by design* is a rule in the code without a dedicated test;
*open* is work to do.

Trust boundary: everything that arrives from outside the process is untrusted
until parsed: card and drive contents, sidecars written by other software,
assignment sheets, the camera on the network, FTP clients, update metadata.

| # | input or action | what could go wrong | mitigation | status |
|---|---|---|---|---|
| 1 | RAW/JPEG EXIF (TIFF IFDs, maker notes) | malformed offsets, loops, huge counts → panic or memory blow-up | bounds-checked reads; `exif::from_bytes` returns `None` | covered: `tests/malformed_input.rs` (noise, truncations, hostile offsets) |
| 2 | XMP sidecar and embedded XMP | garbage, huge numbers, NaN crops, wrong prefixes | regex parse with clamping; crop normalised into the frame; angle limited to ±45° and finite | covered: `malformed_input.rs`, `edits` tests |
| 3 | IPTC-IIM and JPEG segments (APP13, APP1) | wrong lengths, truncated segments | length-checked iteration | covered: `malformed_input.rs` |
| 4 | Writing metadata into a JPEG | a failed write corrupts the file | write to temp, re-read, then replace atomically; never touch image data | by design (`xmp::write_sidecar_with`, `jpeg_xmp`); fixture ExifTool comparison *open* |
| 5 | Copy / ingest from a card | silent corruption, partial copy, card removed mid-copy, name collisions | SHA-256 both sides, `.part` then rename, refuse overwrite, ledger, card read-only | covered by ingest tests; interrupted-copy resume *open* |
| 6 | Move to Trash and restore | losing a file; restore overwriting a newer one | system Trash (never `remove`); restore refuses to overwrite and reports skipped | covered: `fileops` tests; real Trash round trip (`--ignored trash_round_trip`) |
| 7 | Cloud-drive placeholders (online-only files) | a read blocks forever | detect `dataless`, use a timeout, fall back to the card | *open* in code (done by hand 2026-10-23, see memory note); needs a guard in scan/export |
| 8 | Removable drives and volumes | writing to the wrong volume; eject while busy | volume kind from DCIM/system flags; destination shown before ingest | by design |
| 9 | FTP receive (libunftp) | open port on a shared network; weak password | off by default; six-digit session password; local network only; stops with the chip | by design; auth round-trip test exists |
| 10 | Camera over Wi-Fi (PTP/IP pairing) | pairing with the wrong device; auto-trust | pairing needs the code the camera shows; identity saved only on a match | by design; pairing handshake detail is not documented here |
| 11 | Delivery targets (FTP/FTPS/SFTP) | credentials in logs or prefs | secrets in the system keyring; never logged | by design |
| 12 | Assignment sheets and code replacement files | path traversal, oversized values, template injection | values are text only; templates expand known variables; unknown ones stay literal | covered: `codes`/`stationery` tests |
| 13 | Rename templates | names that escape the folder or collide | plain-name check, collision plan refuses | covered: `fileops` tests |
| 14 | Update check (GitHub releases) | a hostile release page | opens the page in the browser; nothing is installed by Apature | by design |
| 15 | Logs and crash output | personal paths, credentials | no secrets are logged; paths are | by design |

Rule for new work: a feature that reads a new kind of input adds a row here
and a test beside the parser.
