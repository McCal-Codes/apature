# Contributing

Thanks for helping with Apature. Bug reports, ideas and fixes are all welcome.

## Before you start

- **Bugs:** use the [bug form](https://github.com/McCal-Codes/apature/issues/new/choose). Include the Apature version
  (About Apature shows version and build), macOS version, the camera and file type, and the steps. Help › Reveal Log
  in Finder gives you `apature.log`, which records every ingest, export, send and error.
- **Bigger changes:** open an issue first for anything that touches how files are copied, named or written.
- **Security problems:** report privately, see [SECURITY.md](SECURITY.md).
- **Artists and animators:** the mascot, icon and marketing are open for collaboration; read the
  [artist brief](docs/design/ARTIST_BRIEF.md) and open an issue with a few pieces of your work.
- Everyone follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## The rules the code follows

- Originals are never modified. Only sidecars, exports and copies are written, and every write goes to a temporary
  name first.
- A feature is done when its result is verified, not when it looks right: tests for logic, the headless screenshot
  harness for the UI (`--screenshot`, `--size`, `--ingest`, `--caption`, `--export`, `--settings`, `--crop`), and a
  real file or card when the change touches them. Say what you measured.
- Every decision that is hard to undo gets a dated entry in [docs/DECISIONS.md](docs/DECISIONS.md).
- No new dependency without a reason in the pull request and a line in [docs/THIRD_PARTY.md](docs/THIRD_PARTY.md).

## Checking a change

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
scripts/bundle.sh            # the app must still build and launch
```

CI runs the first three on Linux and builds, tests and smoke-runs the desktop app on macOS.

## Pull requests

One concern per pull request. A user-visible change adds a line under the unreleased section of
[CHANGELOG.md](CHANGELOG.md). Screenshots must not show personal photos or captions you would not publish.
