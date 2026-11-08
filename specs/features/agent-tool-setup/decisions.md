# Decisions: Agent tool setup

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Three tools from day one — 2026-11-06

**Decision:** Claude Code, GitHub Copilot, and OpenAI Codex. Each runs locally per person, with their own license. No cloud-hosted or background agents.

**Reason:** These are the tools the first users have. Local sessions keep every write under a person's control.

## A fixed path in the instruction block — 2026-11-06

**Decision:** The block references the local path of the knowledge base.

**Reason:** A fixed path is the simplest way for agents in other repositories to find it, and the main reason to avoid an MCP server at first.

## Markers and a manifest — 2026-11-06

**Decision:** Blocks sit between marker comments. Setup records every item in a manifest.

**Reason:** Updates and uninstall touch exactly what waymark wrote.
