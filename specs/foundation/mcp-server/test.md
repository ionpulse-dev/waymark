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

- [ ] [auto] All eight criteria above (`tests/it_mcp_server.rs`)
- [ ] [auto] `read-only` access hides write tools and a direct call gives `access_denied` (`tests/it_mcp_server.rs`)
- [ ] [auto] A call with a missing argument gives `invalid_arguments` with a `next` sentence (`tests/it_mcp_server.rs`)
- [ ] [auto] Nothing but protocol messages goes to stdout (`tests/it_mcp_server.rs`)
- [ ] [auto] `--version` and `--help` work and no other command exists (`tests/it_mcp_server.rs`)
- [ ] [auto] The size of `tools/list` stays under the budget (`tests/it_mcp_server.rs`)
- [ ] [auto] The release binary links only to system libraries (`otool -L` check in `scripts/tests/`)
- [ ] [auto] `status` shows `setup_problem` details for a missing `git` and for a path that is not a knowledge base (`tests/it_mcp_server.rs`)
- [ ] [manual] A real agent tool lists the tools. Steps: add the MCP entry from the README to Claude Code, GitHub Copilot, and OpenAI Codex, start a session, ask the agent to call `status`. Expected: the tool list shows and `status` returns the knowledge base name.

## Notes

The in-memory client of the original spec is replaced by the stdio client of `integration-testing`. The HTTPS part of the conformance criterion is out of scope.
