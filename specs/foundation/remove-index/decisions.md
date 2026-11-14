# Decisions: Remove Index

_Records why choices were made. Never delete entries — add new ones._

---

## Remove the `index` tool — 2026-11-14

**Decision (decided with the product owner):** The tool `index`, the check
`index-stale`, and every special rule for generated index files are removed.

**Alternatives considered:**
- Keep the tool as an optional extra (the state of `quality-checks` before).
  Rejected: it costs a tool description in every agent's context, extra commits,
  and special cases in six tools, and `folders`, `list`, and `search` already
  give agents the same overview.
- Generate index files at every write. Rejected earlier: merge conflicts.

**Reason:** waymark is mostly agent storage. The human need (a map of the
memory repository) fits a future memory browser better, which needs no file and
no write tool. The standard keeps the permission for tools that still want to
generate index files.
