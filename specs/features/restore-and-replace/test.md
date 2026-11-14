# Tests: Restore and replace documents

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| A deprecated document becomes active in one commit | `restore_makes_a_deprecated_document_active_in_one_commit` |
| An active document is refused; an old base is `stale_base` | `restore_refuses_an_active_document_and_an_old_base` |
| A notice names an active document that replaces it | `restore_names_the_active_document_that_replaces_it` |
| Verification resets the trust | `restore_resets_the_trust_with_verification` |
| `update_document` with `supersedes` links and deprecates | `update_document_supersedes_links_and_deprecates_the_old_one` |
| Itself, a missing document, and a loop are refused | `update_document_refuses_itself_a_missing_document_and_a_loop` |
| Limits, `preview`, and `undo` | `restore_follows_the_limits_and_undo_reverts_it` |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary (`tests/it_restore_and_replace.rs`).
- [x] [auto] The verb `restore` reads back from a commit subject (`src/gitsync/message.rs`).
