# Tasks: Write approval

_`requires`: `mcp-server`, `write-documents` (both must be `implemented` before this spec moves to `in-progress`)._

- [x] Spike (done inside `tests/it_write_approval.rs`, no separate file): enable the `rmcp` feature `elicitation` in `Cargo.toml`. Write `tests/it_elicitation_spike.rs` with a minimal server that sends one form request and returns the answer. Record the findings in `decisions.md`. Use the fallback if the spike fails.
- [x] No `Approver` trait: `writes::approve` and `mcp::approval::ask` do it (see `decisions.md`).
- [x] `src/config/mod.rs`: `EffectiveSettings::approval_required`, with unit tests (tool list, `write`, preview, kind).
- [x] `src/mcp/approval_text.rs`: render the message for every tool of the table, and the cut. Unit tests.
- [x] `src/mcp/approval.rs`: `ElicitationApprover` with the capability check, the request, the time-out, the progress messages, and the answer mapping.
- [x] `src/mcp/context.rs` and `src/mcp/mod.rs`: keep the client capabilities and build the approver for each call.
- [x] `src/gitsync/write.rs`: step 6 with the `Approver`, the `console` rule, and the limits check after a yes.
- [x] `src/mcp/error.rs`: the messages and `next` sentences of `approval_denied` and `approval_unavailable`.
- [x] Write tools: the `approval` field in `data`, and the `ApprovalRequest` for `add_document`, `update_document`, `dispute_document`, `deprecate_document`, and `delete_document` (the other tools add theirs in their own specs).
- [x] `tests/it_write_approval.rs`: every test case of `test.md`.
- [x] README: a section "Approval" with the rule, the clients that work, and the permission-prompt way (with the manual test results).
- [x] Run the full suite.
