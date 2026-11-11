---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - updates
---

# Notices and review

## User Story

> As a person, I want to hear about problems with my documents and my knowledge base, so that I can act without watching anything.

## Personas

- **Every person** — gets notices in the results of their agent and sees one list of open items.

## Behaviour

waymark has no background process in local mode and cannot push a message to a person. It uses three ways:

- **Notices in results.** Every tool result may carry `notices` (see `mcp-server`). They are short plain sentences. The server adds a notice for: a document of the person was disputed or changed by others, the last pull failed, writes are paused, and the clone may be behind. A notice is shown until the problem is gone or the person has seen it in `updates`.
- **The `updates` and `status` tools.** A person or an agent can ask at any time (see `updates` and `maintenance`).
- **The review list in the web console** (server mode, see `web-console`): disputes on the person's documents and, with verification on, documents that wait for verification (see `verification`).

What an agent does with a notice is the choice of the team, in its `AGENTS.md`. A suggested wording for the README: tell the person at once.

Optional later: email or webhook messages from a server.

## Interfaces & Contracts

A notice is a string in the `notices` list. The server keeps a small record per person to avoid repeating the same notice in every result. The default is once per session and problem.

## Acceptance Criteria

- **Given** someone disputes my document, **When** the next tool result is built after the next pull, **Then** it carries a notice.
- **Given** a failed pull, **When** a read runs, **Then** the result carries a notice that the clone may be behind.
- **Given** the same problem in one session, **When** several results are built, **Then** the notice appears once.
- **Given** no problems, **When** a result is built, **Then** `notices` is absent.
- **Given** a server and a disputed document of mine, **When** the console review list opens, **Then** it shows with an action.

## Constraints & Edge Cases

- waymark cannot reach a person who uses neither an agent nor the console.
- Desktop notifications do not exist. They belonged to the old app.

## Open Questions

- How often should a notice repeat for the same problem across sessions?
