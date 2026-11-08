# Decisions: App link

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## One protocol, a transport per OS — 2026-11-06

**Decision:** Small JSON messages over a Unix domain socket on macOS and Linux, a named pipe on Windows later. XPC stays an option for hardening.

**Alternatives considered:**
- XPC only — macOS only, which would block a portable CLI.
- A local HTTP server — a port that other programs can reach.

**Reason:** A portable protocol with a platform-specific transport. The approval dialog stays in the app, so the agent only receives approved or canceled.

## The dialog belongs to the app, not the terminal — 2026-11-06

**Decision:** The agent cannot answer the dialog. No answer in 2 minutes means cancel.

**Reason:** This is the only enforced human check. It is a policy with an audit trail, not a hard control against a deliberate bypass.
