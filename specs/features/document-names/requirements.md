---
type: feature
status: implemented
created: 2026-11-16
updated: 2026-11-17
requires:
  - write-documents
  - manual-edits
  - move-documents
---

# Document names

## User Story

> As a person with an agent, I want to choose the file name of a new document, such as `readme`, and I want manual edits to follow the same rules as the agent tools, so that no agent has a reason to go around the tools and no link changes its meaning.

## Personas

- **Every person** — asks the agent for a document with a short, known name, such as `people/readme`.
- **Obsidian user** — edits, adds, renames, and deletes documents by hand and publishes them with `sync`.
- **Memory owner** — sets limits (folders, tools that are switched off, approval, verification) and expects them to hold for every change.

## Background

An agent was asked to create `people/readme`. `add_document` makes the file name from the title, so the agent could not do it. The agent wrote the file by hand and called `sync`. `sync` checked the file with `lint` only. `lint` is weaker than the write tools: it does not check near-duplicate titles, the `folders` limit, tools that are switched off, or verification. So a hand edit was a way around the rules.

This spec closes the gap in three ways: an agent can choose the name, `sync` uses the rules of the write tools, and the agent gets a clear rule for `sync`.

## Behaviour

### 1. An optional name in `add_document`

- `add_document` takes an optional `name`: the file name without `.md`.
- A name uses lowercase letters `a` to `z`, digits, and `-`. It does not start with `-`. It has at most 60 characters. It has no `/`.
- Without `name`, waymark makes the name from the title, as before (see `write-documents`).
- With `name`, the ID is `<folder>/<name>`. If a document with this ID exists (compared without case), the call fails with `validation_failed`. There is no suffix, because the caller chose the name.
- The title stays a free sentence. It can change later, and the name stays.

### 2. `sync` uses the rules of the write tools

`sync` finds the pending manual edits as before. Then it sorts each edit into the write tool that would make the same change, and it applies the rules of that tool:

| Manual edit | Rules of | What the rules add to `lint` |
|---|---|---|
| A new file | `add_document` | Exact and near-duplicate titles, the `folders` limit, the tool is not switched off, approval of `add_document`, `trust: unverified` with verification on. |
| A changed file | `update_document` | Exact and near-duplicate titles when the title changed (the document itself does not count), the `folders` limit, the tool is not switched off, approval, and a change of `title`, `description`, or `body` sets a verified document to `trust: unverified`. |
| A deleted file | `delete_document` | The `folders` limit, the tool is not switched off, approval, and no other document links to it (as before). A document whose own edit is blocked counts with its committed text. |
| A renamed or moved file, with the same text | `move` | The `folders` limit for both places, the tool is not switched off, approval, and every link to the old ID is rewritten. |

- **`lint` still runs first.** Every rule of `quality-checks` still applies. The rules of the table come on top.
- **A renamed file.** A deleted file and a new file with exactly the same text are one rename. A rename that also changes the text is a delete and a new file. The delete waits while other documents link to the old ID.
- **Pending edits among themselves.** The edits are taken in ID order, with deletes last. A title clash with another pending edit counts only when that edit passed earlier in the same run. Like `add_document`, an exact duplicate of an active title (same words) blocks the edit. The `lint` rule `duplicate-title` applies too.
- **Hashes after a pull.** A pull with rebase gives the new commits new hashes. A rename and its link fixes stay one `activity` entry: the link-fix commit belongs to the commit just before it.
- **One commit for each edit.** The subject stays `waymark: sync "<title>"`. The next lines say which tool's rules applied: `Action: add`, `update`, `delete`, or `move`. A move has two commits, as in `move`: the rename alone, then the link fixes. So `activity` and `undo` work for each edit.
- **Blocked edits** stay in the folder, unchanged, as before. Each one is in `blocked` with its findings. The new rules have these names:

| Rule | When |
|---|---|
| `near-duplicate-title` | The title shares at least 80 percent of the words of another active title (see `write-documents`). |
| `outside-folders` | The document is outside the `folders` limit. |
| `tool-switched-off` | The matching write tool is in `tools_denied`. |

- **Approval.** When the limits require approval for `sync` or for one of the matching tools, `sync` asks once and lists all edits, with the old path of a rename.
- **Verification by hand.** With verification on, only `verify_document` sets a document to `verified`. A hand edit that sets `trust: verified`, or changes `verified_by` or `verified`, is published with `trust: unverified`. A document that is not verified keeps no `verified_by` and no `verified`.
- **Conflicts.** One commit for each edit means more local commits. A pull with rebase stops at the first commit that conflicts, so `sync` finds the conflicts of all local commits with a trial merge, and keeps both versions of each one (see `git-sync`).
- **Before every other write.** Every write tool still publishes pending edits first, with the same rules. An edit whose matching tool needs approval is not published there. It waits for `sync`, and the result says so in a notice. Every edit after it waits too, so that no published edit builds on a held one.

### 3. Short wikilinks keep their target

A short wikilink, such as `[[readme]]`, names only a file name. It finds a document only when exactly one document has that file name (see `search-and-read`). When a second document gets the same file name, the link finds no document any more.

