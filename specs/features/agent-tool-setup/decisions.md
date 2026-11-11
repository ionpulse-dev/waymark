# Decisions: Agent tool setup

> **Note 2026-11-11:** All decisions below are superseded by "Documentation, not installation" at the end of this file.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Three tools from day one — 2026-11-06

**Decision:** Claude Code, GitHub Copilot, and OpenAI Codex. Each runs locally per person, with their own license. No cloud-hosted or background agents.

**Reason:** These are the tools the first users have. Local sessions keep every write under a person's control.

## A static block, found by id — 2026-11-07

**Decision:** The block and the skill hold no path. The agent finds the knowledge base by the id in `.waymark/id`. The first design used a fixed path in the block, `~/Projects/<slug>-knowledge-base`; the id binding replaced it.

**Reason:** The text stays static and version-matched, and several knowledge bases can coexist. A known location of the id still lets agents in other repositories find the knowledge base without an MCP server.

## Markers and a manifest — 2026-11-06

**Decision:** Blocks sit between marker comments. Setup records every item in a manifest.

**Reason:** Updates and uninstall touch exactly what waymark wrote.

## Documentation, not installation — 2026-11-11

**Decision:** waymark installs nothing into agent tools. The README gives MCP entries and example `AGENTS.md` snippets.

**Supersedes:** "Three tools from day one" (the three tools stay the documented ones), "A static block, found by id", and "Markers and a manifest".

**Alternatives considered:**
- A helper command that writes the MCP entry. It would be a second interface and would write into files of other tools.

**Reason:** No skill, no instruction block, and no installer.
