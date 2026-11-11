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
