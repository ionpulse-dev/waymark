# Tasks: Join a knowledge base

`requires`: `login-and-identity`, `agent-tool-setup`.

- [ ] `src/join/url.rs`: parse the four URL forms, the repository name, the host, and the same-repository comparison. Unit tests.
- [ ] `src/join/path.rs`: the default path and the checks of an existing path. Unit tests.
- [ ] `gitsync::clone`: take a target folder and a time-out (change in `git-sync`).
- [ ] `src/join/clone.rs`: clone to a temporary folder, rename, cleanup of old temporary folders.
- [ ] `src/join/mod.rs`: `Startup`, `State`, the start thread, and `wait_ready`.
- [ ] `mcp-server`: call `join::start` in `main.rs`, and use the state in `mcp::context` (all tools except `status` wait for it).
- [ ] `status`: show the state (fields in `maintenance`).
- [ ] `tests/it_join_knowledge_base.rs`: the acceptance criteria.
- [ ] Run the full suite.
