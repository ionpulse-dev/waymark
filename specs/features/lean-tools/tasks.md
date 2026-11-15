# Tasks: Lean tools

- [x] Fix the three hints (`delete.rs`, `remove_folder.rs`, `verify.rs`) and the hint of `show`.
- [x] `src/tools/search.rs`: optional query, list mode, defaults. Remove `src/tools/list.rs` and its registration.
- [x] `src/config/limits.rs` and `src/mcp/tool.rs`: the alias `list`, and the end of the 0.4.0 call aliases.
- [x] `src/gitsync/log.rs` and `src/tools/show.rs`: `deprecation_reason`.
- [x] `src/tools/update_document.rs`: `replace`, `append`, `add_tags`, `remove_tags`.
- [x] `src/tools/updates.rs`: `offset`, `next_offset`, and the description of `until`.
- [x] Shorten the shared parameter texts for `preview`, `id`, and `base` in every tool.
- [x] Move the existing tests from `list` to `search`, and from the old aliases to the new rules.
- [x] `tests/it_lean_tools.rs`: the cases of `test.md`, and the stale-name test.
- [x] Update the other specs, the README, `docs/install.md`, and `INDEX.md`. Remove the done item from `write-documents/todo.md`.
- [x] Run the full suite and the budget test.
