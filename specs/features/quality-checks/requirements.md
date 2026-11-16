---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-16
requires:
  - search-and-read
---

# Quality checks

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. CI on the Git host and checks on a server are out of scope for now (see `todo.md`). See [project decisions](../../../docs/decisions.md).

## User Story

> As a person, I want every document to be checked for format, size, links, and secrets, so that the memory repository stays clean and safe.

## Personas

- **Every person** — gets a clear message when a document fails.
- **Memory owner** — adds company-specific secret patterns.

## Behaviour

| Tool | Task | Kind |
|---|---|---|
| `lint` | Checks documents and lists findings. Whole memory repository or some folders. | read |

`lint` also runs automatically in every write flow before the commit, on the document that the write changes, and in `sync` on pending manual edits (see `manual-edits`). In `sync`, the rules of the matching write tool come on top, with the finding rules `near-duplicate-title`, `outside-folders`, and `tool-switched-off` (see `document-names`). They are not `lint` rules. A failed check stops a write with `validation_failed`. The message is plain and the `next` sentence says what to fix.

### Rules

Every finding is `{ path, rule, severity, message }`. `path` is the file path below the root. Severity is `error` or `warning`. An error stops a write. A warning does not.

| Rule | Severity | What it checks |
|---|---|---|
| `no-frontmatter`, `unclosed-frontmatter`, `nested-property`, `missing-title`, `missing-description`, `bad-status`, `bad-date`, `bad-author`, `updated-before-created`, `disputed-without-reason`, `title-too-long`, `description-too-long` | error | The frontmatter schema and the title and description limits (see `document-format`). |
| `bad-file-name`, `bad-encoding`, `bad-line-ending`, `document-too-large` | error | File and folder names, UTF-8, LF line ends, and `documents.max_bytes` (see `document-format`). |
| `broken-link` | error in a property (`related`, `supersedes`), warning in the body | A link whose target document does not exist. |
| `duplicate-title` | error | Two documents with the same title (compared without case and extra white space). The older one (by `created`, then by ID) keeps the title. The other one gets the finding. |
| `author-mismatch` | warning | `created_by` differs from the author of the commit that added the file, or `updated_by` differs from the author of the last commit that changed it. Only for files that are in Git history. |
| `author-not-allowed` | error | The email domain of `created_by` or `updated_by` is not in `people.email_domains` (when set), or it holds a part of `people.email_blocklist`. |
| `secret` | error | Built-in secret patterns and `lint.extra_secret_patterns`. |
| `personal-data` | error | A valid IBAN or a payment card number (Luhn check). |

Built-in secret patterns: a private key block, an AWS access key ID, a GitHub token, a Slack token, an `sk-` API key, a JSON Web Token, a connection string with a password, and an assignment like `password = ...` or `api_key: ...` with a value of 8 or more characters. A placeholder such as `<password>` or `***` is not a secret. A finding never repeats the matched text. It names the pattern and the line.

A memory repository adds patterns as regular expressions in `lint.extra_secret_patterns`, for example for customer ID formats. An invalid expression makes `lint` fail with `setup_problem` and names the entry.

`lint` accepts callouts, embeds, highlights, and comments that people wrote. It does not check files that are not documents: files that start with `.`, entries of `.waymark/exclude`, and files that do not end in `.md`.

## Interfaces & Contracts

### `lint`

| Input | Type | Required | Meaning |
|---|---|---|---|
| `folders` | array of string | no | Default: all. The `folders` limit applies. |

```json
{ "tool": "lint", "arguments": { "folders": ["backend"] } }
```

```json
{
  "message": "42 documents checked: 1 error, 1 warning.",
  "settings": {},
  "checked": 42,
  "errors": 1,
  "warnings": 1,
  "findings": [
    { "path": "backend/payments/retries.md", "rule": "broken-link", "severity": "error", "message": "The property related links to `backend/old`. No such document." },
    { "path": "backend/payments/overview.md", "rule": "broken-link", "severity": "warning", "message": "The body links to `limits.md`. No such document." }
  ],
  "truncated": false
}
```

Findings are sorted: errors first, then by `path`. At most 100 findings. `truncated` is true when more exist. A clean memory repository returns `findings: []`.

### Errors

`lint`: `setup_problem` (bad extra pattern, no memory repository).

### API for write tools

Write tools check the text that they will write with the same code (see `plan.md`). A failed check returns `validation_failed` with the first error as `message` and a `next` sentence, for example "Add a description of at most 50 words."

## Acceptance Criteria

- **Given** a broken relative link in the body, **When** `lint` runs, **Then** it reports the file and the target as a warning.
- **Given** a broken link in `related`, **When** `lint` runs, **Then** it reports an error.
- **Given** a custom pattern, **When** a document matches it, **Then** a write fails with `validation_failed` and `lint` reports `secret`.
- **Given** a document edited in Obsidian with a long title, **When** `lint` runs, **Then** it reports `title-too-long`.
- **Given** `created_by` that differs from Git history, **When** `lint` runs, **Then** it warns with `author-mismatch`.
- **Given** a clean repository, **When** `lint` runs, **Then** it returns no findings.
- **Given** a callout or an embed written by a person, **When** `lint` runs, **Then** it does not reject it.
- **Given** a body with a private key block, **When** a write tool runs, **Then** it fails and the message does not repeat the key.
- **Given** two documents with the same title, **When** `lint` runs, **Then** the newer one has `duplicate-title`.
- **Given** an IBAN in a body, **When** `lint` runs, **Then** it reports `personal-data`.
- **Given** the conformance fixtures, **When** `lint` runs on them, **Then** valid ones have no error and invalid ones have the rule that their file name names.
- **Given** `people.email_domains = ["example.org"]` and an author with another domain, **When** `lint` runs, **Then** it reports `author-not-allowed`.
- **Given** `access = "read-only"`, **When** the client lists tools, **Then** `lint` is listed.
- **Given** any setup, **When** the client lists tools, **Then** there is no `index` tool, and `lint` never reports `index-stale` (see `remove-index`).

## Constraints & Edge Cases

- Secrets and customer data never belong in a memory repository. The scan reduces the risk but cannot guarantee it.
- A secret in a code block is still a finding.
- A duplicate title blocks only the newer document. The older document stays valid.
- The author check needs Git history. A new file that is not committed has no history, so the check skips it.
- A CI job on the Git host can use the server through an MCP client. The binary has no command for CI.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). Whether target organizations allow CI on the Git host is in `todo.md`.
