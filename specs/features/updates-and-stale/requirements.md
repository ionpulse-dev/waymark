---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - search-and-read
---

# Updates and stale documents

## User Story

> As a person, I want to see what changed since I last looked and which documents may be out of date, so that I find out about disputes and old facts without reading everything.

## Personas

- **Every person** — wants to know about disputes on their own documents.
- **Team lead** — checks a folder for stale documents.

## Behaviour

| Command | Task |
|---|---|
| `waymark updates [<folder>...] [--since <date>]` | What changed since a date. Without `--since`, it uses the time of the last `updates` call. Lists new, changed, disputed, and stale documents in the given folders, and changes by others to documents the person wrote. |
| `waymark stale [<folder>...] [--before <date>]` | Lists documents with no change since a date, grouped by folder. `--before` overrides the default date (today minus `stale_after_days`). With verification on, the `verified` date counts. |

- No folder means everything. Folders include subfolders.
- At read level `auto`, the skill reports `updates` at session start. At `manual`, it does not.
- The output is honest. It says "unverified" and "not checked" in plain words.
- `updates` reads Git history. It closes the gap that waymark cannot push to a person.

## Interfaces & Contracts

Both commands take `--kb <uuid>` and `--json`.

## Acceptance Criteria

- **Given** a document I wrote that someone disputed, **When** `updates` runs, **Then** it lists the dispute.
- **Given** no `--since`, **When** `updates` runs twice, **Then** the second run covers the time since the first.
- **Given** a folder, **When** `stale` runs, **Then** it lists only documents under it, grouped by subfolder.
- **Given** `stale_after_days = 180`, **When** a document has no change for 181 days, **Then** it is listed.
- **Given** read level `manual`, **When** a session starts, **Then** the skill does not report updates.

## Constraints & Edge Cases

- Agents run `dispute` if they find a contradiction.
- The check that flags documents whose anchored code changed comes after the first release.

## Open Questions

- Where is the "last updates call" time stored when several tools run `updates`?
