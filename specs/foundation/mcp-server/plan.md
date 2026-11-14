# Plan: MCP server

## Overview

The `waymark` binary parses its arguments, builds a `Context` (settings, repository, knowledge base), and serves MCP over stdio with `rmcp`. A small `Tool` trait and a registry hold every tool. Feature specs add one file each under `src/tools/`. All logic sits in the library crate. `main.rs` only starts the server.

## Components Affected

- Every feature spec adds a tool file and one line in the registry.
- `trust-model` (`config`), `knowledge-base-repo` (`kb`), `git-sync` (`gitsync`), and `document-format` (`format`) are used from here.

## New Files

- `src/main.rs` — argument parsing (`clap`), logging to stderr, start of the server.
- `src/lib.rs` — module declarations (extends the file from `dev-environment`).
- `src/mcp/mod.rs` — the `rmcp` server handler: `initialize`, `tools/list`, `tools/call`.
- `src/mcp/tool.rs` — the `Tool` trait, `ToolKind`, and the registry.
- `src/mcp/result.rs` — `ToolOutput` (`message`, `notices`, `settings`, `data`) and its JSON form.
- `src/mcp/error.rs` — `ToolError` with the `code`, `message`, and `next`, and the mapping to the MCP error result.
- `src/mcp/context.rs` — `Context`: effective settings, repository, knowledge base, session state (read budget).
- `src/mcp/args.rs` — the command line and environment variables of the requirements.
- `src/tools/mod.rs` — the registry list.
- `src/tools/status.rs` — the `status` tool (specified in `maintenance` and `session-start`).
- `tests/it_mcp_server.rs` — end-to-end tests.
- `tests/fixtures/` — fixture repositories and expected results for the conformance suite.

## Technical Approach

1. **Arguments.** `clap` with the arguments and variables of the requirements. Any limit argument maps to a layer value (`config::layers`). `--version` and `--help` are the only other commands.
2. **Server.** `rmcp` over stdio on a `tokio` runtime. Tool handlers are synchronous and run in `spawn_blocking`, because `git` calls block. One call runs at a time per process (the write lock is also in `gitsync`).
3. **Tool trait.** `name()`, `description()`, `kind()` (read, write, destructive), `input_schema()` (JSON Schema, hand-written with `serde_json::json!` to keep descriptions short), and `call(&Context, Value) -> Result<ToolOutput, ToolError>`.
4. **Listing.** `tools/list` returns the tools that `EffectiveSettings` allows: write tools are hidden when `access` is `read-only` or `paused`, and `tools_denied` hides named tools. `verify`, `review`, and `reject` appear only when `verification` is on. The kind becomes the MCP annotations `readOnlyHint` and `destructiveHint`.
5. **Calls.** Before a handler runs, the registry checks that the tool is listed. A call of a hidden tool gives `access_denied`. An unknown name gives the MCP "unknown tool" error. Arguments are checked against the schema. A failure is `invalid_arguments` with a `next` sentence.
6. **Result.** Success returns structured content `{ message, notices, settings, ...data }` and a text copy of `message`. Failure sets `isError` with `{ code, message, next }`.
7. **Context per call.** Settings are loaded at every call (see `trust-model`). The session state (read budget counters) lives in `Context` for the process.
8. **First start.** If `--repo` is a URL and the path does not exist, clone it (see `git-sync`). If the path exists and is not a repository, `setup_problem`.
9. **Instructions.** The server `instructions` field holds a short, neutral text: what waymark is and that results carry `settings` and `notices`. No behavior advice.
10. **Logging.** To stderr only, never to stdout, because stdout is the protocol. Default level `warn`.
11. **Status tool.** Returns the state of the setup, the knowledge base name and id, the effective limits, the Git version check, and open notices. It is the first tool, so every later spec can be tested through the real server.

## Dependencies on Other Foundation Items

- `git-sync` and `trust-model` (listed in `requires`), plus `knowledge-base-repo` and `document-format` through them.
- `integration-testing` — the harness.

## Risks

- `rmcp` 3.x differs from 0.x examples that I know. Mitigation: read the crate documentation at the start of the task and keep all `rmcp` code in `src/mcp/mod.rs`.
- Elicitation support in `rmcp` may be incomplete. Mitigation: `write-approval` has the first task "check elicitation in `rmcp`", with a fallback to raw JSON-RPC for that one request.
- Context cost of tool descriptions. Mitigation: a test that measures the size of `tools/list` and fails above a budget (default 6000 characters for the descriptions, recorded in `decisions.md`).

## Open Questions

None.
