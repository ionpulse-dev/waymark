# Decisions: Write approval

## Approval is a limit, not a level — 2026-11-11

**Decision:** Approval is configured by `approval.required_for` and `approval.via`. There are no write levels.

**Reason:** The server can enforce a list of tools. It cannot enforce when an agent decides to write.

## Fail closed — 2026-11-11

**Decision:** If approval is required and cannot be asked, the write fails.

**Reason:** A silent fallback to "no approval" would defeat the limit.

## Only the console is enforced — 2026-11-11

**Decision:** `console` is the only enforced channel. `elicitation` is honest advice that depends on the client.

**Alternatives considered:**
- A native app dialog. It is macOS only and needs an app. The console gives the same agent-proof channel on every system.

**Reason:** The value of the old dialog was a channel the agent cannot reach. A separate sign-in on the web gives that.

## Time-out means no — 2026-11-11

**Decision:** No answer in 2 minutes ends the write with `approval_denied`.

**Reason:** Carried over from the old approval dialog.

## Supersedes `approval-dialog` — 2026-11-11

The spec `approval-dialog` described the native macOS dialog. It is superseded by this spec and by `web-console`.

## Two ways in local mode, nothing built for the first — 2026-11-12

**Decision:** In local mode, a yes comes from the agent tool (permission prompt on the write tools) or from waymark (elicitation). The first needs documentation and tool kind hints only. The second is the built-in channel.

**Alternatives considered:**
- A native operating system dialog from waymark. Different on every system, and it fails on machines without a screen.
- Staging a write for a later publish (a branch or a pull request). It needs a second step and a Git host. It stays the idea `pull-request-gate`.
- Separate MCP connections for read and write. Not needed, because the agent tools set permissions for each tool.

**Reason:** Both ways use what the agent tools already offer, and both are honest about their strength. Only `console` on a server is enforced.

## Local mode has one channel: elicitation — 2026-11-14

**Decision:** Only `approval.via = "elicitation"` is built. A `console` value in any layer makes every approval fail with `approval_unavailable` in local mode.

**Alternatives considered:**
- Treat `console` as `elicitation` in local mode. Rejected: a layer that asked for the strong channel would silently get the weak one. This loosens a limit.
- Ignore the value. Rejected: same reason.

**Reason:** Fail closed, as `trust-model` says for `via`. The console is out of scope for now. Decided autonomously.

## The form has one boolean `approve` — 2026-11-14

**Decision:** The request uses form mode with one required boolean `approve` (default `false`). A yes is `accept` with `approve` equal to `true`.

**Alternatives considered:**
- An empty form where `accept` alone means yes. Simpler to click, but a client or hook that accepts empty forms by default would approve every write.
- A string field with the values `yes` and `no`. Rejected: a boolean is the simplest schema that all clients render.

**Reason:** The explicit field makes an automatic or sloppy `accept` fail closed. The person ticks one box and presses Accept. Decided autonomously.

## What each action means — 2026-11-14

**Decision:** `accept` with `approve = true` is yes. `accept` otherwise, `decline`, `cancel`, and a time-out are all `approval_denied`, with a different `message` for each. A JSON-RPC error from the client is `approval_unavailable`.

**Alternatives considered:** Treat `cancel` as "ask again". Rejected: a write never repeats a question.

**Reason:** `approval_denied` means the person or the time said no. `approval_unavailable` means no person could be asked. The agent and the person need to tell them apart. Decided autonomously.

## Capability check before the request — 2026-11-14

**Decision:** The server reads the capabilities of the client from `initialize`. The channel is available if `capabilities.elicitation` exists and is an empty object (older clients) or has the key `form`. A client with only `url` is unavailable. A client that announces nothing is unavailable and no request is sent.

**Alternatives considered:** Send the request and wait for an error. Rejected: a client that ignores unknown requests would hold the write for 2 minutes.

**Reason:** Fail fast and fail closed. Some clients did not announce the capability in earlier versions (see `requirements.md` of 2026-11-12). Those clients get `approval_unavailable`, which is safe. Decided autonomously.

## Fixed time-out of 120 seconds with a test override — 2026-11-14

**Decision:** The time-out is 120 seconds, not a setting. The environment variable `WAYMARK_APPROVAL_TIMEOUT_SECONDS` (1 to 600) overrides it, for tests. The progress interval is the smaller of 10 seconds and the time-out divided by 8.

