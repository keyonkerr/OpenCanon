# OpenCanon

[English](README.md) | [中文](README.zh-CN.md)

Deterministic document atom store: split technical documentation into single-fact atoms — **record each fact once; everywhere else only references it**.

The CLI binary is `opencanon`. The agent reads skills, calls the LLM, and invokes commands in order; the Rust in this repo does validation, lifecycle transitions, persistence, and deterministic compute only — no LLM or HTTP dependencies.

## Quickstart

macOS / Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/keyonkerr/OpenCanon/master/scripts/install.sh | sh
```

Windows:

```powershell
irm https://raw.githubusercontent.com/keyonkerr/OpenCanon/master/scripts/install.ps1 | iex
```

Then run `opencanon help` to confirm the binary is on PATH. Day-to-day usage is in the next section.

## How to use

You pick a source document or ask a question; the agent reads the skills and calls commands. Do not hand-edit `opencanon/atoms/` or `opencanon/docs/`. Do not run `init` in this repo (the product tree).

### 1. Initialize in the governed project (once per project)

Go to the root of **the project whose docs you want to govern**:

```bash
cd /path/to/your-project
opencanon init
```

Pick document languages in the terminal. That creates `opencanon/` (including `atoms/` and `config.yaml`) and installs `opencanon-atomize` and `opencanon-compose` into `.agents/skills/`. A TTY is required; it fails without one.

If your agent **does not** load `.agents/skills/` (for example it only uses Cursor / Claude Code / Codex skill directories), copy those two folders into whichever skills path it does load — whole directories, keep the names. They are in the project's `.agents/skills/`, or in this repo's [`skills/`](skills/).

### 2. Write: split a document into atoms

In that project, point the agent at a source document, for example:

- “Atomize `docs/xxx.md`”
- “Migrate this wiki into OpenCanon”

The agent runs `opencanon-atomize`: it extracts single-fact claims, compares them to atoms already in the store (reuse or `edit` if it is the same fact), promotes when it can check against the implementation, and asks you only when it cannot. Claims in the source stay untouched; a true-source link to `opencanon/atoms/<id>.md` is appended at the end of each migrated paragraph. Only atoms with `status: active` are the true source.

### 3. Read: compose an answer from atoms

Once atoms exist, ask the agent, for example:

- “Answer from the atom store: …”
- “Compose a readable document: …”

The agent runs `opencanon-compose`: it recalls `active` atoms for the question and writes prose without inventing facts that are not in those atoms. If you only need it in the session, it shows the body. If you want it on disk or elsewhere, it writes `opencanon/docs/` and places a link at the other path — it does not copy the body.

The closed loop is in [`docs/loop.md`](docs/loop.md) (Chinese).

## What it solves

Docs grow repetitive, nobody knows which copy is authoritative, and they drift out of sync with the current code. Migrated facts live in the governed project's `opencanon/atoms/`; only files with `status: active` are the true source. Claims in the source document stay untouched — a link is appended at the end of each migrated paragraph.

## Who does what

| Role | Does | Does not |
|------|------|----------|
| Human | Picks source documents, asks questions; adjudicates only when the LLM can't check against the implementation | — |
| Agent | Reads skills, reads source files and the implementation itself, calls commands | Hand-edit `opencanon/atoms/` or `opencanon/docs/` |
| `opencanon` | Validation, persistence, lifecycle, substring recall, coarse freshness scoring | LLM calls, orchestration, reading/writing source docs |
