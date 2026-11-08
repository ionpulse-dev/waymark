---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
  - git-sync
---

# Write documents

## User Story

> As a person, I want my agent to write, correct, dispute, and retire documents safely, so that the knowledge base stays accurate and nothing I own is overwritten by mistake.

## Personas

- **Developer** — the agent records a decision or a pitfall.
- **Every person** — the agent corrects or disputes a document that is wrong.

## Behaviour

| Command | Task |
|---|---|
| `waymark add` | Creates a document. Input: folder, title, description, sources, tags, and the body (from a flag, a file, or stdin). waymark creates the frontmatter, picks a unique file name (a slug of the title, plus a short ID if needed), places the file in `knowledge/`, validates it, and syncs. With verification on, it sets `trust: unverified`. |
| `waymark update-document <id>` | Changes a document. Validates and syncs. Takes `--base <hash>`. |
| `waymark dispute <id> --reason <text>` | Sets `disputed: true` and `dispute_reason`. Used when an agent finds a contradiction. The document shows in the review list of its author and in `updates`. |
| `waymark deprecate <id> --reason <text>` | Sets `status: deprecated` and keeps the file. |

`add` sets all four author fields (`created`, `created_by`, `updated`, `updated_by`). `update-document`, `dispute`, and `deprecate` set `updated` and `updated_by`.

All write commands follow the write flow of `git-sync`: lock, fetch, base check, checks, commit under the person's account, push. They accept `--preview`, which shows what would be written and writes nothing.

Every write is checked before the commit: frontmatter schema, Obsidian compatibility, location, size limits, duplicate titles, broken links, and a secret scan. A failed check stops the write with exit code 2.

Write level applies (see `trust-model`):

- `manual`: the person asks, the agent writes.
- `confirm`: the agent decides and asks first, in the chat or in the app dialog (see `approval-dialog`).
- `auto`: the agent decides and writes.

With the CLI present, agents never write files in the content repository directly. They call these commands.

## Interfaces & Contracts

```text
waymark add --kb <uuid> --folder backend/payments --title "..." --description "..." [--sources ...] [--tags ...] [--preview] < body.md
waymark update-document <id> --kb <uuid> --base <hash> [--title ...] [--description ...] < body.md
```

Exit codes: 2 validation, 3 sync, 5 stale base, 6 canceled, 7 app not running, 8 paused.

## Acceptance Criteria

- **Given** a valid input, **When** `add` runs, **Then** one commit under the person's account reaches the remote and the document has all required fields.
- **Given** a document with a secret pattern, **When** `add` runs, **Then** it exits with code 2 and writes nothing.
- **Given** a title that already exists, **When** `add` runs, **Then** it fails as a duplicate.
- **Given** a changed remote document, **When** `update-document` runs with the old base, **Then** it exits with code 5.
- **Given** `--preview`, **When** `add` runs, **Then** nothing is written or committed.
- **Given** `dispute`, **When** it runs, **Then** `disputed` is true and the reason is saved.
- **Given** `deprecate`, **When** it runs, **Then** the file still exists with `status: deprecated`.
- **Given** write level `manual` and no request from the person, **When** the agent follows the skill, **Then** it does not call a write command.

## Constraints & Edge Cases

- Write only what is true in three months and useful outside the team. The skill enforces this by rule, not by the CLI.
- A document over the size cap fails with a message to split it.
- The commit message has a fixed format and no agent tag.

## Open Questions

- Should `update-document` accept a partial frontmatter update without sending the body?