**Alternatives considered:** A limit `approval.timeout` in `trust-model`. Rejected: no need for it in the first release, and it adds a limit with a merge rule.

**Reason:** The variable only shortens or lengthens a wait, and never loosens a limit. Same idea as `WAYMARK_SYSTEM_CONFIG`. Decided autonomously.

## A synchronous wait, no "pending" result — 2026-11-14

**Decision:** The tool call waits for the answer and sends progress messages. There is no asynchronous "pending" result.

**Alternatives considered:** Return `pending` and let the agent call again. Rejected: the agent could then retry or skip the question, and the write flow would need to keep state between calls.

**Reason:** `mcp-server` is stateless between calls. The manual test M6 checks that clients accept the wait. If a client does not, the result is a failed write and the person can retry (fail closed). Decided autonomously.

## No diff in the first release — 2026-11-14

**Decision:** An update shows the full new body and the word counts before and after. It shows no diff.

**Alternatives considered:** A unified diff. Rejected for now: more code, and many clients show plain text only.

**Reason:** The person sees the final text. A diff is in `todo.md`. Decided autonomously.

## The approval goes through a trait in `gitsync` — 2026-11-14

**Decision:** `gitsync::Repo::write` takes an `&dyn Approver`. The trait and the request type live in `src/gitsync/approval.rs`. The elicitation implementation lives in `src/mcp/approval.rs`, so `gitsync` has no dependency on `rmcp`.

**Alternatives considered:** `gitsync` calls `rmcp` itself. Rejected: it breaks the module layout of `mcp-server`, where all `rmcp` code is in `src/mcp/mod.rs`.

**Reason:** One place for the `rmcp` code, and tests of the write flow need no client. This extends the plan of `git-sync` (see the report). Decided autonomously.

## Limits are read again after the answer — 2026-11-14

**Decision:** After a yes, `Repo::write` loads the effective limits again. If `write_block` says `Some`, the write fails with the code of `pause-and-resume`.

**Reason:** The kill switch must work during the wait. See `specs/features/pause-and-resume/decisions.md`. Decided autonomously.

## Verify `rmcp` elicitation first — 2026-11-14

**Decision:** The first task is a spike: build the smallest server with `rmcp` 3.x that sends one elicitation request and reads the answer, tested with the client of `integration-testing`. Findings from the documentation on docs.rs of 2026-11-14: the Cargo feature `elicitation` is not a default feature. `Peer<RoleServer>` has `create_elicitation`, `create_elicitation_with_timeout`, `elicit`, `elicit_with_timeout`, and `supported_elicitation_modes`. `ElicitResult` has `action` (`Accept`, `Decline`, `Cancel`), `content`, and `meta`. The enum and struct are `#[non_exhaustive]`, so matches need a wildcard arm that means "denied".

**Fallback:** If the form cannot be built with a hand-written schema, or the time-out does not send a cancel, the spike result goes into this file and the request is sent with `send_request` as raw JSON-RPC.

**Reason:** `rmcp` 3.x is new to the plan, and the `mcp-server` plan names elicitation support as a risk. Decided autonomously.

## Console and server-mode channel are cut — 2026-11-14

**Decision:** The channel `console`, its acceptance criterion, and the agent-proof claim are moved to `todo.md`.

**Reason:** The scope on 2026-11-14 is local mode only.

## Implementation notes — 2026-11-14

**Decided autonomously:**
- The spike passed: `rmcp` 3.10 with the feature `elicitation` sends a form request built from JSON (`ElicitRequestParams` deserializes the `mode` tag). `accept`, `decline`, and `cancel` arrive as `ElicitationAction`. A time-out gives `ServiceError::Timeout` and a `notifications/cancelled`. `supported_elicitation_modes` reads the capability.
- No `Approver` trait. `writes::approve` (policy, limits again after a yes) calls `mcp::approval::ask`. The tool call keeps the connection in `Context.channel`, set at each `call_tool`. The write flow runs on a blocking thread, so `ask` uses `Handle::block_on`.
- The message is built from `Plan.summary` (first line, then a blank line and the full body), plus the tool, the author, and the changed paths when there is no body. The cut is at 20000 characters.
- The `approval` result field is `approved` or, for a preview, `required`. `sync` and the other tools that use `approve` get `approved` only through `execute`. `sync` adds no field yet (todo).
- The second-process lock test is not written, because the lock wait has no test setting (todo).
