---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-09
requires:
  - write-documents
---

# Move documents

## User Story

> As an organization, I want to move a document or a whole team folder after a reorganization, so that all links keep working and the history of every document stays.

## Personas

- **Knowledge base owner** — reorganizes the folder tree.
- **Every person** — keeps links that work.

## Behaviour

`waymark move <old> <new>` moves or renames a folder or a document and rewrites all links to it, in the body and in properties. A rename is a move: it changes the ID. Do it only when the person asks.

Rules for history, shared by `move`, the app, and the plain-Git rules:

1. Move with `git mv`. Never delete and re-create.
2. Do not change the text of the moved document in the move, except relative body links that the move breaks. Full-path wikilinks need no change.
3. Commit the move alone (a pure rename, message `kb: move`). Then, in a second commit, rewrite the links in other documents and in the moved document, and set `updated` and `updated_by` in each document that changed.
4. Never move and edit the content of one document in the same commit.
5. Check afterwards that `git log --follow <new-path>` reaches the old history. The CLI verifies this.

`waymark show <old-id>` later says where the document moved.

At write level `confirm` and confirm in `app`, the dialog shows a summary, for example "Move 42 documents, update links in 17 documents".

## Interfaces & Contracts

```text
waymark move <old> <new> --kb <uuid> [--preview]
```

## Acceptance Criteria

- **Given** a folder with 42 documents, **When** it is moved, **Then** all links to them in other documents work after the move.
- **Given** a moved document, **When** `git log --follow` runs on the new path, **Then** the old history shows.
- **Given** a move, **When** the commit is inspected, **Then** the first commit contains renames only and the second contains link rewrites only.
- **Given** `show <old-id>`, **When** the document moved, **Then** the answer names the new ID.
- **Given** a move into an existing path, **When** it runs, **Then** it fails.
- **Given** `--preview`, **When** `move` runs, **Then** the list of changes is shown and nothing changes.

## Constraints & Edge Cases

- Git does not store moves. It detects a rename only if the old and new file are similar (about 50%), so rule 4 matters.
- Reorganizations need no `owners` field update, because the folder is the owner.

## Open Questions

- How are inbound links from outside the knowledge base (other repositories) reported?
