---
type: feature
status: draft
created: 2026-11-11
updated: 2026-11-11
requires:
  - server-mode
---

# Web console

## User Story

> As a person or an owner on a shared server, I want one web page where I approve writes, see what my agents wrote, undo it, and stop writes, so that I stay in control without Git or a terminal.

## Personas

- **Every person** — approves, reviews, and undoes their own writes.
- **Knowledge base owner** — sees all activity, pauses writes, and verifies documents.
- **Operator** — sees the connection state of the knowledge bases.

## UI / Visual Design

The console is a small web application that the server itself serves. It works on any system with a browser. The person signs in with the same single sign-on as for MCP, in a separate browser session that the agent cannot reach.

### Pages

- **Approvals.** The open requests of `write-approval` with the full document or the summary, and the buttons **Publish** and **Cancel**.
- **Activity.** The writes made under the person's account, newest first, each with **Undo** (see `activity-and-undo`). Owners can see all people.
- **Review.** Open items: disputes on the person's documents, and documents waiting for verification (see `notifications-and-review` and `verification`).
- **Pause.** One switch that pauses all writes of the person. An owner can pause a whole knowledge base. Resuming needs the console, not an agent (see `pause-and-resume`).
- **Knowledge bases.** For each: state of the clone, last good pull, last error, Git credential state.

### Error States

- The server cannot reach the Git host: the page says so with the time of the last good pull.
- A request expired: the page says "Canceled: no answer in time".

## Behaviour

- Every action in the console is an audit line (see `server-mode`).
- The console holds no document logic. It calls the same core as the tools.
- The console is part of server mode. Local mode has no console. There, the client's prompts and the personal config cover these jobs.

## Interfaces & Contracts

The console uses internal HTTP routes of the server. They are not an API for agents and are not exposed as MCP tools.

## Acceptance Criteria

- **Given** a write that needs approval with `via = "console"`, **When** the person opens the console, **Then** the request shows with the full document.
- **Given** a click on Publish, **When** the answer is stored, **Then** the waiting tool call continues.
- **Given** an agent with a valid MCP token, **When** it calls a console route, **Then** the server rejects it.
- **Given** the pause switch, **When** the person turns it on, **Then** the next write of that person fails with `paused`.
- **Given** a person who is not an owner, **When** they open Activity, **Then** they see only their own writes.
- **Given** no open items, **When** Review opens, **Then** it is empty and says so.

## Constraints & Edge Cases

- A person who uses neither an agent nor the console cannot be reached by waymark.
- The console must work on a phone screen.

## Open Questions

- Do notifications by email or webhook belong in the first release?
- How does the console authenticate so that a token of the agent can never be used for it?
