# Decisions: Login and identity

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Reuse `gh` or paste a token — 2026-11-06

**Decision:** The first release reuses the `gh` token, or the person pastes a token that setup helps to create. Token in the Keychain. Device login comes later.

**Alternatives considered:**
- OAuth device flow first — needs an OAuth app per Git host.

**Reason:** Small to build and no OAuth app to register now.

## The email must reach a person — 2026-11-06

**Decision:** waymark shows the email from the Git host account and the person confirms or corrects it. The checks run in setup and before every commit.

**Alternatives considered:**
- A separate list of people in the repository — extra state.
- Trusting the account email — it is often a private no-reply address.

**Reason:** Every document has a reachable author through Git history. Contact happens outside waymark. If the author is gone, the team folder is the fallback.
