# Decisions: Session start

> **Note 2026-11-11:** "`session` informs, commands enforce" is superseded by "Server instructions" below. "Settings in every result" and "No updates in the session result" stay valid.

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

## Server instructions — 2026-11-11

**Decision:** The server sends a short neutral summary as MCP instructions at connect. `status` returns the same facts. There is no `session` call and no behavior advice.

**Reason:** no skill tells the agent to call a first command. Alternative considered: a first tool that an agent must call. Nothing can make it call it.
