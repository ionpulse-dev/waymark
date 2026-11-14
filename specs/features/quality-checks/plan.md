# Plan: Quality checks

## Overview

A `lint` module runs the checks of `format` and adds the checks that need more than one document or Git. It offers two entry points: `lint::run` for a set of documents (the tool) and `lint::check_candidate` for the text that a write is about to commit (write tools and `sync`). A generator `indexgen` builds index files. Two tools, `lint` and `index`, call them.

## Components Affected

- `format` — `check` gives the single-document findings. `count` gives words and bytes. No change.
- `store` — `Store` provides all documents. This spec adds no change.
- `kb` — `kb::settings` gives `documents.*`, `people.*`, and `lint.extra_secret_patterns`.
- `gitsync` — `gitsync::log` gets `first_authors()` and `last_authors()` (one `git log` pass each). `Repo::write` runs the checks of this spec in step 5.
- `mcp` — the registry gets two tools. `index` is a write tool, `lint` is read-only.
- Write tools of `write-documents`, `move-documents`, and `manual-edits` call `lint::check_candidate`.

## New Files

- `src/lint/mod.rs` — `run(store, scope, ctx) -> Report`, `check_candidate(...)`, and the conversion of a finding list to a `ToolError`.
- `src/lint/secrets.rs` — the built-in patterns and the extra patterns (`regex` crate). The finding never holds the matched text.
- `src/lint/personal.rs` — IBAN (with the mod 97 check) and payment card numbers (Luhn).
- `src/lint/duplicates.rs` — the title index and `find_title(store, title, except_id)`.
- `src/lint/authors.rs` — `author-mismatch` and `author-not-allowed`.
- `src/lint/index_stale.rs` — compares each generated index with the output of `indexgen`.
- `src/indexgen/mod.rs` — builds the index document of one folder and decides what to write.
- `src/tools/lint.rs` — the `lint` tool.
- `src/tools/index.rs` — the `index` tool.
- `tests/it_quality_checks.rs` — end-to-end tests.
- `tests/lint_conformance.rs` — runs `lint::run` over `standard/conformance/`.

## Technical Approach

1. **Run.** `lint::run` loads the `Store`, applies the `Scope` (limit and `folders`), and for each document collects: the parse findings and `format::check` findings (with the settings limits), then secrets, personal data, duplicates, authors, and stale indexes. `broken-link` comes from `format::links` with the full document set: error for a property link, warning for a body link.
2. **Secrets.** Patterns are regular expressions compiled once. Built in: `-----BEGIN [A-Z ]*PRIVATE KEY-----`, `AKIA[0-9A-Z]{16}`, `gh[pousr]_[A-Za-z0-9]{36,}`, `xox[abprs]-[A-Za-z0-9-]{10,}`, `sk-[A-Za-z0-9_-]{20,}`, `eyJ[A-Za-z0-9_-]{10,}\.eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}`, `[a-z][a-z0-9+.-]*://[^\s/:@]+:[^\s/@<>*]+@`, and a case-insensitive assignment `(password|passwd|secret|api[_-]?key|token)\s*[:=]\s*["']?[^\s"'<>*]{8,}`. A value that holds `<`, `>`, `*`, `example`, or `placeholder` is skipped. The finding says "Line N looks like a secret (<pattern name>)."
3. **Personal data.** A run of 15 to 34 letters and digits that starts with two letters and two digits is checked with mod 97. A run of 13 to 19 digits (spaces and dashes allowed) is checked with Luhn.
4. **Duplicates.** Normalize titles (lowercase, collapse white space). Group them. In each group of two or more, sort by `created`, then `id`. All but the first get a finding that names the first.
5. **Authors.** `gitsync::log::first_authors` makes one pass of `git log --reverse --name-status -M` and builds a map from path to the author of the adding commit. A rename moves the entry. `last_authors` gives the author of the last commit of each path. A file with pending edits or no history is skipped. Emails are compared in lowercase. `author-not-allowed` uses `people.*` and needs no Git.
6. **Candidate check.** `check_candidate(path, bytes, store, settings)` parses the bytes and runs the single-document checks, the secret and personal data scan, the duplicate check (`find_title`), and `broken-link` against the store. It returns findings. `into_error(findings)` returns `ToolError::validation_failed` with the first error as `message` and a mapped `next` sentence (a table from rule to sentence).
7. **Sizes.** The finding `document-too-large` says the document is over the cap and the `next` says to split it.
8. **Index generation.** `indexgen::plan(store, scope)` walks the folders in scope. For each folder with documents or subfolders it renders the document with `format::render`: frontmatter, the marker line, the subfolder links, and the document links. It compares the body with the existing file. An existing file that has the same body is `unchanged`. An existing file without the tag `index` is `skipped`. Else it is `written`. `updated` and `updated_by` are set only when the body changes. A new file gets all four author fields.
9. **Index write.** The `index` tool runs `Repo::write` once for all files, verb `update`, with the title "Index of <folder>" for one folder, else "Indexes". The checks run on each file.
10. **Tool visibility.** `lint` is kind read. `index` is kind write, so the registry hides it when `access` is `read-only` or the server is paused.
11. **Descriptions.** `lint`: "Checks documents and lists findings." `index`: "Generates index files for folders."
12. **Settings.** `lint.extra_secret_patterns` is read from the shared file. A bad expression is `setup_problem` and names the entry.

## Cross-Feature Dependencies

- `document-format` — rule names are the contract. This spec adds the names listed in the requirements. The file names of `standard/conformance/invalid/` stay the rule names.
- `git-sync` — `Repo::write` step 5 calls `lint::check_candidate`. This spec adds `first_authors` and `last_authors` to `gitsync::log`.
- `write-documents`, `move-documents` — call `check_candidate` and `find_title`.
- `manual-edits` — calls `lint::run` on the pending files.
- `search-and-read` — `Store` and the index-file rule.

## Risks

- A false secret finding blocks a good document. Mitigation: the placeholder skip, a message with the line number, and the `next` sentence to rewrite the value. A knowledge base cannot turn built-in patterns off in the first release (see `decisions.md`).
- A false personal-data finding on a long number. Mitigation: IBAN and Luhn checks.
- Slow author checks in large histories. Mitigation: one `git log` pass, run only by `lint`, not by writes.
- Index files cause many commits in big knowledge bases. Mitigation: unchanged files are not touched, and `index` runs only on demand.

## Open Questions

None.
