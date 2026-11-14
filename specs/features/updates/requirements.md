---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - search-and-read
---

# Updates

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Per-person state on a server is out of scope for now (see `todo.md`). See [project decisions](../../../docs/decisions.md).

## User Story

> As a person, I want to see what changed since I last looked, so that I find out about disputes and changes by others without reading everything.

## Personas

- **Every person** — wants to know about disputes on their own documents.
- **Team lead** — checks what changed in a folder.

## Behaviour

| Tool | Task |
|---|---|
| `updates` | Lists the documents that others added, changed, moved, deprecated, disputed, or deleted since a time. A read tool. |

- Without `since`, `updates` uses the time of the last `updates` call without `since` for this person and memory repository. If there is none, it uses 7 days ago. A call without `since` then stores the time of this call.
- A call with `since` only reads. It does not move the stored time.
- Changes by the person themselves are not listed. The person knows them (see `activity` for their own writes).
- A document is "mine" when `created_by` has the email of the person.
- No folder means everything. Folders include subfolders. The `folders` limit applies.
- It reads Git history, after a fetch if the last good fetch is older than 3 minutes. It lists only committed changes.
- The output is honest. The message says that waymark did not check the changes for truth. A document with a `trust` property shows it.
- The same facts that matter most arrive as `notices` in other results (see `notifications-and-review`). When an agent runs `updates` is the choice of the team.
- The stored time is one small file for each person and memory repository. All agent tools of the person on this machine share it. The last call without `since` wins.

## Interfaces & Contracts

| Input | Type | Required | Meaning |
|---|---|---|---|
| `folders` | array of string | no | Default: all. |
| `since` | string | no | A date `YYYY-MM-DD` (start of that day, UTC) or a time in RFC 3339. |

```json
{ "tool": "updates", "arguments": { "folders": ["backend"], "since": "2026-11-10" } }
```

```json
{
  "message": "3 changes since 2026-11-10. 1 of your documents is disputed. waymark did not check these changes for truth.",
  "settings": { "access": "read-write" },
  "since": "2026-11-10T00:00:00Z",
  "until": "2026-11-14T09:30:00Z",
  "changes": [
    {
      "id": "backend/payments/retries",
      "title": "Retry policy for payment calls",
      "kind": "disputed",
      "mine": true,
      "by": "Alex Roe <alex@example.org>",
      "date": "2026-11-12",
      "reason": "The retry limit changed in v3."
    },
    {
      "id": "ops/deploy",
      "title": "How we deploy",
      "kind": "changed",
      "mine": false,
      "by": "Alex Roe <alex@example.org>",
      "date": "2026-11-11"
    },
    {
      "id": "ops/rollback",
      "title": "How we roll back",
      "kind": "moved",
      "from": "ops/undo",
      "mine": false,
      "by": "Alex Roe <alex@example.org>",
      "date": "2026-11-11"
    }
  ],
  "total": 3,
  "truncated": false,
  "marked": false
}
```

- `kind` is one of `new`, `changed`, `moved`, `deprecated`, `disputed`, `deleted`. A document that is both changed and newly disputed is `disputed`. A new document stays `new`, whatever later changes.
- `reason` is present for `disputed`.
- `from` is present for `moved`.
- `by` and `date` are the author and day of the last change in the window by someone else.
- Order: documents that are `mine` and `disputed`, then other `mine` documents, then the rest. Inside a group, newest first.
- At most 50 entries. `total` is the full count. `truncated` is true when the list was cut.
- `marked` is `true` when this call stored its time (a call without `since`).

### Errors

`invalid_arguments` (a `since` that is not a date or a time, or in the future), `setup_problem`.

## Acceptance Criteria

- **Given** a document I wrote that someone disputed, **When** `updates` runs, **Then** it lists the dispute with its reason.
- **Given** no `since`, **When** `updates` runs twice, **Then** the second run covers only the time since the first.
- **Given** a call with `since`, **When** it ends, **Then** the stored time does not change.
- **Given** a folder, **When** `updates` runs, **Then** it lists only changes to documents under it.
- **Given** a document outside the `folders` limit, **When** `updates` runs, **Then** it is not listed.
- **Given** a change that I made myself, **When** `updates` runs, **Then** it is not listed.
- **Given** a document that someone moved, **When** `updates` runs, **Then** it is listed as `moved` with `from`.
- **Given** a new document by someone else, **When** `updates` runs, **Then** it is listed as `new`.
- **Given** a `since` that is not a date, **When** `updates` runs, **Then** it fails with `invalid_arguments`.
- **Given** a change on the remote that the clone does not have, **When** `updates` runs, **Then** it fetches first and lists the change.

## Constraints & Edge Cases

- The check that flags documents whose anchor changed comes after the first release (see `docs/ideas.md`).
- A repository that is younger than `since` lists every document as `new`.
- Manual edits that are not committed are not listed (see `manual-edits`).

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`).
