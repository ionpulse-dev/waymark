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
| **Given** an IBAN in a body, **When** `lint` runs, **Then** it reports `financial-data` as a warning. | T10 |
| **Given** the conformance fixtures, **When** `lint` runs, **Then** valid ones have no error and invalid ones the named rule. | T11 |
| **Given** `people.email_domains` and an author with another domain, **When** `lint` runs, **Then** it reports `author-not-allowed`. | T12 |
| **Given** `access = "read-only"`, **When** the client lists tools, **Then** `lint` is listed, and there is no `index` tool and no `index-stale` (see `remove-index`). | T17 |
| **Given** an IBAN in a body, **When** a write through waymark runs, **Then** the write succeeds and the result has a notice about it. | T25 |

## Test Cases

Unit tests are in the files of `src/lint/`. End-to-end tests are in `tests/it_quality_checks.rs` and use a `World` and an `McpClient`.

- [x] [auto] T1: a body link to a missing file gives `broken-link` with severity `warning` and the target in the message.
- [x] [auto] T2: `related` with a missing document gives `broken-link` with severity `error`.
- [x] [auto] T3: a shared setting `extra_secret_patterns = ["CUST-[0-9]{6}"]`. `add_document` (or a pending edit) with `CUST-123456` fails with `validation_failed`. `lint` on a committed document with it reports `secret`.
- [x] [auto] T4: a document with a 16-word title (committed with plain `git`) gives `title-too-long`.
- [x] [auto] T5: a document whose `created_by` email differs from its first commit author gives `author-mismatch` as a warning. A file with no history gives none.
- [x] [auto] T6: the sample vault of `standard/conformance/vault/` gives `findings: []` and `errors: 0`.
- [x] [auto] T7: a document with a callout, an embed, a highlight, and a `%%` comment gives no finding.
- [x] [auto] T8: `add_document` with a private key block fails with `validation_failed`. The `message` and `next` do not contain the key text.
- [x] [auto] T9: two documents with one title: the one with the later `created` has `duplicate-title` and the message names the other.
- [x] [auto] T10: a body with `DE89 3704 0044 0532 0130 00` (a valid test IBAN) gives `financial-data` with severity `warning`. A made-up number with a wrong check digit gives none.
- [x] [auto] T11: `tests/lint_conformance.rs` over `vault/` and `invalid/`.
- [x] [auto] T12: with `people.email_domains = ["example.org"]`, an author `Jo <jo@other.test>` gives `author-not-allowed`.
- [x] [auto] T17: `tools/list` has `lint` and no `index`, with `read-write` and with `read-only`. A call of `index` is a protocol error. `lint` never reports `index-stale`.
- [x] [auto] T20: a placeholder `password = <password>` gives no finding. `password = hunter2hunter2` gives `secret`.
- [x] [auto] T21: a bad extra pattern `[` makes `lint` fail with `setup_problem` and a `next` that names it.
- [x] [auto] T22: 150 findings: the result has 100 and `truncated: true`, errors first.
- [x] [auto] T23: `lint` with `folders: ["ops"]` reports only files below `ops`.
- [x] [auto] T24: a secret inside a fenced code block is a finding.
- [x] [auto] T25: `add_document` with a valid IBAN in the body succeeds (no `validation_failed`) and the result has a notice that names the line. A private key in the same call still fails.

## Notes

- Test secrets are fake and built from the patterns, for example a key made of `AKIA` and 16 digits. The fixture files hold no real credential.
- No `[manual]` item.
