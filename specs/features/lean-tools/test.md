# Tests: Lean tools

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| `list` is not listed and `search` has no required parameter | `list_is_gone_and_search_needs_no_query` |
| `search` without a query lists by ID at level 1 with filters | `search_without_a_query_lists_by_id` |
| The default status is `active` in both modes | `search_without_a_query_hides_deprecated_documents` |
| A call named `list` still works with a notice | `the_old_name_list_runs_search_with_a_notice` |
| `tools_denied = ["list"]` switches off `search` | `list_in_a_limit_switches_off_search` |
| The old names of 0.4.0 are unknown tools | `the_old_names_of_0_4_0_are_unknown_tools` |
| The old names in limits still count | `an_old_name_in_a_limit_still_blocks_the_new_tool` (`it_tool_conventions.rs`) |
| `replace` changes one passage only | `replace_changes_one_passage` |
| `replace` fails on no match or many matches | `replace_needs_exactly_one_match` |
| `append` adds text at the end | `append_adds_text_at_the_end` |
| Conflicting parameters fail | `conflicting_body_and_tag_parameters_fail` |
| `add_tags` and `remove_tags` change single tags | `add_and_remove_tags` |
| `replace` resets trust and checks new links | `replace_resets_trust_and_checks_links` |
| `show` returns the deprecation reason | `show_returns_the_deprecation_reason` |
| A restored document has no reason | `a_restored_document_has_no_deprecation_reason` |
| `updates` reads a long window with `offset` | `updates_pages_with_offset` |
| `offset` without `since` fails | `updates_offset_needs_since` |
| No hint names a retired tool | `no_hint_names_a_retired_tool` |
| The tool list is smaller and in budget | `the_tool_list_is_smaller_than_in_0_4_0` (`it_context_budget.rs`) |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary (`tests/it_lean_tools.rs`).
- [x] [auto] The existing suites pass after the move from `list` to `search`.
