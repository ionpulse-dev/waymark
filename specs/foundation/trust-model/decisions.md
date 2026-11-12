# Decisions: Trust model

> **Note 2026-11-11:** The decisions below that mention read and write levels, the app, or the dialog are superseded by "Limits instead of levels" and "Config layers, the strictest wins" at the end of this file. "Wiki model", "No agent attribution", "No pull request gate in the first release", and "Raw Git is forbidden only by rule" stay valid. In "Raw Git is forbidden only by rule", read "in the `AGENTS.md` of the team" for "in the skill".

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

## Limits instead of levels — 2026-11-11

**Decision:** The read level and the write level are dropped. The server enforces limits: access, pause, switched-off tools, folders, caps, and approval rules. When an agent reads or writes is the choice of the team, in its `AGENTS.md`.

**Supersedes:** "Separate read and write levels, default manual" and "Several knowledge bases with access modes" (access mode becomes the limit `access`).

**Alternatives considered:**
- Keep the levels as advice text. waymark no longer ships behavior text, so levels would have no home.

**Reason:** Limits can be enforced and tested. Advice cannot.

## Config layers, the strictest wins — 2026-11-11

**Decision:** Limits come from five layers: system file, server config, shared knowledge base file, personal file, and MCP entry. The strictest layer wins. A layer can only tighten. A system file owned by an administrator gives IT a hard limit in local mode.

**Supersedes:** "Only the person changes settings" (the app is dropped). Only the person or the operator changes limits, and no tool can.

**Alternatives considered:**
- Limits only in the MCP entry. The user controls that file, so it is no hard limit.

**Reason:** Every case gets the strongest layer that fits it. Local mode is honest about what is hard and what is a safety default.

## Built-in default is `read-write` — 2026-11-11

**Decision:** With no layer set, `access` is `read-write`. The README suggests `read-only` for a first trial.

**Alternatives considered:**
- `read-only` as default, so that writes need an explicit step. This is listed as an open question.

**Reason:** Behavior is the choice of the team. Writes are safe through checks, history, and undo. A layer can tighten at any time.

## Reading is always allowed, autonomy is not a limit — 2026-11-12

**Decision:** Limits do not switch reading off. They set where (`folders`) and how much (`max_results`, `max_tokens`, `max_documents_opened`) an agent reads. "Read on its own" or "read only when asked" is the choice of the team (`AGENTS.md`) and of the agent tool (permission prompts).

**Alternatives considered:**
- A read level on the server. The server sees a tool call and cannot tell whether the person asked for it.
- Approval for every read. Unbearable as a default. It stays possible for sensitive folders later.

**Reason:** The server can only enforce what it can see. The only levels left are for writes: not allowed, allowed after a yes, allowed.

## Local mode: the simple config is the gate — 2026-11-12

**Decision:** In local mode, limits come from the MCP entry and the shared file, and the docs say that this guards against mistakes only. The system config file stays an optional layer for companies. Hard permissions belong to server mode, where they come from the sign-in.

**Alternatives considered:**
- Make local mode hard with root-owned files and a locked MCP config. The setup is complex for private people and small teams. An agent can also start a second `waymark` process with other arguments, and an agent that has the Git login of the person can push without waymark.
- Design the full server permission system now. Server mode only changes who sets the limits and how they are enforced. It does not change what a limit means.

**Reason:** Keep the first product simple and honest. Keep three things so that later work is cheap: one set of limit names, the rule "the strictest layer wins", and all checks in one place in the core.

## No separate MCP connections in local mode — 2026-11-12

**Decision:** Local mode uses one MCP entry and one tool set. Separate connections for read and write are not built. For server mode, they stay an option (see `server-mode`).

**Alternatives considered:**
- A read entry and a write entry in local mode. The agent tools already set `allow`, `ask`, and `deny` for each tool name, so a second entry adds setup and context cost and no hard limit.

**Reason:** One entry is the goal: a person adds one MCP entry and everything else works.
