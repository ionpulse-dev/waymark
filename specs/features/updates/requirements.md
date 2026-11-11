---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - search-and-read
---

# Updates

## User Story

> As a person, I want to see what changed since I last looked, so that I find out about disputes and changes by others without reading everything.

## Personas

- **Every person** — wants to know about disputes on their own documents.
- **Team lead** — checks what changed in a folder.

## Behaviour

| Tool | Task |
|---|---|
| `updates` | What changed since a date. Input: optional folders and an optional `since` date. Without `since`, it uses the time of the last `updates` call in this knowledge base. Lists new, changed, and disputed documents in the given folders, and changes by others to documents the person wrote. |

- No folder means everything. Folders include subfolders. The `folders` limit applies.
- The output is honest. It says "unverified" and "not checked" in plain words.
- `updates` reads Git history. It closes the gap that waymark cannot push to a person.
- The same facts that matter most arrive as `notices` in other results (see `notifications-and-review`). When an agent runs `updates` is the choice of the team.
- The time of the last call is stored per person and knowledge base. In local mode it is a small file in the personal config folder. In server mode it is stored by the server for each person.

## Interfaces & Contracts

`updates` takes optional `folders` and `since`. The result lists documents with ID, title, kind of change, the person, and the date.

## Acceptance Criteria

- **Given** a document I wrote that someone disputed, **When** `updates` runs, **Then** it lists the dispute.
- **Given** no `since`, **When** `updates` runs twice, **Then** the second run covers the time since the first.
- **Given** a folder, **When** `updates` runs, **Then** it lists only changes to documents under it.
- **Given** a document outside the `folders` limit, **When** `updates` runs, **Then** it is not listed.

## Constraints & Edge Cases

- The check that flags documents whose anchor changed comes after the first release (see `docs/ideas.md`).

## Open Questions

- Where is the "last call" time stored when several agent tools run `updates` for the same person?
