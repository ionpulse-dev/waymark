# Tests: Search filters and paging

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `search` pages with `offset` and `next_offset`, no repeats | `search_pages_with_offset_and_next_offset` |
| `author` filters `list` and `search` by name part and email part | `list_and_search_filter_by_author` |
| `updated_after` and `updated_before` filter by range, both ends included | `list_and_search_filter_by_the_updated_range` |
| `disputed` filters `search` | `search_filters_disputed_documents` |
| A wrong date, a negative `offset`, or a reversed range is `invalid_arguments` | `bad_filter_arguments_are_invalid_arguments` |
| The `folders` limit still hides documents | `filters_never_reveal_documents_outside_the_folders_limit` |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary (`tests/it_search_filters.rs`).
- [x] [auto] `Filter` matches author, range, and disputed state (`src/tools/read_common.rs`).
