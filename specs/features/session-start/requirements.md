---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - project-binding
  - agent-skill
---

# Session start

## User Story

> As a person, I want my agent to tell me at the start of a session which knowledge base it uses and how much it may do alone, so that I am never surprised.

## Personas

- **Every person** — sees a one-line reminder of the settings.

## Behaviour

`waymark session` is the first command the skill tells the agent to run in every session. It returns JSON with a plain English `message` and `notices`:

- the knowledge base in use,
- the current read level, write level, and confirm in,
- the pause state and the access mode,
- notices,
- at read level `auto`, the result of `updates` for knowledge bases that allow reading.

The agent shows the person a one-line reminder of the settings and the knowledge base. It follows the `settings` of every later result.

The command only informs. Every command checks the same rules itself, so a skipped or ignored `session` call cannot allow a forbidden action.

## Interfaces & Contracts

```json
{
  "message": "Using the knowledge base 'Example' (read and write).",
  "knowledge_base": {"id": "<uuid>", "name": "Example"},
  "settings": {"read_level": "manual", "write_level": "manual", "confirm_in": "chat", "paused": false, "mode": "read and write"},
  "notices": []
}
```

## Acceptance Criteria

- **Given** a bound project, **When** `session` runs, **Then** the result names the knowledge base and carries all five settings.
- **Given** read level `manual`, **When** `session` runs, **Then** it does not include `updates`.
- **Given** read level `auto`, **When** `session` runs, **Then** it includes `updates` for readable knowledge bases.
- **Given** writes are paused, **When** `session` runs, **Then** a notice says so.
- **Given** an agent that skips `session`, **When** it runs a write, **Then** the CLI still enforces all limits.

## Constraints & Edge Cases

- A folder without a binding and without a default returns a message that no knowledge base is in use.
- Session start hooks of agent tools are an option for later.

## Open Questions

- Should a session hook be added for tools that support it?
