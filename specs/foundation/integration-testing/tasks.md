# Tasks: Integration testing

`requires`: `dev-environment`.

- [x] Add dev-dependencies to `Cargo.toml`.
- [x] `tests/common/git.rs`: helpers for `git` commands, commit lists, file reads, clean-tree checks.
- [x] `tests/common/world.rs`: `World` with the bare remote, two clones, authors, and isolation.
- [x] `tests/common/kb.rs`: builders for the knowledge base layout and documents.
- [x] `tests/common/mcp.rs`: `McpClient` with spawn, initialize, list_tools, call, elicitation hook, and timeouts.
- [x] `tests/common/mod.rs`: exports.
- [x] `tests/it_harness.rs`: tests of the harness (see `test.md`).
- [x] `Makefile`: `test-integration` and `test-all`.
- [x] Run the full suite.
