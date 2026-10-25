# Capabilities by platform

Read from the code on 2026-10-24, not measured on every platform. Windows and
Linux columns say what the source does today; confirm on a real machine
before a release (`docs/RELEASES.md`). A feature that needs something outside
Apature says what.

| capability | macOS | Windows | Linux | needs |
|---|---|---|---|---|
| Contact sheet, culling, marks, captions, export (JPEG) | yes | yes | yes | — |
| RAW through the embedded preview | yes | yes | yes | the file's own preview |
| RAW through the system RAW engine on export (`--decode-raw`) | yes (ImageIO) | no | no | macOS |
| JPEG XL export | yes | untested | untested | `cjxl` from libjxl (`APATURE_CJXL`, Homebrew, PATH) |
| Native menu bar | yes | no | no | — |
| Bare-key shortcuts | yes (handled in the view) | yes | yes | — |
| Move to Trash with ⌘Z restore | yes (exact path from `trashItemAtURL`) | Trash entries are not located, so no restore | searched in `~/.local/share/Trash/files` | system Trash |
| Eject a card | yes | yes (PowerShell) | not checked | — |
| Secret storage for delivery passwords | Keychain | Credential Manager | system keyring | — |
| Camera over Wi-Fi (discovery, pairing, receive) | yes (uses the system `dns-sd`) | no | no | a Nikon in Wi-Fi mode on the same network |
| FTP receive for phones and FTP-capable cameras | yes | yes | yes | same network |
| Update check | yes | yes | yes | network |

Where the app cannot do something on the current system it should say so in
the interface rather than fail quietly; the inventory above is the list to
check that against. A central `capabilities()` function that the UI asks is
the next step; today each feature checks for itself.
