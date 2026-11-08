# Decisions: Session start

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## `session` informs, commands enforce — 2026-11-07

**Decision:** The start command only informs. Every command checks the rules itself.

**Reason:** Advice in the skill is not a control.

## Settings in every result — 2026-11-07

**Decision:** The agent follows the `settings` of the latest result, not the ones from the start.

**Reason:** A change in the app then applies at once, also in a running session.
