---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-16
requires:
  - quality-checks
---

# Manual edits and sync

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The background pull and the Obsidian plugin on a server are out of scope for now (see `todo.md`). See [project decisions](../../../docs/decisions.md).

## User Story

> As a person, I want to edit or move documents in Obsidian or any editor and have them published for me, so that I never need Git.

## Personas

- **Every person** — edits documents by hand.
- **Obsidian user** — browses and edits the memory repository as a vault.

## Behaviour

In local mode, the clone of the memory repository is a normal folder. A person can open it as an Obsidian vault and edit it. waymark has no background job, so manual edits are published when the server runs. A manual edit is a change to a document that was made outside waymark: a changed, new, deleted, or moved `.md` file in the working tree.

| Tool | Task | Kind |
|---|---|---|
| `sync` | Commits pending manual edits, pulls, and pushes. | write |

### What `sync` does

> **Changed on 2026-11-16 by [document-names](../document-names/requirements.md):** each edit gets the rules of the matching write tool on top of `lint` (`add_document`, `update_document`, `delete_document`, or `move`), a renamed file with the same text is a move, and each edit gets its own commit with an `Action:` line. Steps 2, 4, and 5 below are read with this change.

1. **Find pending edits.** Changed, new, and deleted `.md` documents below the root. Files that are not documents (images, `.waymark/` files, files that start with `.`, entries of `.waymark/exclude`) are never committed. They are listed under `skipped`.
2. **Run `lint` first.** Every pending document goes through `lint` (see `quality-checks`). A document with an error is not committed. It is reported under `blocked` with its findings. It stays in the folder, unchanged. A document that passes later is committed by the next `sync`. Warnings do not block, with one exception: a broken body link that the edit adds counts as an error, like in a write tool (see `write-documents`). A link that was already broken in the last commit stays a warning. A document that is deleted by hand while another document still links to it is blocked too, and the finding names the linking documents. If the links go in the same `sync`, both go together.
3. **Set the author fields.** For a changed document that passed, waymark sets `updated` to today and `updated_by` to the author, if they differ. It touches nothing else.
4. **Ask for approval** if the limits require it for `sync` or for writes (see `write-approval`). The request lists the documents. A refusal ends the call, and nothing is committed.
5. **Commit** each passing edit in its own commit under the author: `waymark: sync "<title>"`, then an empty line and `Action: add`, `update`, `delete`, or `move`. A move has two commits: the rename alone, then the link fixes.
6. **Pull and push** as `git-sync` says: `git pull --rebase`, then `git push`. A real conflict keeps both versions (see `git-sync`).

If nothing passes lint and at least one document is blocked, `sync` fails with `validation_failed` and does not pull or push. In all other cases it returns a result.

- **Read-only and paused.** With `access = "read-only"` or paused, `sync` is still listed. It only pulls. It commits nothing and pushes nothing, and it says so in a notice.
- **Preview.** With `preview: true`, `sync` runs `lint` and lists what it would commit. It changes nothing.

### Before every other write

Every write tool commits pending manual edits first, each in its own commit, so that an agent write never carries foreign changes. The same checks apply. An edit whose matching write tool needs approval is not published here: it waits for `sync`, which asks. A document that fails lint is not committed, and the write goes on. The result then has a notice: "2 manual edits are not published. Run `sync`." If the write changes a file that has such an edit, it fails with `stale_base` and the `next` sentence "This document has manual edits that are not published. Run `sync`."

### Moves and deletes by hand

A document moved or renamed by hand with the same text is published with the rules of `move`: the links to it are rewritten. A move with a changed text is a delete and a new file. A document deleted by hand is committed like any edit. Git detects the move. `show` with the old ID follows it (see `search-and-read`). A deleted document that another document still links to is blocked (see step 2). So is the delete half of a move with a changed text, until the links are fixed.

## Interfaces & Contracts

| Input | Type | Required | Meaning |
|---|---|---|---|
| `preview` | boolean | no | `true`: change nothing. |

```json
{ "tool": "sync", "arguments": {} }
```

