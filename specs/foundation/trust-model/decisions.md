# Decisions: Trust model

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Wiki model — 2026-11-06

**Decision:** Every write goes to `main` under the Git account of the person whose agent wrote it. That person is responsible. The `disputed` flag stays. (The stale expiry was dropped later, see `updates`.) Owner verification is optional.

**Alternatives considered:**
- Direct write with review after the fact — readers cannot tell checked knowledge from guesses.
- An inbox that a curator promotes — no background curator exists, so the inbox grows, and moving files breaks links.
- A pull request per write — too slow, owners become a bottleneck.
- Trust as a stored state with owner verification as the default — needs a clear owner for every document and a review queue.

**Reason:** Responsibility goes to the crowd, like a company wiki, executed by AI agents. No Git knowledge is needed and nothing is moved.

## No agent attribution — 2026-11-06

**Decision:** Documents and commits record only the person.

**Reason:** A self-reported agent name can be wrong or forged. Revisit if a tool offers a reliable way to identify itself.

## Separate read and write levels, default manual — 2026-11-07

**Decision:** The first design had per-type autonomy and approval levels. It became one write level (`auto`, `confirm`, `manual`) plus confirm-in. Later the read level was split off. Defaults are the strictest. The person loosens them in the app.

**Alternatives considered:**
- Per-trigger levels and per-type approval — too many settings.
- One level for reads and writes — "read on its own, write only when asked" was not possible.
- Default `auto` — people feel safer with more control at the start.

**Reason:** One simple system. The dialog stays for people who want enforcement.

## Only the person changes settings — 2026-11-06

**Decision:** Personal settings change only in the app. The CLI has no command to change them, not even to tighten. Non-interactive `setup` with flags may set initial values for rollout scripts.

**Reason:** Simpler and safer than "tighten only". An agent can never change its own limits.

## Several knowledge bases with access modes — 2026-11-07

**Decision:** A person can connect several knowledge bases, each with an access mode. A new one starts `off`. Mixing content is limited by binding a project folder to exactly one knowledge base (see `project-binding`).

**Alternatives considered:**
- One knowledge base per person — blocks the common case of a company and a private knowledge base.
- Only one active at a time — too rigid.

**Reason:** An agent with access to two knowledge bases could copy confidential content between them. The mode, the binding, and the repository name in the dialog limit that risk.

## No pull request gate in the first release — 2026-11-07

**Decision:** A second person approving a write is not in the first release.

**Reason:** Keeps the first release simple. It is the only server-side approval control and needs branch protection. See the ideas document.

## Raw Git is forbidden only by rule — 2026-11-07

**Decision:** The only enforcement that writes go through waymark is the rule in the skill and the repository `AGENTS.md`. No Git hooks, credential isolation, or signed commits in the first release.

**Reason:** Keep the first release small. The CLI still checks every write that goes through it. Hard layers are ideas for later.
