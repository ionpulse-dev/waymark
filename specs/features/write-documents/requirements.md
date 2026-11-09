---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-09
requires:
  - cli-core
  - git-sync
---

# Write documents

## User Story

> As a person, I want my agent to write, correct, dispute, replace, and retire documents safely, so that the knowledge base stays accurate and nothing I own is overwritten by mistake.

## Personas

- **Developer** — the agent records a decision or a pitfall.
- **Every person** — the agent corrects or disputes a document that is wrong.

## Behaviour

| Command | Task |
|---|---|
| `waymark add` | Creates a document. Input: folder, title, description, sources, tags, and the body (from a flag, a file, or stdin). waymark creates the frontmatter, picks a unique file name (a slug of the title, plus a short ID if needed), places the file in `knowledge/`, validates it, and syncs. With verification on, it sets `trust: unverified`. |
| `waymark update-document <id>` | Changes a document. Validates and syncs. Takes `--base <hash>`. |
| `waymark dispute <id> --reason <text>` | Sets `disputed: true` and `dispute_reason`. Used when an agent finds a contradiction and does not fix it now. The document shows in the review list of its author and in `updates`. |
| `waymark dispute <id> --clear` | Ends a dispute: removes `disputed` and `dispute_reason`. Used when the document is fixed or the person confirms that it is right. |
| `waymark deprecate <id> --reason <text>` | Sets `status: deprecated` and keeps the file. Used when a document is no longer true and nothing replaces it. |
| `waymark add ... --supersedes <id>` | Replaces a document: the new document names the old one in `supersedes`, and the old one gets `status: deprecated` in the same commit. The old document stores no link to the new one. |
| `waymark delete <id>` | Deletes a document. Only when the person asks, never on the agent's own decision. It searches for links to the document, in bodies and in properties, fixes them in the same commit, and tells the person that the file stays in the Git history. |

`add` sets all four author fields (`created`, `created_by`, `updated`, `updated_by`). `update-document`, `dispute`, `deprecate`, and `delete` (for the documents whose links it fixes) set `updated` and `updated_by`.

Choose the lightest tool that fits: fix the document if you can, else dispute it, else deprecate it, and replace it when a new document takes over. Delete only on request. When someone else set a dispute flag, the agent asks the person before it clears it.

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
- **Given** `dispute --clear`, **When** it runs, **Then** `disputed` and `dispute_reason` are gone and `updated` is set.
- **Given** `add --supersedes <id>`, **When** it runs, **Then** one commit holds the new document and the old document with `status: deprecated`.
- **Given** `delete` of a document that others link to, **When** it runs, **Then** the file is gone and no link to it is left.
- **Given** write level `auto` and no request from the person, **When** the agent follows the skill, **Then** it does not call `delete`.
- **Given** write level `manual` and no request from the person, **When** the agent follows the skill, **Then** it does not call a write command.

## Constraints & Edge Cases

- When the person asks for a document, it is written. Only when the agent decides alone (write level `confirm` or `auto`), it writes what will be true in three months and is useful beyond the conversation. The skill enforces this by rule, not by the CLI.
- A document over the size cap fails with a message to split it.
- The commit message has a fixed format and no agent tag.

## Open Questions

- Should `update-document` accept a partial frontmatter update without sending the body?
