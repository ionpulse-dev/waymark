---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-12
requires:
  - mcp-server
---

# Agent tool setup

## User Story

> As a person, I want a clear recipe for each of my agent tools, so that I add waymark with one entry and write my own rules, without waymark editing my files.

## Personas

- **Every person** — uses Claude Code, GitHub Copilot, or OpenAI Codex.
- **A team** — writes its own `AGENTS.md` and shares an MCP entry in the project.

## Behaviour

waymark does not install anything into agent tools. It ships **no skill** and **no instruction block**. It gives documentation in the README:

1. **MCP entry for each tool.** A copy-paste entry for Claude Code, GitHub Copilot (VS Code and CLI), and OpenAI Codex, for user scope and for project scope. The formats differ (JSON for some tools, TOML for Codex).
2. **Permission advice for each tool.** How to allow the read tools and let the tool ask before a write tool. The rules are in the permission settings of the tool, not in the MCP config. For Claude Code, they are `allow` and `ask` rules such as `mcp__waymark__search` in `settings.json`. Tool kind hints (read-only, write, destructive) support this. The README says that a mode without prompts ("YOLO") ignores `allow` rules, may still honor explicit `ask` rules (verify), and that the built-in approval through elicitation is the other way (see `write-approval`).
3. **Example `AGENTS.md` snippets.** Short examples that a team may copy and change. They are suggestions only. waymark does not insert them and does not check them. Suggested examples:
   - *Manual:* use waymark only when asked.
   - *Balanced:* search at the start of a task, write only when asked or after the person says yes.
   - *Proactive:* search first, save decisions and pitfalls, update before adding, dispute instead of deleting, and tell the person in one line what was saved.
   All examples end with "never write secrets, tokens, or personal data". The server enforces the hard rules. The examples only guide behavior.
4. **How to tighten limits.** Where the personal config, the shared file, and the system config live (see `trust-model`).

The entries use the same server name, `waymark`, so that a project entry can override a user entry (see `project-binding`).

## Interfaces & Contracts

The README sections are the deliverable. Each snippet is tested before a release against the current version of its tool.

## Acceptance Criteria

- **Given** the snippet for a tool, **When** a person adds it and starts a session, **Then** the tool lists the tools of waymark.
- **Given** a tool that is not listed in the README, **When** it speaks MCP over stdio, **Then** the generic entry (command and arguments) works.
- **Given** the README, **When** it is read, **Then** every `AGENTS.md` example is marked as a suggestion.
- **Given** an installed waymark, **When** the home and project folders are inspected, **Then** waymark wrote no file into any agent tool folder.
- **Given** a new release, **When** the snippets are checked, **Then** each one works with the current tool version.

## Constraints & Edge Cases

- No cloud agents. Only local tools that a person starts, or a login on a server.
- The paths and formats of each tool are unverified until the first implementation phase. The snippets list the tool version they were tested with.
- The person may write any rules in their own `AGENTS.md`. waymark does not edit them.

## Open Questions

- What are the verified MCP config formats and locations of each tool?
- Do all three tools show tool kind hints and ask before write tools?
- What does each tool do with `ask` rules when it runs without permission prompts?
