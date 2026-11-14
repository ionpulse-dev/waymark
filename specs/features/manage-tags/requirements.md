---
type: feature
status: implemented
created: 2026-11-15
updated: 2026-11-15
requires:
  - write-documents
  - search-and-read
  - tool-conventions
---

# Manage tags

## User Story

> As a memory owner, I want to see which tags exist and to rename or merge a tag in all documents at once, so that tags do not drift into near-duplicates.

## Personas

- **Memory owner** — keeps the tags tidy.
- **Developer** — looks for the tags that exist before tagging a new document.

## Behaviour

- `list_tags` returns the tags with the number of documents for each.
- `rename_tag` changes a tag in every document that has it, in one commit. When the new tag already exists on a document, the two merge.

## Interfaces & Contracts

### `list_tags`

| Parameter | Meaning |
|---|---|
| `folders` | Only these folders. |
| `status` | `active` (default), `deprecated`, or `any`. |
| `limit` | How many tags. Default 50, at most 200. |
| `offset` | How many tags to skip. |

**Returns:** `tags` (each `{tag, documents}`, most documents first, then by name), `total_tags`, `truncated`, and `next_offset` when truncated.

### `rename_tag`

| Parameter | Meaning |
|---|---|
| `from` | The tag to change. Required. |
| `to` | The new tag. Required. No `#`, no spaces, not empty. |
| `preview` | `true` shows the documents and writes nothing. |

**Returns:** `from`, `to`, `documents` (the number changed), `ids` (up to 20), `commit`, `version`, `pushed`, and `preview`.

**Errors:** `invalid_arguments` for a bad tag or `from` equal to `to`; `not_found` when no visible document has `from`; `validation_failed` when a changed document would fail the checks (the error names it, and nothing changes).

## Acceptance Criteria

- **Given** documents with tags, **When** `list_tags` is called, **Then** each tag is listed once with its document count, most used first.
- **Given** a tag on three documents, **When** `rename_tag` is called, **Then** the three documents have the new tag, `updated` and `updated_by` change, and there is one commit.
- **Given** a document that has both tags, **When** `rename_tag` is called, **Then** the document has the new tag once.
- **Given** a tag that no document has, **When** `rename_tag` is called, **Then** it fails with `not_found`.
- **Given** a bad new tag or the same tag, **When** `rename_tag` is called, **Then** it fails with `invalid_arguments`.
- **Given** the `folders` limit, **When** `list_tags` or `rename_tag` runs, **Then** documents outside the limit are not counted and not changed.
- **Given** `preview: true`, **When** `rename_tag` is called, **Then** nothing is written and the result lists the documents that would change.
- **Given** read-only access, a pause, or `tools_denied`, **When** `rename_tag` is called, **Then** it is denied, and `undo` reverts a rename in one step.

## Constraints & Edge Cases

- A tag matches exactly (case sensitive), as in `list` and `search`.
- The body of a document is never changed.
- `rename_tag` is a write tool. It asks for approval when the limits require it for write tools.

## Open Questions

None.
