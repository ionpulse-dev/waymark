# Plan: Remove Index

## Overview

Update the specs of the affected features first, then delete the tool and its
module, then remove the `is_index` special cases one by one. The test suite
guards each step.

## Components Affected

- `src/tools/index.rs`, `src/tools/mod.rs` — delete the tool and its registration.
- `src/indexgen/mod.rs`, `src/lib.rs` — delete the module.
- `src/lint/mod.rs` — delete the `index-stale` check and the `next_for` entry.
- `src/store/mod.rs` — delete `Doc::is_index()`; `docs()` no longer filters.
- `src/store/backlinks.rs`, `src/store/changes.rs` — drop the index rules.
- `src/memrepo/folders.rs` — drop `is_index_file` and the field for index files.
- `src/tools/delete.rs`, `src/tools/remove_folder.rs` — drop the index rule.
- `src/activity/mod.rs`, `src/tools/activity.rs`, `src/tools/undo.rs`,
  `src/manual/mod.rs`, `src/gitsync/write.rs`, `src/format/frontmatter.rs` —
  check for mentions and remove them if they are about the tool.
- `tests/it_quality_checks.rs`, `tests/it_search_and_read.rs`,
  `tests/it_security.rs`, `tests/it_updates.rs`, `tests/it_write_documents.rs` —
  remove the tests of the tool and the rule. Add one test that an `index.md`
  with the tag `index` is a normal document.
- Specs: `quality-checks`, `search-and-read`, `updates`, `folders`,
  `write-documents`, `pause-and-resume`, `write-approval`, `mcp-server`.
- `README.md` (tool table), `docs/install.md` (allow list),
  `docs/next-steps/open-questions.md`.
- `CHANGELOG.md` — a 0.3.0 entry.

## New Files

_None._

## Risks

- Tests share helpers that make index files. A helper may need a small change.
- The `folders` result has a field for index files. Removing it changes the
  result shape. Allowed, because the project has no users.
