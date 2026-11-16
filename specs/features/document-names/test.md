# Tests: Document names

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `name` sets the ID, the title stays | `a_name_sets_the_id` |
| A taken name fails and names the document | `a_taken_name_fails` |
| A bad name fails with `invalid_arguments` | `a_bad_name_fails` |
| Without `name`, the name comes from the title | `without_a_name_the_title_gives_the_name` |
| `add_document` rewrites short links in the same commit | `a_second_readme_rewrites_short_links` |
| `move` rewrites short links to a document that stays | `a_move_to_a_taken_file_name_rewrites_short_links` |
| `sync` blocks a near-duplicate title | `sync_blocks_a_near_duplicate_title` |
| `sync` blocks a file outside the folders limit | `sync_blocks_a_file_outside_the_folders` |
| `sync` blocks a new file when `add_document` is switched off | `sync_follows_switched_off_tools` |
| `sync` resets trust on a changed verified document | `sync_resets_trust_on_a_changed_verified_document` |
| `sync` sets a new file to unverified | `sync_sets_a_new_file_to_unverified` |
| One commit for each edit, with `Action:` | `sync_makes_one_commit_for_each_edit` |
| A rename by hand is a move with link fixes | `a_rename_by_hand_is_published_as_a_move` |
| A hand edit cannot set `verified` | `sync_does_not_let_a_hand_edit_verify` |
| Two conflicting edits keep both versions | `two_conflicting_hand_edits_keep_both_versions` |
| A hand rename is one `activity` entry, and `undo` takes back both commits | `a_hand_rename_is_one_activity_entry_and_undo_takes_back_both` |
| A hand rename stays one entry after a pull with rebase | `a_hand_rename_stays_one_entry_after_a_pull_with_rebase` |
| A delete waits while a blocked edit still links in its committed text | `a_delete_waits_while_a_blocked_edit_still_links_in_its_committed_text` |
| A preview with read-only access does not block edits as switched off | `a_read_only_preview_does_not_report_switched_off_tools` |
| `sync` asks for approval of `add_document` | `sync_asks_when_add_document_needs_approval` (`it_write_approval.rs`) |
| A write holds back an edit that needs approval | `a_write_holds_back_an_edit_that_needs_approval` (`it_write_approval.rs`) |
| The `sync` description says what it is for | `the_sync_description_names_manual_edits` |
| Both `AGENTS.md` suggestions hold the `sync` rule | `both_agents_md_suggestions_hold_the_sync_rule` (`it_agent_tool_docs.rs`) |

## Test Cases

- [ ] [auto] All cases of the table above, through the real binary (`tests/it_document_names.rs`, `tests/it_write_approval.rs`, `tests/it_agent_tool_docs.rs`).
- [ ] [auto] Unit tests of `check_name`, `Remap::wiki_target`, and `Remap::added`.
- [ ] [auto] The existing suites pass, including `it_manual_edits.rs` and the tool-list budget (`it_context_budget.rs`).
- [ ] [manual] Ask an agent (Claude Code) to "create a readme in the folder people". Expected: it calls `add_document` with `name: "readme"`, and it does not write a file or call `sync`.
- [ ] [manual] In Obsidian, rename a document that another document links to, then ask the agent to sync. Expected: one `moved` entry, and the link in the other document points to the new name.

## Notes

The manual items need a local install of this version and an agent tool with the waymark MCP entry (see `docs/local-install.md`).
