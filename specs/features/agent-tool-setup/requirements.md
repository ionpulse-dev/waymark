---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - agent-skill
  - join-knowledge-base
---

# Agent tool setup

## User Story

> As a person, I want setup to prepare my agent tools, so that my agents know about the knowledge base without me editing files.

## Personas

- **Every person** — uses Claude Code, GitHub Copilot, or OpenAI Codex on their own computer.

## Behaviour

Setup detects which of the three tools are present and asks which to configure. For each chosen tool it:

- adds a short instruction block to the global instructions of the tool, between the markers `<!-- waymark:start -->` and `<!-- waymark:end -->`. Setup replaces only this block and never touches other content.
- writes the skill from the binary into the global skill folder of the tool. If a tool has no skill support, the instruction block points the agent to the skill file by path.

The block is short. It says where the knowledge base is, when to search, when to write, and to use the `waymark` CLI and follow the `AGENTS.md` of the knowledge base.

Global locations, to be verified against the current tool versions before the first release:

| Tool | Global instructions | Global skills |
|---|---|---|
| Claude Code | `~/.claude/CLAUDE.md` | `~/.claude/skills/<name>/SKILL.md` |
| OpenAI Codex | `~/.codex/AGENTS.md` | `~/.codex/skills/<name>/SKILL.md` (verify) |
| GitHub Copilot | `~/.copilot/copilot-instructions.md` (CLI), VS Code user instructions | agent skills folder (verify) |

All three tools read Markdown instructions, so one `SKILL.md` serves all three.

Setup writes a manifest of everything it installs (`~/.config/waymark/installed.json`), used by `uninstall`.

## Interfaces & Contracts

The manifest lists every file and every block that setup created, with the tool name and a hash.

## Acceptance Criteria

- **Given** an existing global file with other content, **When** setup runs, **Then** only the block between the markers changes.
- **Given** setup ran twice, **When** the second run ends, **Then** there is one block per tool.
- **Given** a new release, **When** `waymark update` runs, **Then** the skill file in every tool is replaced.
- **Given** a tool that is not installed, **When** setup runs, **Then** it does not create files for it.
- **Given** the manifest, **When** `uninstall` runs, **Then** every item in it is removed.

## Constraints & Edge Cases

- No cloud agents. Only local tools that a person starts.
- The paths in the table are unverified until the research in the first implementation phase.
- The user may add rules in their own `AGENTS.md`. waymark does not edit them.

## Open Questions

- What are the verified skill and instruction paths of each tool?
