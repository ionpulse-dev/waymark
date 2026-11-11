# Decisions: Plain-Git access

> **Note 2026-11-11:** All decisions stay valid. Mentions of the CLI now mean the MCP tools.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## The rules ship with every repository — 2026-11-07

**Decision:** `AGENTS.md` in every content repository always carries the plain-Git rules, on by default.

**Reason:** An agent that finds the repository by chance has clear rules and does not damage the data. Loading the repository in Claude Code on the web or iPhone gives the agent the whole knowledge base and the rules, with no CLI or MCP.

## A standalone skill, in its own repository — 2026-11-08

**Decision:** The rules are the standalone skill of the `waymark-trail` project. waymark copies it into the template and pins a version. The skill never depends on waymark.

**Alternatives considered:**
- A separate skill file inside waymark — the skill would be tied to waymark and could not spread on its own.
- Two modes inside one `AGENTS.md` without a standalone text — hard to version and to share.

**Reason:** The skill is useful without the project and can spread on its own.

## Fallback, not main path — 2026-11-07

**Decision:** This is a second access channel and a safe default. It is not a replacement for the CLI.

**Reason:** It opposes the "agents do not run raw Git" rule, so only the skill rule holds.

## MCP tools replace the CLI in `AGENTS.md` — 2026-11-11

**Decision:** The `AGENTS.md` of a content repository says: use the waymark tools if present, else the plain-Git rules.

**Reason:** MCP is the only interface for agents. The plain-Git rules remain the fallback.
