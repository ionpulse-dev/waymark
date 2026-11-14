# Plan: Pause and resume

## Overview

Pause is the limit `paused` of the `config` module. This feature adds one small function that decides whether a write may run, and uses it in three places: the tool list, the tool call, and the write flow. It adds no tool.

## Components Affected

- `config` (`src/config/limits.rs`, `src/config/layers.rs`) — already load `paused` from every layer and merge it ("true if any layer says so"). No change, except a `--paused` argument in `src/mcp/args.rs`.
- `mcp` (`src/mcp/tool.rs`) — the registry hides write tools while paused, and a call of a write tool gets the right error.
- `mcp` (`src/mcp/result.rs`) — adds the notice "Writes are paused." to every result while paused.
- `mcp` (`src/mcp/mod.rs`) — sends `notifications/tools/list_changed` when the visible tool list changes.
- `gitsync` (`src/gitsync/write.rs`) — step 1 of the write flow calls the same function. `write-approval` adds the second call after the answer.
- `README.md` — the section "Pause writes" with the exact steps of `requirements.md`.

## New Files

- `src/config/pause.rs` — `WriteBlock` (`ReadOnly`, `Paused`), `write_block(&EffectiveSettings) -> Option<WriteBlock>` (read-only first, then paused), and the texts: the notice, the message, and the `next` sentence of `paused`.
- `tests/it_pause_and_resume.rs` — end-to-end tests with the real binary.

## Technical Approach

1. `write_block` returns `Some(ReadOnly)` if `access` is `read-only`, else `Some(Paused)` if `paused` is true, else `None`. It is a pure function with unit tests.
2. `src/mcp/error.rs` maps `ReadOnly` to `access_denied` and `Paused` to `paused`, with the texts of `pause.rs`.
3. `tools/list`: the registry hides tools of kind write and destructive if `write_block` is `Some`. Settings come from `EffectiveSettings` computed for this call.
4. `tools/call`: for a tool of kind write or destructive, the registry calls `write_block` before the arguments are checked. This also covers a hidden tool, so the answer is `paused` and not "unknown tool". The `status` tool and all read tools never call it.
5. After each call, the handler compares the visible tool names with the list sent last. If they differ, it sends `notifications/tools/list_changed`. The capability `tools.listChanged` is `true` in `initialize`.
6. `ToolOutput` builds `notices`: if `settings.paused` is true, it adds "Writes are paused." as the first notice.
7. `gitsync::Repo::write` step 1 uses `write_block`, so a write called from library code (not through the registry) is checked too.
8. A file that cannot be parsed gives `setup_problem` before any of this (see `trust-model`).
9. `--paused` in `args.rs` is a layer value of the MCP entry. It is read once at start.

## Cross-Feature Dependencies

- `trust-model` — owns `paused` and the merge. This feature adds no merge rule.
- `mcp-server` — owns the registry. This feature changes the listing and call checks in `src/mcp/tool.rs`. Both specs are `planned`, so the change goes into the implementation of `mcp-server` (see the task order).
- `git-sync` — owns the write flow. Same rule.
- `write-documents` — the end-to-end tests use `add_document`.
- `write-approval` — the recheck after the answer.

## Risks

- A client caches the tool list and ignores `list_changed`. Mitigation: a call still returns `paused`, so the write never runs.
- The person adds a second `[limits]` section and the file breaks. Mitigation: the README steps, and `setup_problem` names the file.

## Open Questions

None.
