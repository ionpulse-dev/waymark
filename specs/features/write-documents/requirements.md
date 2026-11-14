---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - mcp-server
  - git-sync
  - document-format
---

# Write documents

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Parts that need server mode stay out of this text and are listed in `todo.md` (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a person, I want my agent to write, correct, dispute, replace, and retire documents safely, so that the knowledge base stays accurate and nothing I own is overwritten by mistake.

## Personas

- **Developer** — the agent records a decision or a pitfall.
- **Every person** — the agent corrects or disputes a document that is wrong.

## Behaviour

| Tool | Kind | Task |
|---|---|---|
| `add` | write | Creates a document. waymark builds the frontmatter, picks a unique file name, validates, commits, and pushes. |
| `update_document` | write | Changes fields or the body of a document. Fields that are not sent stay as they are. |
| `dispute` | write | Sets `disputed: true` and `dispute_reason`. With `clear: true` it ends the dispute. |
| `deprecate` | write | Sets `status: deprecated` and keeps the file. |
| `delete` | destructive | Deletes a document. Refuses while another document links to it. |

All five follow the write flow of `git-sync` (limits, lock, fetch, base check, checks, approval, commit under the author, push). All accept `preview: true`. A document in a result has the ID (path below the root without `.md`) and `version`, the short hash of the last commit of its file. A write that changes an existing document sends that value as `base`. A different value is `stale_base`.

### Rules for every tool

- **Dates and authors.** `add` sets `created`, `created_by`, `updated`, `updated_by`. Every other tool sets `updated` and `updated_by` on each document it changes. The date is today's date on the machine. The author is `Name <email>` from the settings of waymark (see `git-sync`).
- **Commit.** One commit for one call. The message is `waymark: <verb> "<title>"`, with `add`, `update`, `dispute`, `deprecate`, or `delete`. `deprecate` adds the line `Reason: <reason>` after an empty line. No agent tag.
- **File name.** `add` makes the name from the title: lowercase ASCII letters and digits, other characters become `-`, repeated `-` are merged, and the result is cut at 60 characters at a `-`. An empty result becomes `document`. If the file exists, `-<4 hex>` is added (the start of the SHA-256 of folder and title). If that exists too, 6, then 8 hex. The name never changes when the title changes.
- **Folder.** `folder` is a path below the root, with no `..`, no leading `/`, and no hidden segment. Each segment follows the file name rules. A new folder is created. `""` is the root. It must match the `folders` limit, or the call fails with `access_denied` and `next` says "This folder is outside the allowed folders." A write never goes into `.waymark/` or a path of `.waymark/exclude`.
- **Verification on** (see `verification`). `add` sets `trust: unverified`. A change of `title`, `description`, or `body` of a verified document sets `trust: unverified` and removes `verified_by` and `verified`. With verification off, trust fields are kept as they are.
- **Unknown properties** of a document are kept and written after the known ones.
- **Body.** The body is never changed unless the caller sends `body`, or `delete` removes a link from it. Line ends in `body` are changed to LF.

### Checks

Every write is checked before the commit, on the result of the change. A failed check ends the call with `validation_failed`. The `message` lists every finding, one per line, and `next` says what to do. Nothing is written.

| Check | Fails when | Example of `next` |
|---|---|---|
| Format | A rule of `format::check` has an `error` finding: required fields, types, status, dates, file name | "Add a description of at most 50 words." |
| Size | Title or description is over its word limit, or the file is over `documents.max_bytes` | "The document is over the cap. Split it, or move details into a source." |
| Duplicate | Another `active` document has the same title (lowercase words, same order) | "A document with this title exists: `ops/deploy`. Update it instead." |
| Near-duplicate | The word overlap with another `active` title is at least 80 percent of the shorter title (see below) | "A document with a similar title exists: `ops/deploy`. Update it instead." |
| Property links | A link in `related` or `supersedes` has no target | "The link to `ops/old` has no target." |
| Body links | The write adds a link in the body to a document that does not exist | "The body links to `ops/old`. No such document. Write that document first, or remove the link." |
| Secrets | A built-in or custom pattern matches the title, description, body, sources, tags, or a reason | "The text looks like it contains a secret (line 12). Remove it." |

The server never prints the matched secret.

**Near-duplicate rule.** A title is split into lowercase words: runs of letters and digits. The overlap is the number of distinct words that both titles share, divided by the number of distinct words of the shorter title. At 0.8 or more the title is a near-duplicate. If the shorter title has fewer than 3 words, only an exact match counts. The check ignores deprecated documents and the document named in `supersedes`, so a replacement can reuse a title. On `update_document`, the document itself is ignored.

**Body links.** A write never creates a broken link. For `add`, every link in the body counts. For `update_document`, only the links that are new compared with the old body count, so an old broken link does not block an unrelated change. The check covers relative Markdown links that end in `.md` and wikilinks. It ignores URLs, links to other file types, `#heading` parts, and links in code fences and inline code. A target counts as existing if the document exists after the write. `dispute`, `deprecate`, and the other writes that leave the body as it is check nothing. An old broken link stays and `lint` reports it as a warning. `sync` applies the same rule to documents that people edit by hand (see `manual-edits`).

Secret patterns (built in): `-----BEGIN ... PRIVATE KEY-----`, `AKIA` plus 16 letters or digits, `ghp_` or `github_pat_` plus a token, `sk-` plus 20 or more letters or digits, `password`, `passwd`, `secret`, `token`, or `api_key` followed by `=` or `:` and a value, and `://user:password@`. A knowledge base adds regular expressions in `lint.extra_secret_patterns`.

### `add`

Input:

| Field | Type | Required | Meaning |
|---|---|---|---|
| `folder` | string | yes | Folder of the new document. `""` is the root. |
| `title` | string | yes | At most 15 words (the setting). |
| `description` | string | yes | At most 50 words (the setting). |
| `body` | string | yes | Markdown text. May be empty. |
| `sources` | string[] | no | Ticket IDs or URLs. |
| `tags` | string[] | no | No `#`, no spaces. |
| `related` | string[] | no | Document IDs. waymark writes them as full-path wikilinks. |
| `anchors` | string[] | no | `git:<repo>/<path>@<commit>` or `url:<URL>`. |
| `supersedes` | string | no | ID of the document that this one replaces. |
| `preview` | boolean | no | Default `false`. |

`supersedes` names an existing document. The new document stores it as a wikilink list of one entry. The old document gets `status: deprecated`, `updated`, and `updated_by`. It stores no link to the new one. Both changes are in one commit with the message of `add`. An old document that is already deprecated stays as it is.

Result `data`:

```json
{
  "id": "backend/payments/idempotency-keys",
  "title": "Payment API v3 uses idempotency keys",
  "version": "a1b2c3d",
  "commit": "a1b2c3d",
  "pushed": true,
  "deprecated": ["backend/payments/idempotency-keys-v2"],
  "preview": false
}
```

### `update_document`

Input:

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | Document ID. |
| `base` | string | yes | `version` from `show`. |
| `title`, `description`, `body` | string | no | New value. |
| `sources`, `tags`, `related`, `anchors` | string[] | no | New list. It replaces the old list. An empty list removes the property. |
| `preview` | boolean | no | Default `false`. |

At least one of the optional fields must be present, or the call fails with `invalid_arguments`. `status`, `disputed`, `dispute_reason`, `supersedes`, the author and date fields, and trust fields cannot be sent. A change of `title` keeps the file name and the ID. If the new content equals the old content, nothing is committed, and the result has `"commit": null` and the message "No change."

Result `data`: as `add`, without `deprecated`, plus `"changed": ["title", "body"]`.

### `dispute`

Input: `id` (string, required), `base` (string, required), `reason` (string, required unless `clear` is `true`), `clear` (boolean, default `false`), `preview` (boolean).

The reason is one line of at most 50 words. A call with `reason` on an already disputed document fails with `validation_failed`: "This document is already disputed: <reason>. Fix it with `update_document`, or end the dispute with `clear: true`." A call with `clear: true` on a document that is not disputed fails with `validation_failed`. `clear: true` removes `disputed` and `dispute_reason`. Someone else's dispute may be cleared too, because the server does not judge. A call with both `reason` and `clear: true` is `invalid_arguments`.

Result `data`: `{ "id", "title", "version", "commit", "pushed", "disputed": true, "preview" }`. With `clear` the value of `disputed` is `false`.

### `deprecate`

Input: `id` (string, required), `base` (string, required), `reason` (string, required, one line, at most 50 words), `preview` (boolean).

The reason is saved in the body of the commit message, not in the file. A document that is already deprecated fails with `validation_failed`. There is no tool to make a deprecated document active again (see `todo.md`).

Result `data`: `{ "id", "title", "version", "commit", "pushed", "status": "deprecated", "preview" }`.

### `delete`

Input: `id` (string, required), `base` (string, required), `preview` (boolean).

Before it deletes, the tool finds every link to the document in all other documents: relative Markdown links, wikilinks, and embeds in bodies, and wikilinks in properties. If any exists, the call fails with `validation_failed` and changes nothing. The message says how many documents link to it, and `data.linked_from` lists each one with `id`, `where` (`body` or `property`), and `count`. The `next` sentence says: "Remove or replace the links with update_document, then call delete again. Or use deprecate to keep the document." The tool never edits another document. Generated index files (see `quality-checks`) are ignored, because they list every document and `lint` reports a stale one. `preview: true` runs the same check and fails the same way, so an agent can ask what would break without a risk.

The `folders` limit applies to the document that is deleted. If the deleted document has `supersedes`, a notice says that the older documents stay deprecated. The message says that the file stays in the Git history.

Result `data`:

```json
{
  "id": "ops/old-deploy",
  "title": "Old deploy process",
  "commit": "a1b2c3d",
  "pushed": true,
  "preview": false
}
```

### Preview

With `preview: true` the tool runs all checks and returns the same `data` without `commit` and `pushed` (both `null`), plus `"changes": [ { "id": "...", "action": "create" } ]` with the actions `create`, `update`, `deprecate`, and `delete`. `add` and `update_document` also return `"text"`: the full file that would be written. Nothing is written, committed, or pushed, and no approval is asked.

### Errors

| Code | When |
|---|---|
| `validation_failed` | A check failed. |
| `stale_base` | `base` is not the current `version`. `next`: "Read the document again with `show`." |
| `not_found` | `id` or the `supersedes` target does not exist. |
| `invalid_arguments` | A required field is missing, a field is of the wrong type, or the combination is not allowed. |
| `access_denied`, `paused`, `approval_denied`, `approval_unavailable`, `sync_failed` | As in `mcp-server` and `git-sync`. |

## Acceptance Criteria

- **Given** a valid input, **When** `add` runs, **Then** one commit under the author reaches the remote and the document has all required fields.
- **Given** a body with a secret pattern, **When** `add` runs, **Then** it fails with `validation_failed` and writes nothing.
- **Given** a title that already exists, **When** `add` runs, **Then** it fails as a duplicate and names the existing document.
- **Given** a title that shares at least 80 percent of the words of the shorter title with an active document, **When** `add` runs, **Then** it fails as a near-duplicate and names that document.
- **Given** a changed remote document, **When** `update_document` runs with the old `base`, **Then** it fails with `stale_base`.
- **Given** only `description` is sent, **When** `update_document` runs, **Then** the body is byte-for-byte the same and `updated` is set.
- **Given** a changed title, **When** `update_document` runs, **Then** the file name and the ID stay the same.
- **Given** `preview: true`, **When** any write tool runs, **Then** nothing is written or committed.
- **Given** `dispute`, **When** it runs, **Then** `disputed` is true and the reason is saved.
- **Given** `deprecate`, **When** it runs, **Then** the file still exists with `status: deprecated`.
- **Given** `dispute` with `clear: true`, **When** it runs, **Then** `disputed` and `dispute_reason` are gone and `updated` is set.
- **Given** `add` with `supersedes`, **When** it runs, **Then** one commit holds the new document and the old document with `status: deprecated`.
- **Given** `delete` of a document that others link to, **When** it runs, **Then** it fails with `validation_failed`, nothing changes, and `linked_from` lists the linking documents.
- **Given** the links are removed with `update_document`, **When** `delete` runs again, **Then** the file is gone.
- **Given** `delete` with `preview: true` of a linked document, **When** it runs, **Then** it fails the same way and lists the links.
- **Given** `tools_denied = ["delete"]`, **When** the client lists tools, **Then** `delete` is not listed.
- **Given** `access = "read-only"`, **When** any write tool runs, **Then** it fails with `access_denied` and no commit is made.
- **Given** `approval.required_for = ["add"]`, **When** `add` runs, **Then** the approval is asked before the commit (see `write-approval`).
- **Given** a body link to a missing document, **When** `add` runs, **Then** it fails with `validation_failed`, names the link, and changes nothing.
- **Given** a document with an old broken body link, **When** `update_document` changes only the title, **Then** the write succeeds.
- **Given** `update_document` adds a body link to a missing document, **When** it runs, **Then** it fails with `validation_failed`.
- **Given** `folders = ["engineering/**"]`, **When** `add` runs with another folder, **Then** it fails with `access_denied`.
- **Given** a verified document and verification on, **When** `update_document` changes the body, **Then** `trust` is `unverified`.

## Constraints & Edge Cases

- A document over the size cap fails with a message to split it.
- The commit message has a fixed format and no agent tag.
- The same `add` twice: the second call fails as a duplicate. The state is the same as after the first.
- A tool that takes an `id` treats a document outside the `folders` limit as not found (`not_found`), like the read tools.
- Pending manual edits are committed first, in their own commit (see `manual-edits`). They never go into a commit of these tools.
- Choosing the lightest tool (fix, dispute, deprecate, replace, delete) is advice for a team's `AGENTS.md`, not a rule of the server. The server only makes `delete` easy to switch off.
- The server does not judge what deserves a document, and it does not decide when to write.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`): `update_document` accepts a partial update, and the near-duplicate rule is a word overlap.
