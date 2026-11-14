# Tests: Memory repository

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| `init` result opens as an Obsidian vault with Markdown links and relative paths | `init_writes_obsidian_defaults` (auto), Obsidian check (manual) |
| Personal Obsidian files do not show in `git status` | `gitignore_hides_personal_obsidian_files` |
| `settings.toml` without `id` gives `setup_problem` | `missing_id_is_setup_problem` |
| Stricter of shared and personal value applies | `stricter_value_applies` (also in `trust-model`) |
| A reorganization needs no `owners` update | `no_owners_field_written` |

## Test Cases

- [x] [auto] `init` writes every file of the layout and a valid UUID (`src/memrepo/layout.rs`)
- [x] [auto] `init` refuses to overwrite an existing file (`src/memrepo/layout.rs`)
- [x] [auto] `.obsidian/app.json` has `useMarkdownLinks: true` and `newLinkFormat: relative` (`src/memrepo/layout.rs`)
- [x] [auto] In a repository with `.obsidian/workspace.json` created, `git status` shows no change (`tests/it_memory_repository.rs`)
- [x] [auto] Missing `id`, a bad UUID, and a syntax error give `setup_problem` (`src/memrepo/settings.rs`)
- [x] [auto] Unknown keys give a notice and no error (`src/memrepo/settings.rs`)
- [x] [auto] The root is found for the layout `agent-memory/.waymark` and fails with `setup_problem` when absent (`src/memrepo/layout.rs`)
- [x] [auto] The files written by `init` pass the conformance check of `document-format` where they are documents (`tests/it_memory_repository.rs`)
- [x] [auto] A file with a higher `schema` gives `setup_problem` and the message "needs a newer waymark"; a file without `schema` is schema 1; a `schema` of 0, -1, or "x" gives `setup_problem` (`src/memrepo/settings.rs`)
- [x] [auto] With a higher `schema`, `list`, `add_document`, and `search` fail, and `status` reports the problem with the fix (`tests/it_memory_repository.rs`)
- [x] [auto] `init` writes `schema = 1` (`tests/it_memory_repository.rs`)
- [x] [auto] The repair adds `schema = 1` after the header comment (or as the first line if there is none), keeps every other line and comment, does nothing twice, and leaves a file that is not TOML, a higher schema, and a bad schema alone (`src/memrepo/repairs.rs`)
- [x] [auto] A write on a file without `schema` makes the commit `waymark: repair "settings"` and then its own commit, with a notice. A read and a preview make no commit. A second write makes no repair commit (`tests/it_memory_repository.rs`)
- [x] [auto] A repair keeps a limit that the owner set, and its comment (`tests/it_memory_repository.rs`)
- [x] [auto] The file from `init` has no header comment that calls every key optional, says that `id` is required, and has a comment line above every key (`tests/it_memory_repository.rs`)
- [ ] [manual] A repository made by `init` opens in Obsidian as a vault. Steps: run `make sample-memory`, open the folder in Obsidian, create a link with the link dialog. Expected: the link is a relative Markdown link and `git status` shows only the new document.

## Notes

The manual item is part of the Obsidian check at the release (run `make manual-tests`).
