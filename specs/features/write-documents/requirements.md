---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Write documents

## User Story

> As a person, I want my agent to write, correct, dispute, replace, and retire documents safely, so that the knowledge base stays accurate and nothing I own is overwritten by mistake.

## Personas

- **Developer** — the agent records a decision or a pitfall.
- **Every person** — the agent corrects or disputes a document that is wrong.

## Behaviour

| Tool | Task |
|---|---|
| `add` | Creates a document. Input: folder, title, description, sources, tags, and the body. waymark creates the frontmatter, picks a unique file name (a slug of the title, plus a short ID if needed), places the file in `knowledge/`, validates it, and syncs. With verification on, it sets `trust: unverified`. Optional `supersedes`: the ID of the document that this one replaces. |
| `update_document` | Changes a document. Input: the ID, the `base` version from `show`, and the changed fields or body. Validates and syncs. |
| `dispute` | Sets `disputed: true` and `dispute_reason`. Input: the ID and a reason. With `clear: true` it ends a dispute and removes both fields. |
| `deprecate` | Sets `status: deprecated` and keeps the file. Input: the ID and a reason. Used when a document is no longer true and nothing replaces it. |
| `delete` | Deletes a document. It searches for links to the document, in bodies and in properties, fixes them in the same commit, and tells the person that the file stays in the Git history. |

`add` sets all four author fields (`created`, `created_by`, `updated`, `updated_by`). The other tools set `updated` and `updated_by` for every document they change.

A replacement (`add` with `supersedes`) puts the new document and the deprecation of the old one in one commit. The new document names the old one in `supersedes`. The old one stores no link to the new one.

All write tools follow the write flow of `git-sync`: limits, lock, fetch, base check, checks, approval, commit under the author, push. They accept `preview: true`.

### What the server enforces

Every write is checked before the commit. A failed check ends the call with `validation_failed` and a `next` sentence that tells the agent what to do.

| Check | Example of the `next` sentence |
|---|---|
| Frontmatter schema, required fields, and file name | "Add a description of at most 50 words." |
| Size caps (`documents.max_bytes`, title and description words) | "The document is over the cap. Split it, or move details into a source." |
| Duplicate ID or title, and a near-duplicate title | "A document with this title exists: `ops/deploy`. Update it instead." |
| Broken links | "The link to `ops/old` has no target." |
| Secrets and personal data patterns | "The text looks like it contains a secret. Remove it." |
| Folder allowed by the `folders` limit | "This folder is outside the allowed folders." |

The server does not judge what deserves a document, and it does not decide when to write. Those are choices of the team, in its `AGENTS.md`.

Limits apply (see `trust-model`): `access`, `paused`, `tools_denied`, and `approval.required_for`. For example, a team can switch off `delete` completely.

## Interfaces & Contracts

```json
{ "tool": "add", "arguments": { "folder": "backend/payments", "title": "...", "description": "...", "body": "...", "sources": [], "tags": [], "preview": false } }
{ "tool": "update_document", "arguments": { "id": "backend/payments/idempotency-keys", "base": "<hash>", "body": "..." } }
```

Errors: `validation_failed`, `sync_failed`, `stale_base`, `access_denied`, `approval_denied`, `approval_unavailable`, `paused`.

## Acceptance Criteria

- **Given** a valid input, **When** `add` runs, **Then** one commit under the author reaches the remote and the document has all required fields.
- **Given** a body with a secret pattern, **When** `add` runs, **Then** it fails with `validation_failed` and writes nothing.
- **Given** a title that already exists, **When** `add` runs, **Then** it fails as a duplicate and names the existing document.
- **Given** a changed remote document, **When** `update_document` runs with the old `base`, **Then** it fails with `stale_base`.
- **Given** `preview: true`, **When** `add` runs, **Then** nothing is written or committed.
- **Given** `dispute`, **When** it runs, **Then** `disputed` is true and the reason is saved.
- **Given** `deprecate`, **When** it runs, **Then** the file still exists with `status: deprecated`.
- **Given** `dispute` with `clear: true`, **When** it runs, **Then** `disputed` and `dispute_reason` are gone and `updated` is set.
- **Given** `add` with `supersedes`, **When** it runs, **Then** one commit holds the new document and the old document with `status: deprecated`.
- **Given** `delete` of a document that others link to, **When** it runs, **Then** the file is gone and no link to it is left.
- **Given** `tools_denied = ["delete"]`, **When** the client lists tools, **Then** `delete` is not listed.
- **Given** `access = "read-only"`, **When** any write tool runs, **Then** it fails with `access_denied` and no commit is made.
- **Given** `approval.required_for = ["add"]`, **When** `add` runs, **Then** the approval is asked before the commit (see `write-approval`).

## Constraints & Edge Cases

- A document over the size cap fails with a message to split it.
- The commit message has a fixed format and no agent tag.
- Choose the lightest tool that fits: fix, dispute, deprecate, replace, delete. This is advice for a team's `AGENTS.md`, not a rule of the server. The server only makes `delete` easy to switch off.

## Open Questions

- Should `update_document` accept a partial frontmatter update without sending the body?
- How does the server detect a near-duplicate title: by a simple word overlap, or something else?
