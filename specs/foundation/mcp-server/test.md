# Tests: MCP server

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| `tools/list` holds allowed tools and no control tool | `lists_allowed_tools_only` |
| Every result has `message` and `settings` | `results_have_message_and_settings` |
| A tool that changes limits is unknown | `control_tool_is_unknown` |
| The same call twice leaves the same state | `calls_are_idempotent` |
| The fixture suite matches on a new build | `conformance_fixtures` |
| The release binary runs with only `git` | `binary_needs_only_git` |
| A rejected call has `code` and `next` | `errors_have_code_and_next` |
| `--repo` with a new URL clones once and reuses | `clones_once` |

## Test Cases

- [x] [auto] All eight criteria above (`tests/it_mcp_server.rs`)
- [x] [auto] `read-only` access hides write tools and a direct call gives `access_denied` (`tests/it_mcp_server.rs`)
- [x] [auto] A call with a missing argument gives `invalid_arguments` with a `next` sentence (`tests/it_mcp_server.rs`)
- [x] [auto] Nothing but protocol messages goes to stdout (`tests/it_mcp_server.rs`)
- [x] [auto] `--version` and `--help` work and no other command exists (`tests/it_mcp_server.rs`)
- [x] [auto] The size of `tools/list` stays under the budget (`tests/it_mcp_server.rs`)
- [x] [auto] The release binary links only to system libraries (`otool -L` check in `scripts/tests/`)
- [x] [auto] `status` shows `setup_problem` details for a missing `git` and for a path that is not a memory repository (`tests/it_mcp_server.rs`)
- [x] [auto] The lenient parser: known arguments make no issue, an unknown argument (`--x v`, `--x=v`, `--x`) blocks and names the fix without the bad argument, a deprecated argument is a notice, a wrong value blocks and guesses no memory repository (`src/mcp/args.rs`)
- [x] [auto] An unknown argument and a wrong value: `instructions`, `tools/list` (only `status`, with the problem in its description), the error of every other tool, and the failing check `entry` of `status`. A higher schema does the same. A healthy setup shows no warning (`tests/it_updates_needed.rs`)
- [ ] [manual] An agent tool shows the warning. Steps: start a session with an unknown argument in the entry (for example `--future-flag 1`) in Claude Code, GitHub Copilot, and OpenAI Codex. Expected: the agent tells the person that waymark needs an update, with the fix. Note which of the three channels each tool shows (`instructions`, the description of `status`, the error).
- [ ] [manual] A real agent tool lists the tools. Steps: add the MCP entry from the README to Claude Code, GitHub Copilot, and OpenAI Codex, start a session, ask the agent to call `status`. Expected: the tool list shows and `status` returns the memory repository name.

## Notes

The in-memory client of the original spec is replaced by the stdio client of `integration-testing`. The HTTPS part of the conformance criterion is out of scope.
