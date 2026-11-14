---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - write-documents
  - git-sync
---

# Activity and undo

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The console and the view of all people are out of scope for now (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a person, I want to see everything my agents wrote under my name and undo any write, so that letting my agent write on its own feels safe.

## Personas

- **Every person who lets agents write freely** — relies on undo as the safety net.

## Behaviour

| Tool | Kind | Task |
|---|---|---|
| `activity` | read | Lists the recent writes under the person's account, newest first. |
| `undo` | write | Reverts one write with a revert commit and pushes it. |

- The list comes from the Git history of the memory repository. It holds the commits whose author email is the email of the person (compared without case) and whose subject starts with `waymark: `. This includes the verbs `add`, `update`, `dispute`, `deprecate`, `delete`, `move`, `undo`, `sync`, and `init`.
- An undo is a new commit. History stays.
- A write that changes files that later commits also changed cannot be undone by `undo`, unless Git can revert it cleanly (see below). `undo` never overwrites newer changes silently, and it has no `force`.

### `activity`

Input:

| Field | Type | Required | Meaning |
|---|---|---|---|
| `limit` | integer | no | Default 10, at most 50. |
| `since` | string | no | A date `YYYY-MM-DD`. Older entries are left out. |
| `before` | string | no | A commit from an earlier result. Only older entries show. Use it to read further back. |

The list goes back as far as `limit`, `since`, and `before` say. There is no other limit. An answer has no entry for a commit that made no change to a document file.

Result `data`:

```json
{
  "entries": [
    {
      "commit": "a1b2c3d",
      "time": "2026-11-14T09:30:12+01:00",
      "verb": "add",
      "title": "Payment API v3 uses idempotency keys",
      "folder": "backend/payments",
      "ids": ["backend/payments/idempotency-keys"],
      "files": 1,
      "undone": false,
      "later_changes": false
    }
  ],
  "next_before": "9f8e7d6"
}
```

- `folder` is the folder of the first changed document. `ids` lists up to 5 IDs, `files` is the full count.
- `undone` is `true` if an `undo` commit reverted this one. A reverted undo (a redo) makes `undone` `false` again.
- `later_changes` is `true` if a later commit by anyone changed one of the same files. `undo` may then refuse.
- A move shows as one entry with `verb: "move"`. Its link-fix commit (see `move-documents`) is part of it and has no entry of its own. `files` counts both commits.
- `next_before` is the commit to send as `before` for the next page. It is `null` at the end of the history.

### `undo`

Input:

| Field | Type | Required | Meaning |
|---|---|---|---|
| `commit` | string | yes | A commit from `activity`. 7 to 40 hex characters. |
| `preview` | boolean | no | Default `false`. |

Rules, in this order:

1. The commit must be in the activity of the person. Otherwise the call fails with `not_found`: "No write with this commit is in your activity. Run `activity`." A commit of another person is the same case.
2. If a later `undo` commit already reverted it and nobody reverted that undo, the call fails with `validation_failed`: "This write was already undone by `<commit>`."
3. If pending manual edits touch the same files, the call fails with `stale_base`. `next`: "Run `sync` first."
4. If no later commit changed the same files, the revert is clean. If later commits did, the tool tries the revert. A clean result goes on. A conflict ends with `stale_base`. `message` names the later commits (hash, author, title) and `next` says "Change the document with `update_document` instead."
5. The files must match the `folders` limit, or the call fails with `access_denied`.
6. The revert is one commit. The message is `waymark: undo "<title>"` with the title of the undone commit, then an empty line and `Reverts: <hash>`. The author is the person. The commit is pushed (write flow of `git-sync`).
7. An undo of a move reverts the move commit and its link-fix commit together, in one commit. The `Reverts:` line lists both hashes, separated by a space.

The revert restores the earlier state exactly. It does not run the content checks of `write-documents`. For example, the undo of a `delete_document` restores the document even if its links or title now clash with newer documents. `lint` reports such problems.

With `preview: true` the tool runs rules 1 to 5 and returns what would change. Nothing is committed.

Result `data`:

```json
{
  "commit": "e4f5a6b",
  "reverted": ["a1b2c3d"],
  "title": "Payment API v3 uses idempotency keys",
  "changes": [ { "id": "backend/payments/idempotency-keys", "action": "removed" } ],
  "pushed": true,
  "preview": false
}
```

`action` is `removed` (an added file), `restored` (a deleted or moved-away file), or `reverted` (a changed file). With `preview` the `commit` and `pushed` values are `null`.

### Errors

| Code | When |
|---|---|
| `not_found` | The commit is not in the activity of the person. |
| `invalid_arguments` | `commit` is missing or not a hash. |
| `stale_base` | Later commits or pending edits block a clean revert. |
| `validation_failed` | The write was already undone. |
| `access_denied`, `paused`, `approval_denied`, `approval_unavailable`, `sync_failed` | As in `mcp-server` and `git-sync`. |

## Acceptance Criteria

- **Given** an agent wrote a document, **When** `activity` runs, **Then** the document is first in the list.
- **Given** `undo` runs, **When** the revert succeeds, **Then** a new commit under my account reverts the write and is pushed.
- **Given** a later change by someone else to the same document, **When** `undo` runs and the revert conflicts, **Then** it fails with `stale_base` and nothing changes.
- **Given** a later change by someone else to another part of the same document, **When** `undo` runs, **Then** Git reverts cleanly and the later change stays.
- **Given** writes by other people, **When** `activity` runs, **Then** they do not show.
- **Given** `access = "read-only"`, **When** `undo` runs, **Then** it fails with `access_denied`.
- **Given** a move, **When** `undo` runs with its commit, **Then** one new commit reverts the move and the link fixes.
- **Given** an undone write, **When** `activity` runs, **Then** the entry has `undone: true`.
- **Given** a write that was already undone, **When** `undo` runs again, **Then** it fails with `validation_failed`.
- **Given** 12 writes and `limit: 5`, **When** `activity` runs and then runs again with `before: next_before`, **Then** the second answer holds the next 5 older writes.
- **Given** `preview: true`, **When** `undo` runs, **Then** nothing is committed.

## Constraints & Edge Cases

- An undo cannot remove content from Git history. A secret that was pushed must be rotated (see `trust-model`).
- `undo` only undoes writes of the person who calls it. A person who needs to revert another person's write uses `update_document` or plain Git.
- Commits that are not made by waymark (a plain Git commit) do not show.
- A team that wants no undo by agents switches the tool off with `tools_denied = ["undo"]`.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`): how far back the list goes, and what happens when later commits touch the same file.