```json
{
  "message": "1 manual edit published. 1 document failed the checks and stays unpublished.",
  "settings": {},
  "notices": [ "ops/new-doc is not published: the title is missing." ],
  "committed": [ { "id": "ops/deploy", "kind": "changed", "commit": "a1b2c3d" } ],
  "blocked": [
    {
      "id": "ops/new-doc",
      "path": "ops/new-doc.md",
      "findings": [ { "path": "ops/new-doc.md", "rule": "missing-title", "severity": "error", "message": "The title is missing." } ]
    }
  ],
  "skipped": [ { "path": "images/diagram.png", "reason": "Not a document." } ],
  "pulled": 2,
  "pushed": true,
  "conflicts": []
}
```

- `kind` is `new`, `changed`, `deleted`, or `moved`. A hand move with the same text shows as one `moved` entry with `from`. A move with a changed text shows as one `deleted` and one `new`.
- `commit` is the commit of the entry. A `moved` entry names the commit of the link fixes, or of the rename when no link changed. After a pull with rebase, the hashes can change. `activity` shows the current ones.
- `pulled` is the number of commits that the pull brought in. `pushed` is true if a push ran and succeeded.
- `conflicts` holds `{ "id", "copy" }` for each conflict that `git-sync` resolved (see `git-sync`).
- With `preview: true`, `committed` lists what would be committed, without `commit`, and `pulled` and `pushed` are `0` and `false`.
- In read-only mode, `committed` is empty and `pushed` is false.

### Errors

`validation_failed` (all pending documents blocked), `sync_failed`, `setup_problem`, `approval_denied`, `approval_unavailable`, `stale_base` (in other write tools, see above).

## Acceptance Criteria

- **Given** a manual edit that breaks a rule of its matching write tool (near-duplicate title, `folders` limit, switched-off tool), **When** `sync` runs, **Then** the edit is blocked with `near-duplicate-title`, `outside-folders`, or `tool-switched-off` (see `document-names`).

- **Given** an edit in Obsidian, **When** `sync` runs, **Then** a commit `waymark: sync "<title>"` under my account reaches the remote.
- **Given** an edit that fails `lint` and no other edit, **When** `sync` runs, **Then** it fails with `validation_failed`, names the problem, and pushes nothing. The file stays as it was.
- **Given** one passing and one failing edit, **When** `sync` runs, **Then** the passing one is committed and pushed, and the failing one is listed under `blocked` and not committed.
- **Given** pending manual edits, **When** an agent writes a document, **Then** the manual edits are committed first in their own commit.
- **Given** a hand edit that adds a link to a document that does not exist, **When** `sync` runs, **Then** it is blocked with the rule `broken-link` and pushes nothing. A link that was broken in the last commit does not block.
- **Given** a document deleted by hand that another document links to, **When** `sync` runs, **Then** the deletion is blocked and the finding names the linking document.
- **Given** a pending edit that fails `lint`, **When** an agent writes another document, **Then** the write works and the result has a notice.
- **Given** a pending edit that fails `lint`, **When** an agent writes the same document, **Then** the write fails with `stale_base`.
- **Given** approval is required for writes, **When** manual edits are ready, **Then** the request opens before the commit.
- **Given** `access = "read-only"`, **When** `sync` runs, **Then** it pulls and does not commit or push.
- **Given** a changed document, **When** `sync` commits it, **Then** `updated` is today and `updated_by` is the author.
- **Given** a document moved by hand, **When** `sync` runs and then `show` runs with the old ID, **Then** `show` returns the document at the new ID.
- **Given** a file that is not a document, **When** `sync` runs, **Then** it is not committed and is listed under `skipped`.
- **Given** `preview: true`, **When** `sync` runs, **Then** nothing is committed, pulled, or pushed.
- **Given** an edit to `.waymark/settings.toml`, **When** `sync` runs, **Then** the file is not committed.
- **Given** a remote with a change to the same document as a pending edit, **When** `sync` runs, **Then** both versions are kept and a notice says so.

## Constraints & Edge Cases

- Edits made in Obsidian bypass the write checks until they are committed. `lint` is the safety net.
- Files that Obsidian creates for personal use are ignored by Git. Only `.obsidian/app.json`, with the link options of [Waymark Memory Standard 1.0, section 11](../../../standard/waymark-1.0.md), is kept.
- A person who edits and never runs a session leaves edits unpublished. This is a known limit of having no background job. waymark does not commit on a timer.
- A new file from Obsidian has no frontmatter. It is blocked with `no-frontmatter` until the person adds the required fields.
- A repository without a remote commits locally and skips pull and push, with a notice (see `git-sync`).

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). A helper for people without agents is in `todo.md`.
