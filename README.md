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

Then run `opencanon init` in the root of **the project whose docs you want to govern** (not in this repo). After that, point the agent at a source document and run `opencanon-atomize` to write; ask questions with `opencanon-compose` to read.

## What it solves

Docs grow repetitive, nobody knows which copy is authoritative, and they drift out of sync with the current code. Migrated facts live in the governed project's `opencanon/atoms/`; only files with `status: active` are the true source. Claims in the source document stay untouched — a link is appended at the end of each migrated paragraph.

## Who does what

| Role | Does | Does not |
|------|------|----------|
| Human | Picks source documents, asks questions; adjudicates only when the LLM can't check against the implementation | — |
| Agent | Reads skills, reads source files and the implementation itself, calls commands | Hand-edit `opencanon/atoms/` or `opencanon/docs/` |
| `opencanon` | Validation, persistence, lifecycle, substring recall, coarse freshness scoring | LLM calls, orchestration, reading/writing source docs |
