# Tests: Clean Git status

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a new memory repository, **When** `.DS_Store` files are created, **Then** `git status` lists nothing | A1 |
| **Given** a new memory repository, **When** Obsidian saves `app.json` with the same settings, **Then** `git status` lists nothing | A2, M1 |
| **Given** a `.gitignore` with no `.DS_Store` line, **When** a write tool runs, **Then** the lines are added in a repair commit with a notice | A3, U1 |
| **Given** a `.gitignore` with `.DS_Store` or `**/.DS_Store`, **When** a write tool runs, **Then** no repair commit | A3, U2 |
| **Given** the old waymark `app.json`, **When** a write tool runs, **Then** it gets the Obsidian bytes in a repair commit with a notice | A4, U3 |
| **Given** the old `app.json` and Obsidian already saved its form, **When** a write tool runs, **Then** the file is committed and `git status` is clean | A5 |
| **Given** an `app.json` with other keys or values, **When** a write tool runs, **Then** it is not changed | A6, U4 |
| **Given** no `.gitignore` or no `app.json`, **When** a write tool runs, **Then** no file is created | A7 |
| **Given** a repair is needed, **When** a read, a preview, or a read-only call runs, **Then** nothing is committed | A8 |
| **Given** a repair was made, **When** the write runs again, **Then** no repair commit | A3, A4, U5 |

## Test Cases

Unit tests in `src/memrepo/repairs.rs`:

- [x] [auto] U1: `.gitignore` text without the line gets the empty line, the comment, and `.DS_Store` at the end. A text with no final line end gets one first. Every other line is unchanged.
- [x] [auto] U2: a text with a line `.DS_Store`, `**/.DS_Store`, or `  .DS_Store  ` (spaces) needs no repair.
- [x] [auto] U3: the old `app.json` (with a final line end), and the same settings in another key order or indent, give the template.
- [x] [auto] U4: an `app.json` with a third key, another value, or text that is not JSON needs no repair. The template itself needs no repair.
- [x] [auto] U5: every repair is safe to run twice: the output of a repair needs no further repair.
- [x] [auto] U6: the template `obsidian-app.json` does not end with a line end, equals the exact text of the requirements, and parses to the two settings (`src/memrepo/init.rs`).

Integration tests:

- [x] [auto] A1: after `init`, `.DS_Store` files in the root, in `agent-memory/`, and in a folder do not show in `git status --porcelain` (`tests/it_create_memory_repository.rs`).
- [x] [auto] A2: after `init`, writing the bytes of the template into `.obsidian/app.json` leaves `git status --porcelain` empty. Test A7 of `create-memory-repository` checks the exact bytes of both files (`tests/it_create_memory_repository.rs`).
- [x] [auto] A3: a memory repository with the old `.gitignore` committed: a read and a preview commit nothing. A write makes `waymark: repair ".gitignore"` before its own commit, with the notice. The old lines are unchanged. A second write makes no repair commit. A `.gitignore` that has `**/.DS_Store` gets no repair (`tests/it_memory_repository.rs`).
- [x] [auto] A4: a memory repository with the old `app.json` committed: a write makes `waymark: repair ".obsidian/app.json"` with the notice, and the file has the template bytes. A second write makes no repair commit (`tests/it_memory_repository.rs`).
- [x] [auto] A5: the old `app.json` committed, and the template bytes in the working tree (as Obsidian saves it): a write commits the repair, and `git status --porcelain` is empty after it (`tests/it_memory_repository.rs`).
- [x] [auto] A6: an `app.json` with a third key committed: a write leaves it unchanged. A `.gitignore` with an uncommitted edit: a write does not repair it, and the edit stays uncommitted (`tests/it_memory_repository.rs`).
- [x] [auto] A7: with `.gitignore` and `.obsidian/app.json` removed and committed, a write creates neither file (`tests/it_memory_repository.rs`).
- [x] [auto] A8: with personal `access = "read-only"`, a call commits no repair (`tests/it_memory_repository.rs`).
- [x] [auto] The existing repair tests of the settings file still pass: `a_write_repairs_a_missing_schema_in_its_own_commit`, `a_repair_keeps_the_values_that_the_owner_set` (`tests/it_memory_repository.rs`).

Manual tests:

- [ ] [manual] M1: Obsidian on a Mac. Steps: run `init` for a new memory repository, or use one that a repair fixed. Clone it. Open the folder as a vault in Obsidian. Open the settings, change one setting and change it back, then close Obsidian. Open the folders in Finder. Run `git status` in the clone. Expected: `git status` shows nothing to commit (no `.DS_Store`, no `.obsidian/app.json`).

## Notes

- The integration tests use the `World` helpers of `tests/common`. The repair tests commit the old file under the author "Host", as `remove_schema_and_commit` does.
- 2026-11-17, macOS, Rust 1.100.0 (through mise): all unit tests and all integration tests pass. Four timing tests of other specs (`it_maintenance` a6, `it_search_and_read` t21, two `personal_server` tests) failed only while the whole suite ran at once, and pass alone. `make check` stops before the tests on a clippy error and on a format difference in the `document-names` code, and the conformance script fails one check on the Linux note in `README.md`. Both come from earlier commits, not from this spec. `make linux-test` was not run: this Mac has no Docker.
