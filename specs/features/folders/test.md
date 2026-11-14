# Tests: Folders

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `folders` lists the tree down to depth 2 with `documents` and `total` | `folders_lists_the_tree_with_counts_down_to_depth_2` |
| `depth: 1` lists the top level and marks `has_more` | `depth_1_lists_the_top_level_and_marks_folders_with_more` |
| More than 200 folders are truncated | `more_than_200_folders_are_truncated` |
| The `folders` limit hides other folders | `the_folders_limit_hides_other_folders` |
| An unknown start folder is `not_found`; a bad depth is `invalid_arguments` | `a_start_folder_lists_below_it_and_an_unknown_one_is_not_found` |
| `add_empty_folder` writes a `.gitkeep` in one commit, the folder is listed with 0 documents, a document in it is counted, the folder stays after the last delete | `add_empty_folder_creates_a_gitkeep_in_one_commit` |
| An existing folder, a bad path, or a document ID is refused | `add_empty_folder_refuses_an_existing_folder_a_bad_path_and_a_document_id` |
| Limits, `preview`, read-only, and `paused` apply to `add_empty_folder` | `add_empty_folder_follows_the_limits_and_preview` |
| A `.gitkeep` is ignored by `lint`, `list`, and `stats` | `a_gitkeep_is_not_a_document` |
| `remove_folder` removes documents and `.gitkeep` files in one commit, and `undo` restores them | `remove_folder_removes_documents_and_gitkeeps_in_one_commit_and_undo_restores` |
| A wrong `expect_documents` fails with the real number; an empty folder goes with 0 | `remove_folder_needs_the_right_count_and_removes_an_empty_folder_with_0` |
| Links from outside block `remove_folder`; links inside do not | `remove_folder_refuses_links_from_outside_and_allows_links_inside` |
| The `folders` limit, `tools_denied`, and the protected names | `remove_folder_follows_the_limits_and_protects_the_root` |
| `move` carries the `.gitkeep` of a folder | `move_carries_the_gitkeep_of_a_folder` |
| The hints of the three tools | `the_folder_tools_have_the_right_hints` |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary and the `World` harness (`tests/it_folders.rs`).
- [x] [auto] `check_folder_path` accepts `ops/deploy/` and refuses empty, absolute, `..`, uppercase, space, leading `-`, empty parts, hidden names, and 9 levels (`src/writes/mod.rs`).
- [x] [auto] The verbs `add-folder` and `remove-folder` read back from a commit subject (`src/gitsync/message.rs`).
- [ ] [manual] `remove_folder` with a real agent tool: ask the agent to remove a folder, check that it calls `preview` first, passes the count, and that the agent tool asks before the call. Expected: the folder and its documents are gone in one commit, and `undo` brings them back.
