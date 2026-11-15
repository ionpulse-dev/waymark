---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-15
requires:
  - mcp-server
---

# Search and read

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The read budget on a shared server (an HTTP session) is out of scope for now (see `todo.md`). See [project decisions](../../../docs/decisions.md).

## User Story

> As a person with an agent, I want my agent to find the few documents that answer a question quickly and cheaply, so that it has the right context and room left for my code.

## Personas

- **Developer** — the agent looks up decisions and contracts.
- **Every person** — asks "what do we know about X?".

## Behaviour

| Tool | Task |
|---|---|
| `search` | Full-text search over path, title, description, tags, and body. Ranked. Without a query it lists documents sorted by `id` (see `lean-tools`). |
| `show` | Returns one document with its properties, body, version, backlinks, and replacement. |
| `stats` | Counts size and documents in folders. |

All three are read tools. They work when writes are paused or `access` is `read-only`.

### What the server enforces

- **Result caps.** `search` returns at most `max_results` documents (default 5) and `max_tokens` tokens (default 800). A call may ask for fewer, never for more. The token count is the estimate of section 9.1 of the standard over the text of the returned entries. If nothing matches, the result is empty and is not an error.
- **Read budget.** `show` opens at most `max_documents_opened` different documents per session (default 20). Opening the same document again costs nothing. After that, `show` fails with `access_denied`.
- **Folders.** The `folders` limit narrows every read. A document outside it is not found. A `folders` argument narrows the call more. No folder means the whole memory repository. A folder includes its subfolders.
- **Levels.** Level `0` returns only the `id`. Level `1` adds `title`, `status`, `updated`, and, for a disputed document, `disputed` and `dispute_reason`. Level `2` adds `description`. `search` returns a `snippet` at level 2 when it has a query.
- **Pull before read.** If the last good fetch is older than 3 minutes, a read fetches first. If that fails, the result carries a notice that the clone may be behind. The read still works.
- **No stored index.** Search reads the local files. It sees manual edits that are not committed yet.

The server does not decide when an agent reads. That is the choice of the team, in its `AGENTS.md`.

## Interfaces & Contracts

The filters and the paging of `search` are in [search-filters](../search-filters/requirements.md). `show` with a `version`, and `history`, are in [document-history](../document-history/requirements.md). The tags are in [manage-tags](../manage-tags/requirements.md).

Every result has `message`, `settings`, and optional `notices` (see `mcp-server`). The fields below are the tool's `data` fields at the top level of the result.

### `search`

| Input | Type | Required | Meaning |
|---|---|---|---|
| `query` | string | no | One or more words. A document must contain all words, in any order, in the searched parts. Case is ignored. Without it, `search` lists documents sorted by `id` (see `lean-tools`). |
| `in` | array of `path`, `title`, `description`, `tags`, `body` | no | Parts to search. Default: all five. |
| `level` | integer 0, 1, or 2 | no | Default 2 with a query, 1 without. |
| `folders` | array of string | no | Default: all. |
| `tag` | string | no | Only documents with this tag. |
| `status` | `active`, `deprecated`, or `any` | no | Default `active`. |
| `disputed`, `author`, `updated_after`, `updated_before` | | no | Filters. See `search-filters`. |
| `limit` | integer ≥ 1 | no | Fewer results than the cap. |
| `offset` | integer ≥ 0 | no | Default 0. Matches to skip. The result has `next_offset` when `truncated` is true (see `search-filters`). |

```json
{ "tool": "search", "arguments": { "query": "idempotency keys", "level": 2, "folders": ["backend"] } }
```

```json
{
  "message": "2 documents found.",
  "settings": { "access": "read-write", "max_results": 5, "max_tokens": 800 },
  "results": [
    {
      "id": "backend/payments/idempotency-keys",
      "title": "Payment API v3 uses idempotency keys",
      "status": "active",
      "updated": "2026-10-02",
      "description": "How clients send idempotency keys and how long the server keeps them.",
      "snippet": "... the server keeps a key for 24 hours ..."
    },
    {
      "id": "backend/payments/retries",
      "title": "Retry policy for payment calls",
      "status": "active",
      "updated": "2026-09-20",
      "disputed": true,
      "dispute_reason": "The retry limit changed in v3.",
      "description": "When a client retries a failed payment call.",
      "snippet": "..."
    }
  ],
  "total_matches": 2,
  "truncated": false
}
```

`total_matches` counts all matches. `truncated` is `true` when a cap cut the list.

### `show`

| Input | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | Document ID. |
| `version` | string | no | A version from `history`. The result is the document at that version (see `document-history`). Default: the current version. |

The result has `is_current`: `true` for the current version. For an older version it has no `backlinks`, `replaced_by`, `pending_edit`, or `moved_from`, and it has `current_version`.

