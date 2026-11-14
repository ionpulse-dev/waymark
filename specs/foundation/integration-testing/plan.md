# Plan: Integration testing

## Overview

Plain Rust integration tests in `tests/`, with a shared module `tests/common/`. The MCP client is a thin JSON-RPC client over a child process, written without the MCP library, so a bug in the library cannot hide in the test client.

## Components Affected

- `Cargo.toml` — dev-dependencies: `tempfile`, `serde_json`, `assert_cmd` (for the binary path), `regex`.
- `Makefile` — `test-integration` runs `cargo test --test '*'` for the files named `it_*.rs`.

## New Files

- `tests/common/mod.rs` — exports the parts below.
- `tests/common/world.rs` — `World`: remote, clones, authors, environment isolation.
- `tests/common/git.rs` — helpers: run `git` in a clone, list commits, read a file at a commit, check the working tree is clean.
- `tests/common/mcp.rs` — `McpClient`: spawn, `initialize`, `list_tools`, `call(tool, args)`, and a hook to answer elicitation.
- `tests/common/memrepo.rs` — builders for a memory repository layout and for documents.
- `tests/it_harness.rs` — tests of the harness itself.

## Technical Approach

1. `World::new()` creates a temp root, `git init --bare remote.git`, clones to `a/` and `b/`, writes a minimal memory repository (marker folder, `settings.toml`, a few documents) in `a/`, commits, and pushes.
2. Environment isolation: `HOME`, `XDG_CONFIG_HOME`, `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_SYSTEM=/dev/null`, `GIT_AUTHOR_*` and `GIT_COMMITTER_*` are set for every child process.
3. `McpClient::spawn(world, person, args)` starts `waymark --repo <path> --path <clone> ...` and speaks newline-delimited JSON-RPC on stdio. A read thread collects messages. `call` waits for the response with a timeout.
4. A test keeps the temp folder when it fails (a guard object checks `std::thread::panicking()`).
5. Every feature adds its own `tests/it_<feature>.rs`.

## Dependencies on Other Foundation Items

- `dev-environment` — the Makefile target and the toolchain.
- The binary under test comes from `mcp-server`. The harness itself can be built and tested before it, with a stub binary in `tests/common/stub.rs`.

## Risks

- A slow test suite. Mitigation: one process per test, small repositories, parallel runs.
- Differences between the MCP library and the client. Mitigation: the client follows the MCP specification for stdio (newline-delimited JSON-RPC), and conformance tests check both sides.

## Open Questions

None.
