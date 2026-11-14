# Tests: Quality checks

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The feature moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a broken relative link in the body, **When** `lint` runs, **Then** it reports the file and target as a warning. | T1 |
| **Given** a broken link in `related`, **When** `lint` runs, **Then** it reports an error. | T2 |
| **Given** a custom pattern, **When** a document matches it, **Then** a write fails with `validation_failed` and `lint` reports `secret`. | T3 |
| **Given** a long title from Obsidian, **When** `lint` runs, **Then** it reports `title-too-long`. | T4 |
| **Given** `created_by` that differs from Git history, **When** `lint` runs, **Then** it warns. | T5 |
| **Given** a clean repository, **When** `lint` runs, **Then** it returns no findings. | T6 |
| **Given** a callout or an embed, **When** `lint` runs, **Then** it does not reject it. | T7 |
| **Given** a private key block, **When** a write tool runs, **Then** it fails and the message does not repeat the key. | T8 |
| **Given** two documents with the same title, **When** `lint` runs, **Then** the newer one has `duplicate-title`. | T9 |
| **Given** an IBAN in a body, **When** `lint` runs, **Then** it reports `personal-data`. | T10 |
| **Given** the conformance fixtures, **When** `lint` runs, **Then** valid ones have no error and invalid ones the named rule. | T11 |
| **Given** `people.email_domains` and an author with another domain, **When** `lint` runs, **Then** it reports `author-not-allowed`. | T12 |
| **Given** a knowledge base, **When** `index` runs, **Then** each folder gets an index file with links. | T13 |
| **Given** an unchanged knowledge base, **When** `index` runs twice, **Then** the second run makes no commit. | T14 |
| **Given** a hand-written `index.md`, **When** `index` runs, **Then** it is unchanged and listed under `skipped`. | T15 |
| **Given** `preview: true`, **When** `index` runs, **Then** nothing is written. | T16 |
| **Given** `access = "read-only"`, **When** the client lists tools, **Then** `index` is hidden and `lint` is listed. | T17 |
| **Given** a generated index that a new document makes stale, **When** `lint` runs, **Then** it warns `index-stale`. | T18 |

## Test Cases

Unit tests are in the files of `src/lint/` and `src/indexgen/`. End-to-end tests are in `tests/it_quality_checks.rs` and use a `World` and an `McpClient`.

- [x] [auto] T1: a body link to a missing file gives `broken-link` with severity `warning` and the target in the message.
- [x] [auto] T2: `related` with a missing document gives `broken-link` with severity `error`.
- [x] [auto] T3: a shared setting `extra_secret_patterns = ["CUST-[0-9]{6}"]`. `add` (or a pending edit) with `CUST-123456` fails with `validation_failed`. `lint` on a committed document with it reports `secret`.
- [x] [auto] T4: a document with a 16-word title (committed with plain `git`) gives `title-too-long`.
- [x] [auto] T5: a document whose `created_by` email differs from its first commit author gives `author-mismatch` as a warning. A file with no history gives none.
- [x] [auto] T6: the sample vault of `standard/conformance/vault/` gives `findings: []` and `errors: 0`.
- [x] [auto] T7: a document with a callout, an embed, a highlight, and a `%%` comment gives no finding.
- [x] [auto] T8: `add` with a private key block fails with `validation_failed`. The `message` and `next` do not contain the key text.
- [x] [auto] T9: two documents with one title: the one with the later `created` has `duplicate-title` and the message names the other.
- [x] [auto] T10: a body with `DE89 3704 0044 0532 0130 00` (a valid test IBAN) gives `personal-data`. A made-up number with a wrong check digit gives none.
- [x] [auto] T11: `tests/lint_conformance.rs` over `vault/` and `invalid/`.
- [x] [auto] T12: with `people.email_domains = ["example.org"]`, an author `Jo <jo@other.test>` gives `author-not-allowed`.
- [x] [auto] T13: `index` creates `index.md` and `backend/index.md`. Each lists its documents with relative links that resolve, and the folder links to the subfolder index.
- [x] [auto] T14: a second `index` run returns `written: []` and the commit count does not grow.
- [x] [auto] T15: a document `ops/index.md` without the tag `index` is unchanged and listed under `skipped`.
- [x] [auto] T16: `index` with `preview: true` changes no file and makes no commit, and `written` lists the planned files.
- [x] [auto] T17: with `access = "read-only"`, `tools/list` has `lint` and not `index`.
- [x] [auto] T18: after `index`, a new document is added. `lint` reports `index-stale` as a warning for the folder index.
- [x] [auto] T19: `index` commits with the message `waymark: update "Index of backend"` for one folder.
- [x] [auto] T20: a placeholder `password = <password>` gives no finding. `password = hunter2hunter2` gives `secret`.
- [x] [auto] T21: a bad extra pattern `[` makes `lint` fail with `setup_problem` and a `next` that names it.
- [x] [auto] T22: 150 findings: the result has 100 and `truncated: true`, errors first.
- [x] [auto] T23: `lint` with `folders: ["ops"]` reports only files below `ops`.
- [x] [auto] T24: a secret inside a fenced code block is a finding.

## Notes

- Test secrets are fake and built from the patterns, for example a key made of `AKIA` and 16 digits. The fixture files hold no real credential.
- No `[manual]` item.
