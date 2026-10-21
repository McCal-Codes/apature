# Shared engineering standards (proposal for Folio and Apature)

Candidate principles, not rules, until each project's owner adopts them in its
own standards document with its own ids. Each entry: the principle, what it
means in practice, where it conflicts with an existing standard, and the
evidence that it is already followed somewhere.

| # | Principle | In practice | Conflicts | Evidence |
|---|---|---|---|---|
| 1 | Correctness before presentation | a feature is "done" when its output is verified, not when it looks right; UI polish follows | none; Folio's "Folio quality" and Apature's "no unmeasured claims" both say this | Apature exiftool validation; Folio `VersionCodeTest` |
| 2 | Performance backed by measurements | every performance claim carries a number, a device and a date; targets are confirmed before enforced | none; Folio PERF already marks targets "proposed" | Apature benchmarks page; Folio performance log |
| 3 | Hardware capabilities detected, never assumed | one capability table per app, computed at start, with "unavailable" as a state with a reason | none | Folio `CapabilityBroker`; Apature's `VolumeKind` is the seed |
| 4 | User data protected by default | the source is never written; every write is temp-then-rename; a damaged file is kept beside its replacement; nothing is deleted without a visible, reversible step | none | Apature `copy.rs`; Folio `state_damaged_backup` |
| 5 | Explicit state ownership | each piece of state has one owner named in a table; others read it through that owner | none; Folio STA has the table, Apature does not yet | Folio state-data.md |
| 6 | Interfaces separate from business logic | parsers, planners and copiers are testable without a UI; the UI is a thin dispatcher | Apature follows this by crate; Folio by file and partially | Apature workspace; Folio `:market` module |
| 7 | Recoverable errors, no silent failures | an error becomes a state the user can see and act on; logs record it; "it just didn't happen" is a bug | none | Folio `UnavailableReason`; Apature toasts and log |
| 8 | Accessibility and responsive behaviour | Reduce Motion honoured; keyboard or non-gesture path for everything; layouts checked at the small and large ends | none; Apature lacks screen-reader support and says so | Folio ADP/A11Y; Apature focus rings, `--size` sweeps |
| 9 | Automated tests plus real hardware | unit tests for logic, a harness for the UI, and a recorded run on the real device before a release claim | none | Folio Fold8 runs; Apature real-card ingest |
| 10 | Documented decisions | an ADR per irreversible choice, dated, with the alternative rejected | none; Apature is nine ADRs behind | both have the file |
| 11 | Compatibility with existing user data | every saved format is versioned; a newer file is refused visibly by an older build; fixtures from shipped versions load in tests | none | Folio STA-5/6/7 |
| 12 | Maintainable code, few dependencies | a new dependency needs a reason in the PR and a licence line; prefer the platform's tool when it is equivalent | none; both already keep THIRD_PARTY files | Apature cjxl/sips choice; Folio `third_party/` |

Platform rules stay in each project: Compose and Android lifecycle rules in
Folio, egui and macOS rules in Apature. No shared library is proposed.
