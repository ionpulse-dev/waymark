# Tasks: Join a memory repository

`requires`: `login-and-identity`, `agent-tool-setup`.

- [x] `src/join/url.rs`: parse the four URL forms, the repository name, the host, and the same-repository comparison. Unit tests.
- [x] `src/join/path.rs`: the default path and the checks of an existing path. Unit tests.
- [x] `gitsync::clone`: take a target folder and a time-out (change in `git-sync`).
- [x] `src/join/clone.rs`: clone to a temporary folder, rename, cleanup of old temporary folders.
- [x] `src/join/mod.rs`: `Startup`, `State`, the start thread, and `wait_ready`.
- [x] `mcp-server`: call `join::start` in `main.rs`, and use the state in `mcp::context` (all tools except `status` wait for it).
- [x] `status`: show the state (fields in `maintenance`).
- [x] `tests/it_join_memory_repository.rs`: the acceptance criteria.
- [x] `mcp::context`: the state `cloning` comes from a flag that is set only when a real clone is needed (bug: opening an existing clone showed `cloning`). Test A14.
- [x] Run the full suite.
