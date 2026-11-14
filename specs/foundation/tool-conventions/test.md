# Tests: Tool conventions

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| Write tools on one document are `<verb>_document`, no old name is listed | `tool_names_follow_the_naming_scheme` |
| Every parameter has a description | `every_parameter_has_a_description` |
| Every description says what it returns, write tools say commit and push | `every_description_says_what_it_returns_and_write_tools_say_they_commit` |
| An old tool name runs the new tool with a notice | `an_old_tool_name_still_runs_with_a_notice` |
| An old name in a limit still switches the new tool off | `an_old_name_in_a_limit_still_blocks_the_new_tool` |
| `since` accepts an RFC 3339 time | `activity_since_accepts_a_date_and_an_rfc_3339_time` |
| The tool list stays below the budget | `tools_list_stays_under_the_budget` (`tests/it_mcp_server.rs`), `tool_descriptions_and_instructions_stay_small` (`tests/it_context_budget.rs`) |

## Test Cases

- [x] [auto] All cases of the table above, through the real binary (`tests/it_tool_conventions.rs`).
- [x] [auto] `canonical_tool` maps every old name and leaves other names alone (`src/config/limits.rs`).
- [x] [auto] `parse_when` accepts a date and an RFC 3339 time and refuses a future time and text (`src/tools/read_common.rs`).
- [ ] [manual] Start the server with an old MCP entry that has `--no-delete`. Expected: `delete_document` is not listed, and a call with the old name `delete` is denied.
