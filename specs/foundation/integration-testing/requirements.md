---
type: foundation
status: implemented
created: 2026-11-14
updated: 2026-11-14
requires:
  - dev-environment
---

# Integration testing

## Technical Goal

Test the real `waymark` binary end to end, with no network and no credentials. Each test builds its own world: a temporary Git repository with a bare "remote", a second clone that acts as a second person, and a small MCP client that talks to the binary over stdio. Every feature spec uses this harness for its `[auto]` tests.

## Dependents

- Every feature spec and `mcp-server`, `git-sync`, `memory-repository`, `trust-model` for their end-to-end tests.
- `distribution` — the local Homebrew install test.

## Deliverables

- A test harness in `tests/common/` with these parts: a world builder, a second clone, an MCP stdio client, and assertion helpers for Git state.
- `make test-integration`.
- The rule that decides `[auto]` and `[manual]` for all specs (see below).

## Scope

**In scope:** local mode over stdio, on macOS, with `file://` remotes and the real `git` binary.

**Out of scope:** a real Git host, real credentials, a real agent tool, Obsidian, the HTTPS transport.

## Constraints

- Tests never touch the network, the Git config of the person, or the real home folder. Each test sets `HOME`, `XDG_CONFIG_HOME`, and `GIT_CONFIG_GLOBAL` to a temporary folder, and sets the Git author by environment variables.
- Tests are independent and can run in parallel. No test shares a folder with another.
- A test cleans up. A failing test keeps its folder and prints the path.
- A test runs the compiled binary (`CARGO_BIN_EXE_waymark`), not library functions.

### The world

A `World` has: a bare repository (the remote), a first clone (person A), a second clone (person B), each with its own author and its own `waymark` process on request, and a config home. The memory repository is initialized in the remote with the layout of `memory-repository`.

### MCP client

The client starts the binary with arguments, sends `initialize`, `tools/list`, and `tools/call` over stdio as JSON-RPC, and returns the structured result and the error flag. It lets a test also answer an elicitation request (see `write-approval`).

### `[auto]` or `[manual]`

A test is `[manual]` only if it needs a human in a real environment: Obsidian, a real agent tool, a real Git host with credentials, or the first install from the public tap. Everything else is `[auto]`, also tests that start the server and run many tool calls.

## Acceptance Criteria

- **Given** the harness, **When** a test builds a `World`, **Then** it has a remote and two clones with different authors and no access to the real home or Git config.
- **Given** a test that calls `tools/list`, **When** it runs, **Then** it gets the tool list of the real binary.
- **Given** two tests, **When** they run in parallel, **Then** they do not interfere.
- **Given** a failing test, **When** it ends, **Then** its folder remains and the path is printed.
- **Given** `make test-integration`, **When** it runs, **Then** it builds the binary and runs every test in `tests/`.
- **Given** a test with the person's real Git config present, **When** it runs, **Then** the real config has no effect.

## Open Questions

None.
