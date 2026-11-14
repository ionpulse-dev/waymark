# Tasks: Document format

_Written after `plan.md` was approved. Strict prerequisite-first order. The full test suite runs after every task._

`requires`: none.

## Setup

- [x] Check the release dates of `yaml-rust2` and `saphyr` on crates.io. Record the result in `decisions.md`. If `yaml-rust2` is abandoned, change the decision before any code.
- [x] Create `Cargo.toml` (package `waymark`, dependency `yaml-rust2` at the checked version) and `src/lib.rs` with `mod format;`.
- [x] Add `rustfmt` and `clippy` to the checks that run with the test suite.

## Building blocks

_These four have no dependency on each other and may run in parallel._

- [x] `src/format/finding.rs`: `Finding { path, rule, severity, message }` with `Severity::{Error, Warning}`. Rule names are the file names of `standard/conformance/invalid/`.
- [x] `src/format/limits.rs`: `Limits` with the defaults 15, 50, and 102400.
- [x] `src/format/count.rs`: bytes, words, markup removal (with the unclosed `%%` rule), and estimated tokens, as in section 9.1 of the standard.
- [x] `src/format/id.rs`: document ID from a path, and the file-name rules for the file and every folder.

## Tests of the building blocks

- [x] `tests/format_counting.rs`: word boundaries (15/16 and 50/51), markup removal for each kind, unclosed `%%`, non-ASCII text, and the token estimate.
- [x] Unit tests in `id.rs`: uppercase, each forbidden character, a valid name, and an ID that depends only on the path.

## Parsing and checks

- [x] `src/format/frontmatter.rs`: split at the two `---` lines, parse with `yaml-rust2`, convert to a flat map. Findings for no frontmatter, unclosed frontmatter, and nested values. Handle `yes`/`no`, unquoted dates, and a number as a title.
- [x] `src/format/fields.rs`: the field table, with required fields, types, `status` values, dates, `updated` not before `created`, the `"Name <email>"` format, and `disputed` with `dispute_reason`. Unknown fields are kept.
- [x] `src/format/links.rs`: find body links and property wikilinks. Check the form. With a document set, check that each target exists: error in a property, warning in the body.
- [x] `src/format/mod.rs`: `parse` and `check` in the fixed order (parse, fields, names, size, links). Return all findings.
- [x] `src/format/mod.rs`: `render`: fields in table order, `YYYY-MM-DD`, LF, the body unchanged.

## Verification

- [x] `tests/format_conformance.rs`: every file of `standard/conformance/vault/` passes with no error. Every file of `standard/conformance/invalid/` fails with the rule named by its file name.
- [x] Round-trip test: `render(parse(doc))` equals `doc` for every vault document.
- [x] Boundary tests: generated documents of 102400 and 102401 bytes. The failure message tells the agent to split the document or move details into a source.
- [x] Link severity tests: a missing `related` or `supersedes` target is an error, and a missing body target is a warning.
- [x] Run the full test suite, `cargo fmt --check`, and `cargo clippy`. Check off the `[auto]` items in `test.md`.
- [x] Update `status` in `requirements.md` and `INDEX.md`, and update `Next Up`.
