# Plan: Write documents

## Overview

Five tools, one file each in `src/tools/`. Each tool builds a plan of file changes and hands it to `gitsync::Repo::write`, which holds the lock, updates the clone, checks `base`, runs the approval, commits, and pushes. Shared helpers (file names, title checks, secret scan, link fixing, result building) live in a small `writes` module so that `move`, `undo`, and `verify` reuse them.

## Components Affected

- `format` (`document-format`) — `parse`, `render`, `check`, `Limits`, `Finding`. This feature never writes frontmatter by hand.
- `kb` (`knowledge-base-repo`) — the root, `documents.*` settings, `lint.extra_secret_patterns`, `verification`.
- `config` (`trust-model`) — `EffectiveSettings`: `folders`, `approval`. Folder globs use `config::glob`.
- `gitsync` (`git-sync`) — `Repo::write`. See "What this feature needs from `git-sync`".
- `mcp` (`mcp-server`) — the `Tool` trait, `ToolOutput`, `ToolError`, and the registry.
- `src/tools/mod.rs` — one line per tool in the registry.

## New Files

- `src/tools/add.rs` — the `add` tool.
- `src/tools/update_document.rs` — the `update_document` tool.
- `src/tools/dispute.rs` — the `dispute` tool.
- `src/tools/deprecate.rs` — the `deprecate` tool.
- `src/tools/delete.rs` — the `delete` tool.
- `src/writes/mod.rs` — `DocPlan` (the changes of one call), the glue to `Repo::write`, the common input checks, and the preview and result builders.
- `src/writes/slug.rs` — file name from a title, and the hex suffix rule.
- `src/writes/titles.rs` — title words, duplicate check, near-duplicate check.
- `src/lint/secrets.rs` — built-in secret patterns, custom patterns, and the scan that reports rule and line, never the match. It was `src/writes/secrets.rs` and moved here with `quality-checks`.
- `src/writes/linkfix.rs` — find links to a document in all documents and rewrite them: remove (used by `delete`) or retarget (used by `move`). Uses `format::links` for the link spans.
- `tests/it_write_documents.rs` — end-to-end tests with the `World` harness.

## Technical Approach

1. **Common path.** A tool parses its arguments (`invalid_arguments` on a missing or wrong field). It resolves the knowledge base and the `folders` limit. Then it calls `Repo::write` with a closure.
2. **The closure** runs inside the lock, after fetch and fast-forward. It reads the current file, compares `version` with `base` (`gitsync` does the hash, see below), builds the new document with `format::parse` and `format::render`, runs the checks, and returns a `DocPlan`: the files to write, delete, or leave, the commit subject (`waymark: <verb> "<title>"`), an optional message body, and a short approval summary (title, folder, and the full body for `add` and `update_document`).
3. **Checks** run on the rendered result in a fixed order: `format::check` (all `error` findings), size, duplicate and near-duplicate titles (`titles.rs`), property links (`format::links` with the document set), secrets (`secrets.rs`). Findings from all checks are collected, so the agent fixes the call in one pass. Warnings (body links) go to `notices`.
4. **`add`.** Build the frontmatter in the order of the field table. `status: active`, dates, and authors are set. If `kb` has `verification = true`, add `trust: unverified`. Make the file name with `slug.rs`. With `supersedes`, read the old document, set `status: deprecated`, `updated`, and `updated_by`, and put both files in the plan.
5. **`update_document`.** Parse the current file, replace only the fields that were sent, and set `updated` and `updated_by`. If verification is on and the document is verified and `title`, `description`, or `body` changed, reset the trust fields. The body is carried over byte for byte if it was not sent.
6. **`dispute`, `deprecate`.** Set or remove the fields and set `updated` and `updated_by`. `deprecate` passes `Reason: <reason>` as the commit message body.
7. **`delete`.** Scan all documents for links to the target with `linkfix.rs`. Rewrite each changed document with `render`. Check the `folders` limit for every changed document. Remove the file. Report `links_fixed`.
8. **Titles** (`titles.rs`). Words are runs of Unicode alphanumeric characters, lowercased. Exact duplicate: equal word lists. Near-duplicate: the shared distinct words divided by the distinct words of the shorter title is at least 0.8, only if the shorter title has 3 or more words. The scan reads the `title` of every active document. It skips the target of `supersedes` and the document being updated.
9. **Preview.** The closure returns the plan. `Repo::write` with `preview: true` stops before approval and writes nothing. The tool builds `changes` and `text` from the plan.
10. **Result.** `ToolOutput` with `message` in plain words ("Added ... as `backend/payments/idempotency-keys`."), `notices`, and the `data` of the requirements. `version` is the short hash of the new commit for the changed file.

### What this feature needs from `git-sync`

These are small additions to the plan of `git-sync`. They are recorded in `decisions.md` and in the report.

- `Repo::write` takes a closure that runs inside the lock and returns the plan. It returns the commit hash, or none for a preview.
- `gitsync::message` supports an optional message body (lines after an empty line), for `Reason:`, `Move:`, and `Reverts:`.
- `format::links` returns the byte span of each link and says whether it is in the body or in a property.

## Cross-Feature Dependencies

- `write-approval` — the approval step of `Repo::write`. Tests for the approval need it. Until it exists, those tests are `#[ignore]`.
- `manual-edits` — commits pending edits before a write, inside `Repo::write`.
- `verification` — sets and resets trust fields. The code is in `add` and `update_document`, and it reads the setting only.
- `quality-checks` — `lint` reuses `secrets.rs` and the title rules. `lint` may report a title pair that the write tools let through, such as a replacement and its deprecated document.
- `move-documents` and `activity-and-undo` — reuse `linkfix.rs` and the `writes` helpers.

## Risks

- A link rewrite that damages text. Mitigation: `linkfix.rs` changes only the spans from `format::links`, never links in code fences or inline code, and tests compare the result byte for byte.
- False positives of the secret scan. Mitigation: the message names the rule and line, and `next` says to remove or replace the text with a placeholder. Patterns are few and exact.
- Near-duplicate false positives. Mitigation: the 3-word minimum, and the check ignores deprecated documents.
- YAML round trip changes a hand-written frontmatter. Mitigation: `render` after `parse` does not change a conforming document (a test in `document-format`).

## Open Questions

None.
