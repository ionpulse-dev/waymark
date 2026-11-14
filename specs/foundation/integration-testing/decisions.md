# Decisions: Integration testing

## A test client without the MCP library — 2026-11-14

**Decision:** The test client speaks newline-delimited JSON-RPC over stdio by itself. It does not use `rmcp`.

**Alternatives considered:** The client of `rmcp` (rejected: a bug in the library would hide in both sides).

**Reason:** An independent client checks the server against the protocol. Decided autonomously.

## `file://` remotes and bare repositories — 2026-11-14

**Decision:** The remote is a local bare repository. Tests need no network, no SSH, and no credentials.

**Reason:** Git behaves the same for push, pull, rebase, and conflicts. Decided autonomously.

## Isolation by environment variables — 2026-11-14

**Decision:** Each child process gets its own `HOME`, `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_SYSTEM=/dev/null`, and author variables.

**Reason:** The real config of the developer must never change a result. Decided autonomously.

## The test client speaks raw JSON-RPC and there is no stub binary — 2026-11-14

**Decision:** `tests/common/mcp.rs` starts the real binary (`CARGO_BIN_EXE_waymark`) and sends and reads newline-delimited JSON. A reader thread collects messages. A request from the server (for example a question to the person) is answered by an optional hook. `McpClient::for_person(world, person, Spawn)` sets `--path`, the author, a personal home folder `home-a` or `home-b` in the world, a system file path (`WAYMARK_SYSTEM_CONFIG`), and the Git isolation. `close()` ends stdin and waits for the process. There is no stub binary, because `mcp-server` was built before the first test that needs a client.

**Alternatives considered:** The `rmcp` client (a bug in the library could hide a bug in the server), and a stub binary (not needed any more).

**Reason:** Simple, and it tests the wire format that real agent tools see. Decided autonomously.