- Every write that makes a new ID applies this rule: `add_document`, `move`, and `sync` (new and renamed files).
- Before the second document gets the file name, waymark rewrites every short wikilink that found the first document to its full path: `[[readme]]` becomes `[[people/readme]]`. The alias and the heading stay.
- For `add_document`, the rewrites are in the same commit as the new document. For `move` and a renamed file in `sync`, they are in the link commit.
- A document with a pending manual edit is not rewritten by `add_document` and `sync`. Its edit is checked when it is published. `move` fails with `stale_base` when such a document needs a fix, as it does for any link fix (see `move-documents`).
- A linking document outside the `folders` limit stops the write with `access_denied`, as in `move`.
- The rule is part of the Waymark Memory Standard, section 8.
- Known limits: a document with a pending edit is not rewritten (see above), and `undo` of a delete or a move restores the earlier state exactly, also when that brings back a taken file name (see `activity-and-undo`). `lint` reports a short link that finds no document.

### 4. A rule for agents

waymark gives no behavior advice in tool descriptions or in the server instructions (see `mcp-server`). The server enforces the rules, and the team writes its advice in `AGENTS.md`.

- The description of `sync` says what it is for: "Publishes manual edits: documents that a person changed by hand, outside waymark." It also says that each edit gets the checks of the matching write tool.
- Both `AGENTS.md` suggestions in `docs/local-install.md` get the rule: "Do not call `sync` unless the person asks for it after a manual edit."
- An agent that calls `sync` anyway gets the same checks as the write tools, so it gains nothing.

## Interfaces & Contracts

### `add_document`

New optional input field:

| Field | Type | Required | Meaning |
|---|---|---|---|
| `name` | string | no | File name without `.md`: lowercase letters, digits, and `-`, at most 60 characters. Default: made from the title. |

The result has the new field `links_updated` only when short wikilinks were rewritten: `[{ "id": "ops/overview", "count": 1 }]`.

### `sync`

The input does not change. In the result, each entry of `committed` has its own `commit`. `kind` is `new`, `changed`, `deleted`, or `moved`. A `moved` entry has `from`.

```json
{
  "message": "2 manual edits published.",
  "committed": [
    { "id": "people/readme", "kind": "new", "commit": "a1b2c3d" },
    { "id": "ops/deploy-guide", "kind": "moved", "from": "ops/deploy", "commit": "e4f5a6b" }
  ],
  "blocked": [],
  "skipped": [],
  "pulled": 0,
  "pushed": true,
  "conflicts": []
}
```

For a move, `commit` is the commit of the link fixes, or of the rename when no link changed.

## Acceptance Criteria

- **Given** `add_document` with `name: "readme"` and `folder: "people"`, **When** it runs, **Then** the ID is `people/readme` and the title is unchanged.
- **Given** a document `people/readme`, **When** `add_document` runs with the same folder and name, **Then** it fails with `validation_failed` and names `people/readme`.
- **Given** a name with capitals, a `/`, `.md`, a leading `-`, or more than 60 characters, **When** `add_document` runs, **Then** it fails with `invalid_arguments`.
- **Given** no `name`, **When** `add_document` runs, **Then** the name comes from the title as before.
- **Given** `people/readme` and a document whose body links with `[[readme]]`, **When** `add_document` creates `projects/readme`, **Then** the link becomes `[[people/readme]]` in the same commit and `people/readme` still has the backlink.
- **Given** the same link, **When** `move` gives another document the file name `readme`, **Then** the link becomes `[[people/readme]]`.
- **Given** a manual new file whose title is a near-duplicate of an active title, **When** `sync` runs, **Then** the file is blocked with `near-duplicate-title`.
- **Given** the limit `folders = ["ops"]` and a manual new file in `people`, **When** `sync` runs, **Then** the file is blocked with `outside-folders`.
- **Given** `tools_denied = ["add_document"]`, a manual new file, and a manual change, **When** `sync` runs, **Then** the new file is blocked with `tool-switched-off` and the change is published.
- **Given** verification on, a verified document, and a manual change of its body, **When** `sync` runs, **Then** the published document has `trust: unverified` and no `verified_by`.
- **Given** verification on and a manual new file with `trust: verified`, **When** `sync` runs, **Then** the published document has `trust: unverified`.
- **Given** verification on and a manual change of only `trust` from `unverified` to `verified`, **When** `sync` runs, **Then** the published document has `trust: unverified`.
- **Given** two manual edits to two documents that another person also changed, **When** `sync` runs, **Then** both conflicts keep both versions and `sync` succeeds.
- **Given** a document renamed by hand and published by `sync`, **When** `activity` runs, **Then** the rename and its link fixes are one entry, and `undo` takes back both.
- **Given** two manual edits, **When** `sync` runs, **Then** there are two commits, each with its `Action:` line, and `activity` lists two entries.
- **Given** a document renamed by hand with the same text, and another document that links to it, **When** `sync` runs, **Then** it is published as `moved`, the link points to the new ID, and `show` with the old ID follows the move.
- **Given** approval required for `add_document`, **When** `sync` publishes a manual new file, **Then** it asks first.
- **Given** approval required for `add_document` and a manual new file, **When** the agent calls `update_document` on another document, **Then** the new file is not published and the result has a notice.
- **Given** the tool list, **When** an agent reads the description of `sync`, **Then** it says that `sync` publishes manual edits with the checks of the matching write tool, and it holds no behavior advice.
- **Given** the `AGENTS.md` suggestions in `docs/local-install.md`, **When** a person reads them, **Then** both hold the rule for `sync`.

## Constraints & Edge Cases

- A rename that Git sees as a staged rename counts like a deleted file and a new file with the same text.
- A rename and a text change at once is a delete and a new file. The deleted file is not in the folder any more, so its title does not clash with the new file.
- The short-link rule does not change Markdown links and full-path wikilinks. They name the full path.
- A short wikilink that already found two or more documents before the write stays as it is.

## Open Questions

None.
