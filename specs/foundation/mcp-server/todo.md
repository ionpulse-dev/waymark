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

- The `status` tool is the first version: checks `git`, `settings`, `clone`, `memory_repository`. `maintenance` adds the rest of its list.
- A test of the 9000 and 28000 character budget exists (`tests/it_mcp_server.rs`, `tests/it_context_budget.rs`). It covers all tools.
- `tests/it_mcp_server.rs` has no test for a missing `git` binary, because the test would have to hide `git` from the `PATH` of a child process. `status` has the code path and a unit test of `check_git` exists. Add the integration test in phase G (failure behavior).
- Settings are re-read from disk several times per call (`kb_if_ready` in `src/mcp/tool.rs` and `src/notices/mod.rs`). Read them once per call (code review of 2026-11-14, skipped).
- `tests/it_context_budget.rs`: the search-cap test returns only 272 bytes, so it does not test the cap. Give it a memory repository with many large documents. The missing-`git` test now exists in `tests/it_failures.rs`.
