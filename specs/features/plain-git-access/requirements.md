---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - knowledge-base-repo
---

# Plain-Git access

## User Story

> As a person, I want to open my knowledge base repository in any agent session and have the agent read and write it safely with ordinary Git, so that I can use it on the web or on my phone without installing anything, and so that agents without MCP support can still use it.

## Personas

- **Private person** — uses Claude Code on the web or on iPhone.
- **Anyone** whose agent finds the repository by chance.

## Behaviour

Every content repository has an `AGENTS.md` in its root. The `init` tool writes it from a template. It always contains the plain-Git rules, which are a copy of a standalone skill (today in the separate `waymark-trail` repository, independent of waymark; its future home is an open question). `init` copies the `personal-knowledge-base` skill for a knowledge base of one person and the `team-knowledge-base` skill for several people. The skill works on any Git repository of Markdown documents with no `.mkbase/waymark.toml` and no waymark program.

The file has two modes:

- With the waymark MCP tools available, the agent uses the tools. They enforce the checks.
- Without them, the agent follows the plain-Git rules: document format, reading and searching, writing, editing, moving with link rewrite (`git mv`, see `move-documents`), pull with `--rebase` before a write, the base-version check, the commit author, and no secrets.

The standalone skill reads and writes only when the person asks. The person can loosen this in the chat. This rule belongs to the standalone skill, not to the waymark MCP server, which leaves behavior to the team.

Advantages: no install, safe by default, and no cost. Limits: without the MCP server only the text rule holds. There is no `lint`, no approval, no pause switch, no limits, and no email check.

waymark depends on the skill, never the other way. waymark pins a version of the skill and updates the copy in the template with each release.

## Interfaces & Contracts

The template block between marker comments holds the skill text and a version header.

## Acceptance Criteria

- **Given** the `init` tool, **When** it ends, **Then** `AGENTS.md` contains the plain-Git rules and the skill version.
- **Given** the repository opened in Claude Code on the web, **When** the agent follows `AGENTS.md`, **Then** it can search and write a valid document with no install.
- **Given** an agent with the waymark MCP tools, **When** it reads `AGENTS.md`, **Then** it uses the tools.
- **Given** a new release, **When** the template is built, **Then** the copy of the skill has the pinned version.
- **Given** a repository without waymark, **When** the standalone skill is used, **Then** it never mentions waymark IDs or settings files.

## Constraints & Edge Cases

- Without the MCP server the agent could ignore the rules. This is a fallback channel, not the main path.
- The plain Claude chat tab and non-GitHub hosts are not verified for this mode.

## Open Questions

- Where do the standalone skills live once `waymark-trail` is retired? The standard is already in `standard/` of this repository.

- How does waymark pin a version of the standalone skill: a Git tag, a commit hash, or a version header in the copy?
