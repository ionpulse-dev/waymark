---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# Agent skill

## Technical Goal

The skill is the manual and the rules of behavior for an agent that uses waymark. It decides whether waymark is useful: when to read, when to write, what is worth a document, and how to write it. It is the most important text of the project.

## Dependents

- `agent-tool-setup` — writes the skill into each agent tool.
- `session-start` — the skill makes the agent run it first.
- `search-and-read` and `write-documents` — the skill guides how agents use them (not a build dependency).

## Deliverables

- The skill text, in English, compiled into the CLI.
- The static project instruction block that points to `.waymark/id`.

## Scope

**In scope:**
- The content of the skill (below).

**Out of scope:**
- Writing it into tools (see `agent-tool-setup`).
- The standalone skills of `waymark-trail`, which are a separate project.

## Constraints

- The text is fixed, not editable by users, the same in every project, and version-matched to the CLI.
- English only. The agent speaks the language of the person and writes documents in the language of the knowledge base.
- The skill is advice. Enforced limits live in the CLI.

### What the skill covers

- **Manual:** what waymark is and the commands.
- **Start of a session:** run `waymark session` first, show the person a one-line reminder of the settings and the knowledge base in use, and follow the `settings` of every later result.
- **When to read:** at read level `auto`, search before cross-team or durable work and report `updates` at session start. At `manual`, only when asked.
- **When to write (triggers):** A the person asks, B a durable fact is learned, C a decision is made, D a contradiction is found. Triggers B to D are guidance. At write level `manual` the agent acts only on trigger A. At `confirm` it asks first. At `auto` it writes.
- **What is worth a document:** true in three months, useful outside the team, no status updates, no chat summaries.
- **How to write:** one topic per document, a short description, the shortest complete text, sources, anchors, links, and the right folder. Keep documents small. Splitting is the agent's own judgment. Parts link to each other, optionally with a short index document.
- **How to treat documents:** use them, name the author and date, check important facts against the code, and dispute contradictions.
- **Levels and confirmation:** when to ask, and wait for the dialog. Allow enough time for it, because some tools stop a command after about 2 minutes. Know the exit codes.
- **Notices:** pass them on to the person at once.
- **Forbidden:** raw Git commands in the content repository, changing a level, binding a folder, verifying without a human "yes", carrying content from one knowledge base to another, and writing secrets.

## Acceptance Criteria

- **Given** a new session, **When** the agent follows the skill, **Then** it runs `waymark session` before any other waymark command.
- **Given** read level `manual`, **When** the person does not ask, **Then** the agent does not search.
- **Given** a result with notices, **When** the agent answers, **Then** it tells the person.
- **Given** a skill update in a new release, **When** `update` runs, **Then** every tool gets the new text.
- **Given** two projects, **When** the skill is read in each, **Then** the text is identical.

## Open Questions

- How long should the skill be, so that it stays cheap in every session?
- Which wording makes all three agent tools follow the start-of-session step reliably?
