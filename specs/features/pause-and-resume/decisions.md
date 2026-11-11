# Decisions: Pause and resume

> **Note 2026-11-11:** "Resume needs the dialog" is superseded by "Pause is a limit" below. "Two states, no timer" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Two states, no timer — 2026-11-06

**Decision:** Running or paused, no automatic resume.

**Alternatives considered:**
- A pause with a timer — a forgotten timer silently re-enables writes.

**Reason:** A kill switch must be simple and predictable.

## Resume needs the dialog — 2026-11-06

**Decision:** `resume` uses the native approval dialog.

**Reason:** An agent must not be able to switch the switch off.

## Pause is a limit — 2026-11-11

**Decision:** Pause is the limit `paused`. The person sets it in the personal config, IT in the system config, and a server user in the console. Only the layer that set it can clear it.

**Reason:** the dialog is dropped. Honest limit: a personal flag is a safety default in local mode.