```json
{
  "message": "Document found.",
  "settings": { "access": "read-write" },
  "id": "backend/payments/idempotency-keys",
  "title": "Payment API v3 uses idempotency keys",
  "description": "How clients send idempotency keys and how long the server keeps them.",
  "status": "active",
  "created": "2026-08-01",
  "updated": "2026-10-02",
  "age_days": 43,
  "created_by": "Jane Doe <jane@example.org>",
  "updated_by": "Jane Doe <jane@example.org>",
  "last_editor": "Jane Doe <jane@example.org>",
  "disputed": false,
  "tags": ["payments"],
  "sources": [],
  "related": ["backend/payments/retries"],
  "supersedes": [],
  "anchors": [],
  "body": "# Idempotency keys\n...",
  "version": "a1b2c3d",
  "backlinks": ["backend/payments/overview"],
  "backlinks_total": 1,
  "replaced_by": [],
  "pending_edit": false,
  "moved_from": null
}
```

- `version` is the short hash of the last commit of the file. A write sends it as `base`.
- `age_days` counts whole days from `updated` to today.
- `last_editor` is the author of the last commit of the file.
- A disputed document has `disputed: true` and `dispute_reason`.
- `replaced_by` lists the documents whose `supersedes` names this one. It is empty for an active document.
- `backlinks` lists at most 50 IDs. `backlinks_total` is the full count.
- `pending_edit` is `true` when the file has changes that are not committed (see `manual-edits`).
- Optional properties (`trust`, `verified_by`, `verified`) appear when the document has them.
- If `id` does not exist but Git history shows that the document moved, `show` returns the document at its new ID, sets `moved_from` to the old ID, and adds a notice.

### `search` without a query

Since 0.5.0 the tool `list` is a mode of `search`: a call with no `query` lists the documents that pass the filters, sorted by `id`, at level 1 by default, with the same status default `active`, and the same paging (`offset`, `next_offset`). The message is "2 of 7 documents.". A call named `list` still works until 0.6.0 (see [lean-tools](../lean-tools/requirements.md)).

### `stats`

| Input | Type | Required | Meaning |
|---|---|---|---|
| `folders` | array of string | no | Default: all. |
| `metric` | `words`, `tokens`, `chars`, or `documents` | no | Default `words`. |
| `by` | `none`, `folder`, or `author` | no | Default `none`. `folder` groups by the first folder level below each given folder. `author` groups by `created_by`. |

`words` counts the title, the description, and the body (section 9.1). `tokens` is the estimate. `chars` counts the characters of the whole file. `documents` counts files.

```json
{
  "message": "42 documents, 31200 words.",
  "settings": {},
  "metric": "words",
  "total": 31200,
  "documents": 42,
  "groups": [ { "key": "backend", "value": 20100, "documents": 25 } ],
  "groups_truncated": false
}
```

`groups` is sorted by `value`, largest first, and holds at most 50 entries.

### Errors

`invalid_arguments` (empty `query`, bad `level`, unknown `in` part), `not_found` (no document with this ID, or outside the `folders` limit), `access_denied` (read budget used up), `setup_problem` (no memory repository).

## Acceptance Criteria

- **Given** a query with no match, **When** `search` runs, **Then** it returns an empty list and no error.
- **Given** more matches than the cap, **When** `search` runs, **Then** it returns at most 5 documents within 800 tokens, and `truncated` is true.
- **Given** a call that asks for 50 results, **When** `search` runs, **Then** it returns at most the cap.
- **Given** a document that matches in the title and one that matches only in the body, **When** `search` runs, **Then** the title match ranks first.
- **Given** the read budget is used up, **When** `show` runs for a new document, **Then** it fails with `access_denied` and says why.
- **Given** a document already opened, **When** `show` runs for it again after the budget is used up, **Then** it works.
- **Given** `level` 0, **When** `search` runs with no query, **Then** each entry is only the ID.
- **Given** a document that others link to, **When** `show` runs, **Then** it lists the backlinks.
- **Given** a disputed document, **When** `show` runs, **Then** it shows the `dispute_reason`.
- **Given** a deprecated document that another document supersedes, **When** `show` runs, **Then** `replaced_by` names the replacement.
- **Given** the limit `folders = ["engineering/**"]`, **When** `search` runs, **Then** no document outside it appears, and `show` of one fails with `not_found`.
- **Given** a folder, **When** `stats` runs with `by: "author"`, **Then** it counts per `created_by`.
- **Given** a clone that is more than 3 minutes behind, **When** `search` runs, **Then** it fetches first and finds a document from the remote.
- **Given** a failed fetch, **When** `search` runs, **Then** it answers from the local clone with a notice that the clone may be behind.
- **Given** a document that moved, **When** `show` runs with the old ID, **Then** it returns the document and `moved_from`.

## Constraints & Edge Cases

- Documents are local files, so search takes milliseconds. Up to some thousand files, a plain scan is enough. No vector database and no stored index at the start.
- The read budget is state of one session. It resets when the session ends.
- A file that cannot be parsed is skipped by `search` and `stats`. `lint` reports it. `show` fails with `validation_failed` and names the first finding.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`).
