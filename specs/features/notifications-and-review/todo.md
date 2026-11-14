# Todo: Notices and review

_Known bugs, deferred improvements, and open questions. Updated throughout the feature lifecycle._

## Bugs

None.

## Deferred

- [ ] The review list in the web console, with an action for each item. Out of scope for now (server mode, `web-console`).
- [ ] Documents that wait for verification in the review list. Blocked by `verification` and the console.
- [ ] Email or webhook messages from a server. Out of scope for now (server mode).
- [ ] A per-person record on a server. Out of scope for now (server mode).
- [ ] A setting for the number of sessions (3) and for the window (7 days).

## Open Questions

None.

## Notes for other specs

- `maintenance` and `session-start`: `status` must return `notices::open_items`.
- `pause-and-resume`: use `notices::text::PAUSED`. Do not add a second "paused" notice.
- `mcp-server`: the registry must call `notices::finalize` after every call.

## Notes 2026-11-14

- Use `notices::text::PAUSED` in the registry instead of the literal text.
- `maintenance`: `status` already lists the open notices through `finalize`. Keep that when you add the other checks.
- T15 (manual): check how Claude Code, Copilot, and Codex show `notices`.
