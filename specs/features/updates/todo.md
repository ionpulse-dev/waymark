# Todo: Updates

_Known bugs, deferred improvements, and open questions. Updated throughout the feature lifecycle._

## Bugs

None.

## Deferred

- [ ] A flag to include the person's own changes. `activity` covers them today.
- [ ] A check that flags documents whose anchor changed. Blocked by `docs/ideas.md`.
- [ ] The stored time per person on a server. Out of scope for now (server mode).
- [ ] Uncommitted manual edits in the list. Blocked by `manual-edits` and a clear need.

## Open Questions

None.

## Notes 2026-11-14

- Unit tests for `gitsync::diff` on a temporary repository (the integration tests cover the paths today).
- `deleted` has no end-to-end test. Add one with the `delete` tool.
- With no Git identity, `own` is empty in `src/tools/updates.rs`, so the person's own changes show as changes by others. Decide how `updates` behaves without an author (found in the code review of 2026-11-14, skipped).
