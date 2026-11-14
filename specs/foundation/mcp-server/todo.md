# Todo: MCP server

## Bugs

None.

## Deferred

- The HTTPS transport (server mode).
- Documents as MCP resources.
- Merging `list` and `stats` into `search`, if the context budget needs it.
- An embedded Git library instead of the `git` binary.

## Open Questions

None.

## Notes from the implementation (2026-11-14)

- The `status` tool is the first version: checks `git`, `settings`, `clone`, `knowledge_base`. `maintenance` adds the rest of its list.
- A test of the 6000 and 20000 character budget exists. It has only the `status` tool today. It gets its real value when the features add tools.
- `tests/it_mcp_server.rs` has no test for a missing `git` binary, because the test would have to hide `git` from the `PATH` of a child process. `status` has the code path and a unit test of `check_git` exists. Add the integration test in phase G (failure behavior).
