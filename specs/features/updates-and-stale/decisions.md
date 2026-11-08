# Decisions: Updates and stale documents

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## `updates` with an explicit date — 2026-11-06

**Decision:** The command is named `updates` and takes `--since <date>`. Without it, it uses the last call.

**Reason:** A clearer name and an explicit date.

## `updates` is in the first release — 2026-11-06

**Decision:** The skill runs it at session start (at read level `auto`).

**Reason:** It is cheap to build (reads Git history) and closes the "no push" gap.

## `stale` has the same shape — 2026-11-06

**Decision:** `stale [<folder>...] [--before <date>]`.

**Reason:** Same shape as `updates --since`, so it is easy to learn.
