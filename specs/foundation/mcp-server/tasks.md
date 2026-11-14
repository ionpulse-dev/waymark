# Tasks: MCP server

`requires`: `git-sync`, `trust-model`.

- [x] Read the `rmcp` 3.x documentation and write the notes in `decisions.md`. Add `rmcp`, `tokio`, `clap`, `serde_json` to `Cargo.toml`.
- [x] `src/mcp/error.rs`: `ToolError` and the error codes.
- [x] `src/mcp/result.rs`: `ToolOutput` and its JSON form.
- [x] `src/mcp/args.rs`: command line, environment variables, and the layer values.
- [x] `src/mcp/context.rs`: `Context`.
- [x] `src/mcp/tool.rs`: the `Tool` trait, the registry, listing, and call checks.
- [x] `src/tools/status.rs`: the `status` tool.
- [x] `src/mcp/mod.rs`: the `rmcp` handler with `initialize`, `tools/list`, `tools/call`, and instructions.
- [x] `src/main.rs`: start, logging to stderr, first-start clone.
- [x] Replace the stub binary of `integration-testing` with the real one.
- [x] `tests/it_mcp_server.rs`: the acceptance criteria.
- [x] A size test for `tools/list`.
- [x] Run the full suite.
- [x] Updates of the MCP entry: lenient parser, the state "needs an update" in `initialize`, `tools/list`, the tool errors, and `status` (2026-11-14).
