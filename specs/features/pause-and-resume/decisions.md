# Decisions: Pause and resume

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
