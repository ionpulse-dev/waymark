---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-09
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

| Command | Task |
|---|---|
| `waymark updates [<folder>...] [--since <date>]` | What changed since a date. Without `--since`, it uses the time of the last `updates` call. Lists new, changed, and disputed documents in the given folders, and changes by others to documents the person wrote. |

- No folder means everything. Folders include subfolders.
- The skill runs `updates` only when the person asks. It does not report updates at session start.
- The output is honest. It says "unverified" and "not checked" in plain words.
- `updates` reads Git history. It closes the gap that waymark cannot push to a person.

## Interfaces & Contracts

`updates` takes `--kb <uuid>` and `--json`.

## Acceptance Criteria

- **Given** a document I wrote that someone disputed, **When** `updates` runs, **Then** it lists the dispute.
- **Given** no `--since`, **When** `updates` runs twice, **Then** the second run covers the time since the first.
- **Given** a folder, **When** `updates` runs, **Then** it lists only changes to documents under it.
- **Given** a session start at any read level, **When** the skill is followed, **Then** it does not run `updates` unless the person asks.

## Constraints & Edge Cases

- Agents run `dispute` if they find a contradiction.
- The check that flags documents whose anchor changed comes after the first release (see `docs/ideas.md`).

## Open Questions

- Where is the "last updates call" time stored when several tools run `updates`?
