---
type: feature
status: implemented
created: 2026-11-15
updated: 2026-11-15
requires:
  - search-and-read
  - tool-conventions
---

# Document history

## User Story

> As a developer, I want to see how a document changed and to read an older version, so that I can judge a change before I edit, undo, or dispute it.

## Personas

- **Developer** — wants to know who changed a document and why.
- **Memory owner** — reviews changes and decides what to undo.

## Behaviour

- A new read tool, `history`, lists the versions of one document, newest first.
- `show` takes `version`. It returns the document as it was at that version.

## Interfaces & Contracts

### `history`

| Parameter | Meaning |
|---|---|
| `id` | The document. Required. |
| `limit` | How many versions. Default 10, at most 50. |
| `before` | A version from an earlier result. Shows older versions. |

**Returns:** `id`, `entries`, and `next_before` (a version, or `null`). An entry has `version`, `time`, `author` (`Name <email>`), `verb` (the waymark verb, or `null` for a change by hand), and `subject` (the commit subject). The history follows a move of the document.

**Errors:** `not_found` for an unknown ID or a hidden one; `invalid_arguments` for a `before` that is not a version of this document, or a `limit` below 1.

### `show` with `version`

| Parameter | Meaning |
|---|---|
| `version` | A version from `history`. |

**Returns:** the same fields as `show`, except `backlinks`, `backlinks_total`, `replaced_by`, `pending_edit`, and `moved_from`. Instead it has `is_current` (`true` when the version is the current one). For an old version, `version` is the asked version and `current_version` is the current one.

**Errors:** `invalid_arguments` when the version is not a version of this document.

## Acceptance Criteria

- **Given** a document with three changes, **When** `history` is called, **Then** it returns three entries, newest first, each with version, time, author, verb, and subject.
- **Given** more versions than `limit`, **When** `history` is called with the `next_before` of the result, **Then** it returns the older versions, and none twice.
- **Given** a document that was moved, **When** `history` is called, **Then** the versions from before the move are listed.
- **Given** an old version, **When** `show` is called with it, **Then** the title, description, and body are those of that version, and `is_current` is `false`.
- **Given** the current version, **When** `show` is called with it, **Then** the result equals a call without `version`, with `is_current` `true`.
- **Given** an unknown document, a hidden document, or a version of another document, **When** either tool is called, **Then** it fails with `not_found` or `invalid_arguments`.
- **Given** `show` with an old version, **When** the read budget counts, **Then** the document counts once.

## Constraints & Edge Cases

- Both tools read Git history. They change nothing.
- A document that was deleted is `not_found`. Its history is in Git.
- The `folders` limit applies, as for `show`.

## Open Questions

None.
