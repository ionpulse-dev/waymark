---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# Quality checks

## User Story

> As a person, I want every document to be checked for format, size, links, and secrets, so that the knowledge base stays clean and safe.

## Personas

- **Every person** — gets a clear message when a document fails.
- **Knowledge base owner** — adds company-specific secret patterns.

## Behaviour

| Command | Task |
|---|---|
| `waymark lint` | Checks Obsidian compatibility, the frontmatter schema, file location and names, size limits, broken links, duplicate titles, the author fields against Git history, and secrets and personal data with built-in patterns. Runs automatically before every commit that waymark makes. Can run on demand for the whole repository or some folders. |
| `waymark index` | Generates index files (per domain and per team). Nobody edits index files by hand. There is no central index that writers edit. |

A failed check stops a write with exit code 2 and a message in plain words. `lint` also runs on manual edits (see `manual-edits`).

Secret scan: built-in patterns for common credentials. A knowledge base adds its own patterns as regular expressions in `lint.extra_secret_patterns`, for example for IBAN or customer ID formats.

## Interfaces & Contracts

`lint --json` returns a list of findings: `path`, `rule`, `severity`, `message`.

## Acceptance Criteria

- **Given** a broken relative link, **When** `lint` runs, **Then** it reports the file and the target.
- **Given** a custom pattern, **When** a document matches it, **Then** the write fails with exit code 2.
- **Given** a document edited in Obsidian with a long title, **When** `lint` runs, **Then** it reports the title limit.
- **Given** `created_by` that differs from Git history, **When** `lint` runs, **Then** it warns.
- **Given** a clean repository, **When** `lint` runs, **Then** it exits 0.
- **Given** a callout or an embed written by a person, **When** `lint` runs, **Then** it does not reject it.

## Constraints & Edge Cases

- Secrets and customer data never belong in a knowledge base. The scan reduces the risk but cannot guarantee it.
- CI on the Git host may run `lint` too. This is optional.

## Open Questions

- Is CI on the Git host allowed in the target organizations?
