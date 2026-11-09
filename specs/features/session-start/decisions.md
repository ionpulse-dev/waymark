# Decisions: Session start

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## `session` informs, commands enforce — 2026-11-07

**Decision:** The start command only informs. Every command checks the rules itself.

**Reason:** Advice in the skill is not a control.

## Settings in every result — 2026-11-07

**Decision:** The agent follows the `settings` of the latest result, not the ones from the start.

**Reason:** A change in the app then applies at once, also in a running session.

## No updates in the session result — 2026-11-09

**Decision:** `session` does not include `updates`, also at read level `auto`.

**Reason:** A change report at the start costs context in every session. The person can ask for `updates` at any time. The tested rules of the waymark-trail skills dropped the startup report.
