# Security

## Reporting a vulnerability

Please don't open a public issue for security problems. Report them privately through GitHub:
**Security › Report a vulnerability** on this repo ([direct link](https://github.com/McCal-Codes/apature/security/advisories/new)).

If you can't use GitHub, email [contact@mcc-cal.com](mailto:contact@mcc-cal.com). Include what's affected, how to
reproduce it, and the Apature version and macOS version.

## What Apature touches

Apature reads camera files and sidecars, writes sidecars, exports and copies, and talks to the delivery targets you
configure (folder, FTP, FTPS, SFTP). Delivery passwords live in the macOS Keychain, never in the settings file.
Parsers for EXIF, XMP, IPTC and TIFF treat every input as untrusted and are capped in size; fuzzing them is on the
plan.

## Supported versions

Apature is in development (0.x). Fixes go into the latest release only.
