# Tests: Document history

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `history` lists the versions newest first with all fields | `history_lists_the_versions_newest_first` |
| `before` pages without repeats | `history_pages_with_before` |
| The history follows a move | `history_follows_a_move` |
| `show` with an old version returns the old text | `show_with_an_old_version_returns_the_old_text` |
| `show` with the current version equals a call without it | `show_with_the_current_version_is_the_current_document` |
| Unknown, hidden, and foreign versions fail | `history_and_show_refuse_unknown_hidden_and_foreign_versions` |
| The read budget counts the document once | `show_with_a_version_counts_the_document_once` |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary (`tests/it_document_history.rs`).
