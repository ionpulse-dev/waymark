---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - project-binding
---

# Session start

## User Story

> As a person, I want my agent to know at the start of a session which knowledge base it uses and which limits apply, so that I am never surprised.

## Personas

- **Every person** — can see the knowledge base in use and the limits.

## Behaviour

When an MCP client connects, the server answers with its **instructions**. They are a short, neutral summary and hold no behavior advice:

- what waymark is, in one sentence, and the meaning of a document, a folder, and an ID,
- the knowledge base in use (name and ID),
- the effective limits (see `trust-model`),
- open notices.

The same facts are in the `status` tool, which an agent or a person can call at any time. Every tool result also carries the current `settings`, so a change of a limit applies at once.

Whether the agent tells the person, and what it does first in a session, is the choice of the team, in its `AGENTS.md`.

The summary only informs. Every tool checks the same limits itself, so an ignored summary cannot allow a forbidden action.

## Interfaces & Contracts

The instructions are plain text of a few lines, for example:

```text
waymark: documents of the knowledge base "Example" (read-write, paused: no).
Tools: search, show, list, add, ... Limits: results 5, documents opened per session 20.
```

## Acceptance Criteria

- **Given** a bound project, **When** a client connects, **Then** the instructions name the knowledge base and carry the effective limits.
- **Given** the instructions, **When** they are read, **Then** they hold no advice on when to read or write.
- **Given** writes are paused, **When** a client connects, **Then** the instructions and a notice say so.
- **Given** an agent that ignores the instructions, **When** it runs a write, **Then** the server still enforces all limits.
- **Given** no knowledge base is set up, **When** a client connects, **Then** the instructions say so and `status` shows how to fix it.

## Constraints & Edge Cases

- Some clients ignore server instructions. The facts are in `status` and in every result.
- The instructions stay short, because they use context in every session.

## Open Questions

- Do Claude Code, Copilot, and Codex show or use server instructions today? Verify.
