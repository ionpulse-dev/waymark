# Plan: Write approval

## Overview

The write flow of `git-sync` gets an `Approver`. The tool builds an `ApprovalRequest` (plain data). `gitsync::Repo::write` decides if approval applies and calls the `Approver`. The implementation in `mcp` renders the request as text and sends an MCP elicitation through `rmcp`.

## Components Affected

- `gitsync` (`src/gitsync/write.rs`, `src/gitsync/mod.rs`) — `Repo::write` takes `&dyn Approver` and an `ApprovalRequest`, and runs step 6 of the write flow. It reads the limits again after a yes.
- `config` (`src/config/mod.rs`) — already has `approval.required_for` (union) and `approval.via` (`console` wins). A helper `EffectiveSettings::approval_required(tool, kind) -> bool` is added.
- `mcp` (`src/mcp/mod.rs`, `src/mcp/context.rs`) — keeps the client capabilities from `initialize` and builds an `ElicitationApprover` for each call. `Cargo.toml` enables the `rmcp` feature `elicitation`.
- Every write tool (`write-documents`, `move-documents`, and the others) — passes an `ApprovalRequest` to `Repo::write`. The tool fills the fields of the table in `requirements.md`.
- `tests/common/mcp.rs` — uses the elicitation hook of the harness.

## New Files

- `src/gitsync/approval.rs` — `Approver` trait (`ask(&ApprovalRequest) -> Answer`), `ApprovalRequest` (`tool`, `kb_name`, `action`, `fields`, `body`, `summary_lines`), and `Answer` (`Yes`, `No { reason }`, `Unavailable { reason }`).
- `src/mcp/approval.rs` — `ElicitationApprover`: capability check, the `rmcp` request, the time-out, progress messages, and the mapping of answers.
- `src/mcp/approval_text.rs` — pure functions that render an `ApprovalRequest` into the `message` text, with the cut at 20000 characters. No `rmcp` types.
- `tests/it_write_approval.rs` — end-to-end tests.
- `tests/it_elicitation_spike.rs` — the spike of task 1. It stays as the smoke test that the `rmcp` version still sends elicitation.

## Technical Approach

1. **Spike first.** Enable the `rmcp` feature `elicitation`. Build a minimal server and call `Peer::create_elicitation` with a hand-written form. Check with the test client: the request arrives, `accept`, `decline`, and `cancel` come back as the three `ElicitationAction` values, a time-out sends `notifications/cancelled`, and the capability of the client is readable from `peer_info()`. Write the findings into `decisions.md`.
2. **Decide if approval applies** in `Repo::write`: `settings.approval_required(tool, kind)` is true if `tool` is in `required_for`, or `write` is in `required_for` and the kind is write or destructive. It is false for `preview: true` and for a write that changes no file.
3. **The channel.** If `approval.via` is `console`, return `Unavailable` without a request. Else ask the `Approver`.
4. **`ElicitationApprover::ask`.**
   - Capability check (see `decisions.md`). No capability: `Unavailable`.
   - Build `params` with the text of `approval_text.rs` and the schema of `requirements.md`.
   - Call `create_elicitation_with_timeout` with the time-out (120 s, or the environment variable). The call runs on the tokio handle from the blocking thread (`Handle::block_on`), because tool handlers are synchronous.
   - If the caller sent a progress token, a second task sends `notifications/progress` at the interval of `decisions.md` until the answer arrives.
   - Map the answer: `Accept` with `content.approve == true` is `Yes`. Every other action is `No` with its message. A time-out is `No` and the request is cancelled. A service error is `Unavailable`.
   - All `rmcp` types stay in `src/mcp/mod.rs` and `src/mcp/approval.rs`.
5. **After a yes**, `Repo::write` computes the effective settings again and calls `write_block` (see `pause-and-resume`). Then it writes, commits, and pushes.
6. **Result.** The tool adds `approval: "approved"` to `data` after a yes. With `preview: true` and an applicable rule, it adds `approval: "required"`.
7. **The lock** stays held during the wait (step 2 of the write flow). A second process fails after 30 seconds with `sync_failed`.
8. **Render rules** (`approval_text.rs`): the first line is `waymark asks to <verb> a document in "<memory repository name>".` for the document tools. The other tools use the one-line summary of their `ApprovalRequest`. The lines `Tool`, `Folder`, `Title`, `Document`, and `Author` come next. The body follows after `--- Body ---`. A cut message ends with the notice of `requirements.md`.

## Cross-Feature Dependencies

- `git-sync` — owns the write flow. This feature adds the `Approver` parameter and the details of step 6. Both specs are `planned`, so the change goes into the implementation of `git-sync`. See the report.
- `trust-model` — owns the keys and the merge rule of `approval`. Unchanged.
- `pause-and-resume` — `write_block` after the answer.
- `write-documents`, `move-documents`, `activity-and-undo`, `verification`, `manual-edits` — each write tool builds its `ApprovalRequest`. Those specs add the call in their own tasks.
- `integration-testing` — the client hook for elicitation exists there. It must be able to announce the `elicitation` capability (or not), answer with a chosen action and content, delay the answer, and record the request.

## Risks

- `rmcp` 3.x elicitation is incomplete or differs. Mitigation: the spike, and the fallback of raw JSON-RPC in `decisions.md`.
- A client does not show the form, or auto-answers. Mitigation: fail closed, the explicit boolean, and the manual tests.
- A long wait is cut by the client. Mitigation: progress messages, and the manual test M6.
- The lock is held during the wait. Mitigation: a second process fails clearly. A later option is to ask before the lock (see `todo.md`).

## Open Questions

None.
