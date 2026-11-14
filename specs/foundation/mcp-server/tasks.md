# Tasks: MCP server

`requires`: `git-sync`, `trust-model`.

- [ ] Read the `rmcp` 3.x documentation and write the notes in `decisions.md`. Add `rmcp`, `tokio`, `clap`, `serde_json` to `Cargo.toml`.
- [ ] `src/mcp/error.rs`: `ToolError` and the error codes.
- [ ] `src/mcp/result.rs`: `ToolOutput` and its JSON form.
- [ ] `src/mcp/args.rs`: command line, environment variables, and the layer values.
- [ ] `src/mcp/context.rs`: `Context`.
- [ ] `src/mcp/tool.rs`: the `Tool` trait, the registry, listing, and call checks.
- [ ] `src/tools/status.rs`: the `status` tool.
- [ ] `src/mcp/mod.rs`: the `rmcp` handler with `initialize`, `tools/list`, `tools/call`, and instructions.
- [ ] `src/main.rs`: start, logging to stderr, first-start clone.
- [ ] Replace the stub binary of `integration-testing` with the real one.
- [ ] `tests/it_mcp_server.rs`: the acceptance criteria.
- [ ] A size test for `tools/list`.
- [ ] Run the full suite.
