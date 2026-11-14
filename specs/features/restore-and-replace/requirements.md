---
type: feature
status: implemented
created: 2026-11-15
updated: 2026-11-15
requires:
  - write-documents
  - tool-conventions
---

# Restore and replace documents

## User Story

> As a developer, I want to bring a deprecated document back, and to say later that a document replaces an older one, so that a mistake in the lifecycle of a document is easy to fix.

## Personas

- **Developer** — deprecated a document by mistake, or wrote the replacement after the old document.
- **Memory owner** — keeps the memory current.

## Behaviour

- A new tool, `restore_document`, sets a deprecated document back to `active`. It keeps the file and the history.
- `update_document` takes `supersedes`. It names the document that this one replaces. The replaced document becomes `deprecated`, like when `add_document` is called with `supersedes`.

## Interfaces & Contracts

### `restore_document`

| Parameter | Meaning |
|---|---|
| `id` | The document. Required. |
| `base` | The version from `show`. Required. |
| `reason` | Optional. One line. It goes into the commit message. |
| `preview` | `true` shows the change and writes nothing. |

**Returns:** `id`, `title`, `status` (`active`), `commit`, `version`, `pushed`, and `preview`.

**Errors:** `not_found`; `validation_failed` when the document is already active; `stale_base` for an old `base`.

**Notice:** when another active document lists this one in `supersedes`, the result says so.

### `update_document` with `supersedes`

| Parameter | Meaning |
|---|---|
| `supersedes` | ID of a document that this one replaces. Added to the `supersedes` list. |

**Returns:** the usual result. `deprecated` lists the IDs that were deprecated.

**Errors:** `invalid_arguments` when the ID is the document itself; `not_found` when it does not exist or a limit hides it; `validation_failed` when the other document already replaces this one.

## Acceptance Criteria

- **Given** a deprecated document, **When** `restore_document` is called with the current `base`, **Then** the status is `active`, `updated` and `updated_by` change, and there is one commit.
- **Given** an active document, **When** `restore_document` is called, **Then** it fails with `validation_failed`.
- **Given** an old `base`, **When** `restore_document` is called, **Then** it fails with `stale_base` and nothing changes.
- **Given** a deprecated document that an active document replaces, **When** it is restored, **Then** the call works and the result has a notice that names the replacing document.
- **Given** a memory repository with verification, **When** a document is restored, **Then** its trust is `unverified`.
- **Given** two documents, **When** `update_document` is called with `supersedes` for the older one, **Then** the newer one lists it, the older one is `deprecated`, and there is one commit.
- **Given** the same document, a missing document, or a loop, **When** `update_document` is called with `supersedes`, **Then** it fails and nothing changes.
- **Given** read-only access, a pause, `tools_denied`, or a `folders` limit, **When** `restore_document` is called, **Then** the limit applies as for the other write tools, and `undo` reverts the restore.

## Constraints & Edge Cases

- `restore_document` is a write tool, not a destructive tool.
- A restore never changes the body or the other properties.
- With `preview: true`, nothing is written and no approval is asked.

## Open Questions

None.
