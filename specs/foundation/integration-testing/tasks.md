# Tasks: Integration testing

`requires`: `dev-environment`.

- [ ] Add dev-dependencies to `Cargo.toml`.
- [ ] `tests/common/git.rs`: helpers for `git` commands, commit lists, file reads, clean-tree checks.
- [ ] `tests/common/world.rs`: `World` with the bare remote, two clones, authors, and isolation.
- [ ] `tests/common/kb.rs`: builders for the knowledge base layout and documents.
- [ ] `tests/common/mcp.rs`: `McpClient` with spawn, initialize, list_tools, call, elicitation hook, and timeouts.
- [ ] `tests/common/mod.rs`: exports.
- [ ] `tests/it_harness.rs`: tests of the harness (see `test.md`).
- [ ] `Makefile`: `test-integration` and `test-all`.
- [ ] Run the full suite.
