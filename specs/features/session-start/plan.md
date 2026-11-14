# Plan: Session start

## Overview

Two pure functions build the facts: one makes the `settings` JSON from `EffectiveSettings`, and one makes the instructions text from the binding, the settings, and the notices. `initialize` returns the text. `status` returns the same facts as fields.

## Components Affected

- `mcp` (`src/mcp/mod.rs`) — the `rmcp` handler sets `instructions` in the `initialize` result from `instructions::build`.
- `mcp` (`src/mcp/result.rs`) — builds the `settings` field of every result with `session::settings_json`, so all results and `status` share one shape.
- `mcp` (`src/mcp/context.rs`) — gives access to `binding`, and to the list of local notices.
- `tools` (`src/tools/status.rs`) — the tool of `mcp-server`. It returns `memory_repository`, `problem`, `settings`, and `notices` from `session::facts`. `maintenance` adds `checks` and the version.
- `config` — `EffectiveSettings` only. No change.

## New Files

- `src/mcp/session.rs` — `SessionFacts` (`memory_repository`, `problem`, `settings`, `notices`), `facts(&Context, &EffectiveSettings) -> SessionFacts`, `settings_json(&EffectiveSettings) -> Value`, and `local_notices(...)`.
- `src/mcp/instructions.rs` — `build(&SessionFacts) -> String`, the cut rules, and the `sanitize` function for values from files.
- `tests/it_session_start.rs` — end-to-end tests.

## Technical Approach

1. **Facts.** `session::facts` reads the `Binding` (or the `Problem`), the effective settings of this moment, and the local notices. It reads no network and runs no `git` command, except `git remote get-url origin`, which is local.
2. **Settings JSON.** `settings_json` writes the keys in the fixed order of `requirements.md`, with the types shown there.
3. **Notices.** `local_notices` returns the notices of `requirements.md` in this order: paused, no remote, unknown keys, found by folder. `ToolOutput` adds the notices of the call after them.
4. **Instructions.** `instructions::build` fills the fixed lines. `sanitize` cuts a value at the first line break and at 80 characters. The text is cut at 1200 characters as a last guard. With a `Problem`, the second form is used.
5. **Initialize.** The handler calls `facts` and `build` once when `initialize` arrives and stores nothing else. No network call is made.
6. **Status.** The tool maps `SessionFacts` to its result. `memory_repository.remote` uses `repo_url::redact` (see `project-binding`). With a `Problem`, the result is a normal result with `problem` and no error flag.
7. **Word list test.** A unit test builds the text for many setting combinations (a table) and fails if it has a forbidden word.
8. **Budget test.** A unit test checks the lengths of `requirements.md` (below 900 with defaults, at most 1200 in the worst case).

## Cross-Feature Dependencies

- `project-binding` — `Binding`, `Problem`, and `found_by`.
- `mcp-server` — the handler, `ToolOutput`, and the `status` tool file. Item 9 of its plan names the instructions. This feature defines their content.
- `trust-model` — the limits.
- `pause-and-resume` — the paused notice and `settings.paused`.
- `maintenance` — adds `checks` and the version to `status`.
- `memory-repository` — the unknown-keys notice.
- `git-sync` — the no-remote notice.

## Risks

- A client ignores the instructions. Mitigation: `status` and the settings in every result.
- A client truncates long instructions. Mitigation: a short text, and the budget test.
- A file value injects text. Mitigation: `sanitize` and the fixed template.

## Open Questions

None.
