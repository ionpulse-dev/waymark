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
