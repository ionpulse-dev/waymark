# Tests: Move documents

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| Links to 42 moved documents work | `folder_move_keeps_links` |
| `git log --follow` shows the old history | `history_follows_rename` |
| Commit 1 has renames only, commit 2 link rewrites only | `two_commits_pure_then_links` |
| `to` exists: `validation_failed` | `move_to_existing_fails` |
| `tools_denied = ["move"]` hides the tool | `move_hidden_when_denied` |
| `preview: true` changes nothing | `preview_changes_nothing` |
| Rename of one document | `rename_rewrites_links` |
| No inbound links: one commit | `no_links_one_commit` |
| `folders` limit: `access_denied` | `move_outside_folders` |

## Test Cases

- [ ] [auto] All criteria above, through the real binary and the `World` harness (`tests/it_move_documents.rs`)
- [ ] [auto] Commit 1 has only `R100` entries (`git diff-tree -M --name-status`) (`tests/it_move_documents.rs`)
- [ ] [auto] Commit 2 has the subject `waymark: update "<title>"` and the body line `Move: <hash of commit 1>` (`tests/it_move_documents.rs`)
- [ ] [auto] `updated` and `updated_by` change only in documents whose links changed (`tests/it_move_documents.rs`)
- [ ] [auto] A property wikilink, a body Markdown link, a short-name wikilink with alias, and a link in a code fence: the first three are rewritten, the last is not (`tests/it_move_documents.rs`)
- [ ] [auto] A move into itself, of the root, of `.mkbase/`, and with different kinds each fail with the right code (`tests/it_move_documents.rs`)
- [ ] [auto] A failed history check removes the local commits and pushes nothing (`tests/it_move_documents.rs`, with a fake that hides history)
- [ ] [auto] Both commits reach the remote with one push, and the second clone sees the new layout after a pull (`tests/it_move_documents.rs`)
- [ ] [auto] The same move twice gives `not_found` the second time, with no change (`tests/it_move_documents.rs`)
- [ ] [auto] Relative path computation (unit tests in `src/writes/moveplan.rs`)
- [ ] [manual] Links still work in Obsidian after a folder move. Steps: run `move` on a folder with 5 linked documents through a real agent tool, pull in a vault, click each link and open the graph. Expected: no unresolved link shows.

## Notes

The tests use the `World` harness. A test builds a knowledge base with a nested folder and cross links first.
