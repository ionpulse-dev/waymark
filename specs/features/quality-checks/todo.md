# Todo: Quality checks

_Known bugs, deferred improvements, and open questions. Updated throughout the feature lifecycle._

## Bugs

None.

## Deferred

- [ ] A setting to turn off or extend built-in secret patterns (allow list). Blocked by real false positives.
- [ ] A command for CI on the Git host. Blocked by demand. Today a CI job can use the server through an MCP client.
- [ ] A check for near-duplicate titles (word overlap). It belongs to `write-documents`, which may use `lint::duplicates`.
- [ ] A check that flags documents whose anchor changed (`docs/ideas.md`).
- [ ] Checks on a shared server. Out of scope for now (server mode).
- [ ] Index files per team or per tag. Today an index is per folder.

## Open Questions

- Is CI on the Git host allowed in the target organizations? This needs the product owner. It does not block the first release.

## Done in the implementation (2026-11-14)

- All 24 `[auto]` cases pass (`tests/it_quality_checks.rs`, `tests/lint_conformance.rs`).
- Deferred: an allow list for the built-in patterns, when real use shows false findings.
