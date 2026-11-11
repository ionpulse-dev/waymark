---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - write-documents
---

# Activity and undo

## User Story

> As a person, I want to see everything my agents wrote under my name and undo any write, so that letting my agent write on its own feels safe.

## Personas

- **Every person who lets agents write freely** — relies on undo as the safety net.
- **Knowledge base owner** — in server mode, sees the activity of everyone.

## Behaviour

| Tool | Task |
|---|---|
| `activity` | Lists the recent writes under the person's account, newest first: the commit, the title, the folder, the time, and the tool. Input: optional `limit` and `since`. Read-only. |
| `undo` | Reverts one write. Input: the commit from `activity`. It makes a revert commit under the person's account and pushes it. A write tool, so the limits apply. |

- The list comes from Git history of the knowledge base, filtered by the author email of the person.
- An undo is a new commit. History stays.
- If the document changed after the write, `undo` stops with `stale_base` and shows what would be reverted. It does not overwrite newer changes silently. The call can be repeated with `force: true` after the person agreed.
- An undo of a move reverts the move and the link rewrites together.
- In server mode, the console shows the same list with an **Undo** button (see `web-console`). Owners see all people.
- A team that wants undo only by people switches the tool off for agents with `tools_denied = ["undo"]` and uses the console on a server.

## Interfaces & Contracts

```json
{ "tool": "activity", "arguments": { "limit": 10 } }
{ "tool": "undo", "arguments": { "commit": "<hash>", "force": false } }
```

## Acceptance Criteria

- **Given** an agent wrote a document, **When** `activity` runs, **Then** the document is first in the list.
- **Given** `undo` runs, **When** the revert succeeds, **Then** a new commit under my account reverts the write and is pushed.
- **Given** a later change by someone else to the same document, **When** `undo` runs without `force`, **Then** it fails with `stale_base`.
- **Given** writes by other people, **When** `activity` runs, **Then** they do not show.
- **Given** `access = "read-only"`, **When** `undo` runs, **Then** it fails with `access_denied`.

## Constraints & Edge Cases

- An undo cannot remove content from Git history. A secret that was pushed must be rotated (see `trust-model`).

## Open Questions

- How far back does the list go?
