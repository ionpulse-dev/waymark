# Todo: Join a knowledge base

## Bugs

None.

## Deferred

- A server workspace for each knowledge base (out of scope for now, server mode).
- The HTTPS entry of a client (server mode).
- The key `[kb."<uuid>".limits]` in the personal config: it tightens limits for one knowledge base. `trust-model` has no such layer yet. Add it there first.
- Clean temporary clone folders of dead processes that live in other folders than the current path.
- Offer a progress notice while the clone runs.

## Open Questions

None.

## Notes 2026-11-14

- The `git --version` time-out of 5 seconds is not built (A1 uses a fake `git` that fails fast).
- Decide whether the default folder should be `~/Projects/<repository-name>` (see `decisions.md`).
- The shared layer is missing from the limits of the very first call after a clone (see `decisions.md`). Re-read the limits after the wait if it matters.
