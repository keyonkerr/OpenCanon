# OpenCanon

Deterministic document atom store. The agent drives the work and calls the LLM. This repo's Rust does validation, lifecycle, persistence, and deterministic compute, with no LLM or HTTP dependencies.

The product tree (this repo) is not the true-source tree. True source is `status: active` files under `opencanon/atoms/` in the governed project's cwd. `skills/` ships with the CLI and is not written into the consumer's `opencanon/`. `init` installs product skills into `.agents/skills/` by same-name overwrite.

## Where to change

Read that module's `AGENTS.md` before editing. Each rule has one place it may occur; delete copies and keep `canon-core`.

| Change | Open |
|--------|------|
| Validation, state machine, ids, field merge, filters, deterministic algorithms | [`crates/canon-core/AGENTS.md`](crates/canon-core/AGENTS.md) |
| Atom paths, markdown key order, atomic writes | [`crates/canon-store/AGENTS.md`](crates/canon-store/AGENTS.md) |
| Envelope, exit codes, clap, command wiring | [`crates/opencanon/AGENTS.md`](crates/opencanon/AGENTS.md) |
| Flow steps and human-review gates | [`skills/AGENTS.md`](skills/AGENTS.md) |

Stable for callers: subcommand names, stdin JSON, stdout envelope, `error.code`, atom file shape. Internals of `ops`, store temp filenames, and clap wiring may change without touching that contract.
