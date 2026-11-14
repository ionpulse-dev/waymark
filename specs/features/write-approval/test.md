# Tests: Write approval

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_The tests use the elicitation hook of the harness of `integration-testing`. The hook can announce or hide the capability, answer with an action and content, delay the answer, and record every request. The tests set `approval.required_for` in the shared file or the personal file of the `World`. The time-out tests set `WAYMARK_APPROVAL_TIMEOUT_SECONDS`._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** `required_for = ["delete"]`, **When** `delete` runs, **Then** the client gets a request with the document and the question before the commit | `delete_asks_before_commit` |
| **Given** the person says yes, **When** the answer arrives, **Then** the write continues and `approval: "approved"` is in the result | `yes_continues` |
| **Given** the person says no, **When** the answer arrives, **Then** `approval_denied` and nothing is written | `no_denies` |
| **Given** a client without elicitation, **When** approval is required, **Then** `approval_unavailable` and nothing is written | `no_capability_is_unavailable` |
| **Given** no answer for 2 minutes, **When** the time ends, **Then** `approval_denied` | `timeout_denies` |
| **Given** `add` is not in `required_for`, **When** it runs, **Then** no request opens | `unlisted_tool_never_asks` |
| **Given** `required_for = ["write"]`, **When** any write tool runs, **Then** a request opens | `write_covers_all_write_tools` |
| **Given** `preview: true`, **When** a write tool runs, **Then** no request opens and the result says `approval: "required"` | `preview_never_asks` |
| **Given** `accept` with `approve` false or missing, **When** the answer arrives, **Then** `approval_denied` | `accept_without_approve_denies` |
| **Given** a pause set while the request waits, **When** the person says yes, **Then** `paused` and nothing written | `pause_during_approval` |
| **Given** `via = "console"`, **When** a write needs approval, **Then** `approval_unavailable` and no request | `console_is_unavailable_in_local_mode` |
| **Given** a long body, **When** the request is built, **Then** the message is cut and says how much is missing | `long_body_is_cut` |

## Test Cases

- [x] [auto] The spike: the server sends a form request and reads `accept`, `decline`, and `cancel` (`tests/it_elicitation_spike.rs`)
- [x] [auto] The request has the `message` lines of `requirements.md` and a `requestedSchema` with one required boolean `approve` (`tests/it_write_approval.rs`)
- [x] [auto] `decline` gives `approval_denied` with the message "The person said no." The clone is clean and the remote has no new commit (`tests/it_write_approval.rs`)
- [x] [auto] `cancel` gives `approval_denied` with the message "The person closed the question." (`tests/it_write_approval.rs`)
- [x] [auto] `accept` with `{ "approve": false }`, with `{}`, and with `{ "approve": "yes" }` each give `approval_denied` (`tests/it_write_approval.rs`)
- [x] [auto] A client that announces `elicitation` with only `url` mode gets `approval_unavailable`, and no request is sent (`tests/it_write_approval.rs`)
- [x] [auto] A client that announces `elicitation: {}` gets the request (`tests/it_write_approval.rs`)
- [x] [auto] A client that answers the request with a JSON-RPC error gives `approval_unavailable` (`tests/it_write_approval.rs`)
- [x] [auto] With the time-out at 2 seconds and a delayed answer, the write fails with `approval_denied`, the client got `notifications/cancelled`, and a following write works because the lock is free (`tests/it_write_approval.rs`)
- [x] [auto] With a progress token and a 8 second time-out, the client receives `notifications/progress` while it waits (`tests/it_write_approval.rs`)
- [x] [auto] A read tool and `status` never send a request, also with `required_for = ["write"]` (`tests/it_write_approval.rs`)
- [x] [auto] Two writes in a row send two requests (`tests/it_write_approval.rs`)
- [x] [auto] The hook writes `paused = true` before it answers yes. The write fails with `paused`, with no new commit. With `access = "read-only"` set instead, the code is `access_denied` (`tests/it_write_approval.rs`)
- [x] [auto] `required_for` from the shared file and from the personal file are both applied (union) (`tests/it_write_approval.rs`)
- [x] [auto] A second process that wants the lock during the wait fails with `sync_failed` after the lock wait. The test sets the wait small (`a_second_lock_waits_and_then_fails` in `src/gitsync/lock.rs`)
- [x] [auto] Render tests for `add`, `update_document`, `delete`, `move` (synthetic request), `undo`, and `sync`, and the cut at 20000 characters (inline in `src/mcp/approval_text.rs`)
- [x] [auto] Unit tests of `approval_required` (inline in `src/config/mod.rs`)
- [ ] [manual] M1, Claude Code CLI shows the request. Steps: (1) Set `[approval] required_for = ["add"]` in `~/.config/waymark/config.toml`. (2) Start Claude Code in a project that has the MCP entry. (3) Ask the agent to add a document. Expected: Claude Code shows the request with the memory repository, folder, title, and body, and a box "Approve this write". After Accept with the box ticked, the document is committed. After Decline, the agent reports `approval_denied` and nothing is committed.
- [ ] [manual] M2, Claude Code in a mode that skips permission prompts. Steps: as M1, but start Claude Code with the permission mode that skips prompts. Expected: record if the request still appears. If it does not, the client auto-answers, and the README says so.
- [ ] [manual] M3, Claude Code in VS Code and Claude Desktop. Steps: as M1 in each. Expected: either the request shows, or the tool fails with `approval_unavailable` and nothing is committed. Record which. A silent write is a bug.
- [ ] [manual] M4, GitHub Copilot in VS Code. Steps: as M1. Expected and record: as M3.
- [ ] [manual] M5, OpenAI Codex CLI. Steps: as M1. Expected and record: as M3. Also record which setting of Codex controls the request.
- [ ] [manual] M6, a long wait. Steps: as M1 in each client that showed the request, and wait 100 seconds before the answer. Expected: the write continues after Accept. If the client cuts the call earlier, record the time-out of the client.
- [ ] [manual] M7, automatic answers. Steps: in Claude Code, add a hook that answers elicitation requests with `accept` and `approve` true. Run M1. Expected: the write continues with no question. The README states that a hook can do this.
- [ ] [manual] M8, the permission-prompt way. Steps: in Claude Code, set the permission rule `ask` for `mcp__waymark__add`. Ask the agent to add a document. Expected: Claude Code asks before it calls the tool. Record the result with a mode that skips prompts.

## Notes

The manual tests fix the open questions of the old spec: which clients support elicitation, whether a prompt-skipping mode still asks, whether a client can auto-answer, and whether a 2-minute wait is accepted. The result of each goes into the README section "Approval".
