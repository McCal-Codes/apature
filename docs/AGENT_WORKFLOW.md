# Working with coding agents

One document, so every tool reads the same rules. `CLAUDE.md` and `AGENTS.md`
only point here. The handoff log stays in `docs/PLAN.md`; there is no second
task tracker.

## Who does what

One owner per change. Two agents never edit the same working tree.

* **Planning and judgement** (architecture, hard debugging, interaction
  analysis, cross-crate changes, explaining Rust): the stronger model, producing
  a bounded plan.
* **Scoped work** (tests, fixtures, small refactors, CI and release checks,
  an independent review of a finished diff): a cheaper model or a second tool,
  given a task brief, not the whole conversation.
* **Concurrent work** uses separate git worktrees or topic branches.

A task brief has: the goal, the files that matter, the constraints (the
standards in `docs/ENGINEERING_STANDARDS.md`), the acceptance check, and the
commit it starts from.

## Spending attention and usage

* Start a new task in a fresh session; read only the files it touches.
* No repo-wide reads, no subagents for a simple change, no re-doing research
  that is already in `docs/research/`.
* Targeted tests while working; the full set (`cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets`, `cargo test --workspace`) before a
  merge.
* Near a limit: finish the current atomic change, then write the handoff
  (what is done, what is next, what is risky) at the top of `docs/PLAN.md`.

## Rules that never change

* Commits, branches and PRs carry no AI attribution.
* Marketing and design copy are written by the owner; agents give structure
  and placeholders, not finished copy.
* Public text never names the owner's outlet or assignment sheets.
* Headless runs set `APATURE_CONFIG_DIR` to a temporary folder; the real
  `prefs.json` is never touched by a test.
* Look at the result (screenshot, contact sheet, a real keystroke) before
  saying visual or input work is done.
