# Tests: Knowledge base repository

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| `init` result opens as an Obsidian vault with Markdown links and relative paths | `init_writes_obsidian_defaults` (auto), Obsidian check (manual) |
| Personal Obsidian files do not show in `git status` | `gitignore_hides_personal_obsidian_files` |
| `waymark.toml` without `id` gives `setup_problem` | `missing_id_is_setup_problem` |
| Stricter of shared and personal value applies | `stricter_value_applies` (also in `trust-model`) |
| A reorganization needs no `owners` update | `no_owners_field_written` |

## Test Cases

- [x] [auto] `init` writes every file of the layout and a valid UUID (`src/kb/layout.rs`)
- [x] [auto] `init` refuses to overwrite an existing file (`src/kb/layout.rs`)
- [x] [auto] `.obsidian/app.json` has `useMarkdownLinks: true` and `newLinkFormat: relative` (`src/kb/layout.rs`)
- [x] [auto] In a repository with `.obsidian/workspace.json` created, `git status` shows no change (`tests/it_knowledge_base_repo.rs`)
- [x] [auto] Missing `id`, a bad UUID, and a syntax error give `setup_problem` (`src/kb/settings.rs`)
- [x] [auto] Unknown keys give a notice and no error (`src/kb/settings.rs`)
- [x] [auto] The root is found for the layout `knowledge/.mkbase` and fails with `setup_problem` when absent (`src/kb/layout.rs`)
- [x] [auto] The files written by `init` pass the conformance check of `document-format` where they are documents (`tests/it_knowledge_base_repo.rs`)
- [ ] [manual] A repository made by `init` opens in Obsidian as a vault. Steps: run `make sample-kb`, open the folder in Obsidian, create a link with the link dialog. Expected: the link is a relative Markdown link and `git status` shows only the new document.

## Notes

The manual item is part of the Obsidian check at the release (see `docs/manual-test-checklist.md`).
