# Todo: Search and read

_Known bugs, deferred improvements, and open questions. Updated throughout the feature lifecycle._

## Bugs

None.

## Deferred

- [ ] A local search index or cache of `Store`, if a scan takes too long on a large memory repository. Blocked by real numbers from use.
- [ ] Word-boundary matching and stemming in `search`.
- [ ] The read budget on a shared server, where a session is an HTTP session. Out of scope for now (server mode).
- [ ] Per-user read limits on a server. Out of scope for now (server mode).

## Open Questions

None.

## Notes for other specs

- `gitsync::log` gets `last_commit_info` and `renamed_to`. `git-sync` must list them (see `decisions.md`).
- `quality-checks` and `updates` use `src/store/`. `quality-checks` and `updates` must list `search-and-read` in `requires`.

## Done in the implementation (2026-11-14)

- All 22 `[auto]` cases pass in `tests/it_search_and_read.rs`. A scan of 2 000 documents took well under 500 ms in a debug build. The timing test T21 was removed on 2026-11-17 (see `decisions.md`).
- Deferred: a cache for `Store` if a memory repository grows past some thousand files.
