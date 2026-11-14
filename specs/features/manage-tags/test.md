# Tests: Manage tags

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `list_tags` counts documents, most used first, and pages | `list_tags_counts_documents_and_pages` |
| `rename_tag` changes all documents in one commit | `rename_tag_changes_all_documents_in_one_commit` |
| A merge keeps the new tag once | `rename_tag_merges_into_an_existing_tag` |
| An unknown tag is `not_found` | `rename_tag_refuses_an_unknown_tag` |
| A bad tag or the same tag is `invalid_arguments` | `rename_tag_refuses_bad_tags` |
| The `folders` limit | `tag_tools_respect_the_folders_limit` |
| `preview` writes nothing | `rename_tag_preview_writes_nothing` |
| Read-only, pause, denial, and `undo` | `rename_tag_follows_the_limits_and_undo_reverts_it` |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary (`tests/it_manage_tags.rs`).
- [x] [auto] The verb `rename-tag` reads back from a commit subject (`src/gitsync/message.rs`).
- [x] [auto] `check_tag` accepts normal tags and refuses `#x`, `a b`, and the empty text (`src/writes/mod.rs`).
