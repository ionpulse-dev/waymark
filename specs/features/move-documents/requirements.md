---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - write-documents
  - git-sync
---

# Move documents

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. This spec has no server-mode part (see [project decisions](../../../docs/decisions.md)).

## User Story

> As an organization, I want to move a document or a whole team folder after a reorganization, so that all links keep working and the history of every document stays.

## Personas

- **Memory owner** — reorganizes the folder tree.
- **Every person** — keeps links that work.

## Behaviour

The `move` tool moves or renames a document or a folder and rewrites all links to it, in bodies and in properties. A rename is a move: it changes the ID. When to move is the choice of the team, in its `AGENTS.md`. A team that wants no moves by agents switches the tool off with `tools_denied`.

`move` is a write tool and follows the write flow of `git-sync`. It does not take `base`, because a move changes no content.

### Input

| Field | Type | Required | Meaning |
|---|---|---|---|
| `from` | string | yes | The ID of a document, or a folder path with a trailing `/`. |
| `to` | string | yes | The new ID, or the new folder path with a trailing `/`. Same kind as `from`. |
| `preview` | boolean | no | Default `false`. |

- A document ID has no `.md`. A folder path ends with `/`. If `from` is a folder, `to` must be a folder path too.
- Every segment of `to` follows the file name rules of `document-format`. New folders are created.
- `to` must not exist. A folder is never merged into an existing folder.
- A folder cannot move into itself.
- The root folder and `.waymark/` cannot move.
- `from` and `to` (and the old and new ID of every moved document) must match the `folders` limit, or the call fails with `access_denied`. This holds for every document that needs a link fix too.

### Steps

1. Move with `git mv`: the whole folder, with all files in it, or the one document.
2. **Commit 1: the move alone.** The message is `waymark: move "<title>"`. For a folder, the title is the new folder path with `/`. The commit holds renames only and no content change.
3. **Commit 2: the link fixes.** The message is `waymark: update "<title>"` with the same title, then an empty line and `Move: <hash of commit 1>`. It holds only link rewrites. `updated` and `updated_by` are set in each document that changed. If nothing needs a fix, there is no second commit.
4. Check that `git log --follow` on the new path of each moved document reaches the old history. This runs before the push. If the check fails, the local commits of this call are removed, nothing is pushed, and the call fails with `validation_failed`.
5. Push both commits together.

### Link rewrites

| Link | Rewrite |
|---|---|
| Relative Markdown link to a moved document, in any document | The path is recomputed from the new location of the linking document to the new location of the target. |
| Relative Markdown link in a moved document to a document that did not move | The path is recomputed from the new location. |
| Full-path wikilink in a property or in the body | The ID is replaced. Alias and `#heading` stay. |
| Wikilink with a short name, in the body | Replaced by the full new path only if the short name no longer finds the document. |
| `sources` and `anchors` | Not changed. |
| Links in code fences and inline code | Not changed. |

Links between documents that move together stay valid and are not rewritten, unless the folder depth between them changed.

### Result

```json
{
  "kind": "folder",
  "from": "backend/payments/",
  "to": "platform/payments/",
  "moved": [ { "from": "backend/payments/idempotency-keys", "to": "platform/payments/idempotency-keys" } ],
  "moved_total": 42,
  "links_updated": [ { "id": "ops/deploy", "count": 2 } ],
  "commits": ["a1b2c3d", "e4f5a6b"],
  "pushed": true,
  "history_checked": true,
  "preview": false
}
```

`kind` is `document` or `folder`. `moved` shows at most 100 entries. `moved_total` is the full count. With `preview: true`, `commits`, `pushed`, and `history_checked` are `null`, and nothing changes. The `notices` always say: "Links from outside this memory repository are not checked."

If approval is required (see `write-approval`), the request shows a summary, for example "Move 42 documents, update links in 17 documents".

### Errors

| Code | When |
|---|---|
| `not_found` | `from` does not exist, or the folder does not exist. A folder with only a `.gitkeep` can move. |
| `validation_failed` | `to` exists, is inside `from`, has a bad name, or the history check failed. |
| `invalid_arguments` | `from` and `to` are of different kinds, are equal, or `to` ends with `.md`. |
| `access_denied`, `paused`, `approval_denied`, `approval_unavailable`, `sync_failed` | As in `mcp-server` and `git-sync`. |

## Acceptance Criteria

- **Given** a folder with 42 documents, **When** it is moved, **Then** all links to them in other documents work after the move.
- **Given** a moved document, **When** `git log --follow` runs on the new path, **Then** the old history shows.
- **Given** a move, **When** the commits are inspected, **Then** the first commit contains renames only and the second contains link rewrites only.
- **Given** a move into an existing path, **When** it runs, **Then** it fails with `validation_failed`.
- **Given** `tools_denied = ["move"]`, **When** the client lists tools, **Then** `move` is not listed.
- **Given** `preview: true`, **When** `move` runs, **Then** the list of changes is shown and nothing changes.
- **Given** a rename of one document, **When** it runs, **Then** the ID changes and links to the old ID are rewritten.
- **Given** a move with no inbound links, **When** it runs, **Then** exactly one commit is made.
- **Given** `folders = ["engineering/**"]`, **When** a document moves out of it, **Then** the call fails with `access_denied`.

## Constraints & Edge Cases

- Git does not store moves. It detects a rename only if the old and new file are similar (about 50%). Commit 1 is a pure rename, so the similarity is 100 percent.
- Reorganizations need no `owners` field update, because the folder is the owner.
- Two files with identical content can confuse `git log --follow`. The history check catches this and the call fails.
- A move never changes the content of a document in commit 1. A move and a content change are never in one commit.
- Calling the same move twice: the second call fails with `not_found`, and the repository is as after the first call.
- `show` with an old ID says where a document moved. This is part of `search-and-read` and uses `git log --follow` on the commits that this tool makes.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`): links from outside the memory repository are not checked, and a fixed notice says so.
