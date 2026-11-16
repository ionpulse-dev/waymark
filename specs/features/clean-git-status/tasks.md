# Tasks: Clean Git status

_Written after `plan.md` is approved. Bite-sized implementation checklist. AI checks off items as work is completed._
_List tasks in strict prerequisite-first order. A task may only appear after all tasks it depends on._

Prerequisites: `create-memory-repository` and `memory-repository` are `implemented`.

- [x] T1: Update the spec `create-memory-repository`: the table "What `init` does" gets the new content of `.gitignore` and `.obsidian/app.json`, with a link to this spec. Test A7 in its `test.md` names the exact bytes.
- [x] T2: Update the spec `memory-repository`: the layout line of `.gitignore`, the row "Repair", and a list item for the two new repairs with a link to this spec.
- [x] T3: Change the template `src/memrepo/templates/gitignore` (comment lines and `.DS_Store`).
- [x] T4: Change the template `src/memrepo/templates/obsidian-app.json` (no final line end). Make the constant public in `src/memrepo/init.rs` so the repair can use it. Add unit test U6.
- [x] T5: In `src/memrepo/repairs.rs`, add `Target` and the field `target`. Change `repair` to take the target. Keep the settings repair as it is.
- [x] T6: Add the repair `gitignore-ds-store` with unit tests U1, U2, U5.
- [x] T7: Add the repair `obsidian-app-json-format` with unit tests U3, U4, U5.
- [x] T8: In `src/writes/mod.rs`, loop the repair step over the three targets: path, `HEAD` text, working-tree text, the rule for uncommitted changes, one commit per file, the notices.
- [x] T9: Integration tests A1 and A2, and the exact bytes in A7 of `create-memory-repository` (`tests/it_create_memory_repository.rs`).
- [x] T10: Integration tests A3 to A8 (`tests/it_memory_repository.rs`).
- [x] T11: Add `.claude/worktrees/` to `.gitignore` of this code repository.
- [x] T12: Add an entry to "Unreleased" in `CHANGELOG.md`.
- [ ] T13: Build and test on a machine with Rust: `make check` and `make linux-test`. Fix what fails. Tick the `[auto]` items in `test.md`. Set the status to `implemented`.
