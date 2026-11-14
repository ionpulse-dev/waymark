# Tests: Document format

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

_AI checks off `[auto]` items when tests pass. A human checks off `[manual]` items. The foundation item moves to `done` only when all items are checked._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| **Given** a document that waymark wrote, **When** it is opened in the current Obsidian version, **Then** the properties, links, graph, and tags show without errors. | Obsidian check of the sample vault (manual) |
| **Given** a document with a nested frontmatter object, **When** `lint` runs, **Then** it fails the document. | Conformance test, `invalid/nested-property.md` |
| **Given** a file name with an uppercase letter or a forbidden character, **When** `lint` runs, **Then** it fails the document. | File-name unit tests in `id.rs` |
| **Given** a document without `description`, **When** it is written through waymark, **Then** the write fails with `validation_failed`. | Conformance test, `invalid/missing-description.md` (error finding). The mapping to `validation_failed` is tested in `mcp-server`. |
| **Given** a document over the hard cap, **When** it is written, **Then** the write fails and tells the agent to split the document or move details into a source. | Hard-cap boundary test |
| **Given** a title change, **When** the document is saved, **Then** the file name and the ID stay the same. | ID unit test (the ID depends only on the path). The save flow is tested in `mcp-server`. |
| **Given** a title of 16 words or a description of 51 words, **When** `lint` runs, **Then** it fails the document. **Given** exactly 15 and 50 words, **Then** it passes. | Word-limit boundary tests, and `invalid/title-too-long.md`, `invalid/description-too-long.md` |
| **Given** a body with callouts, embeds, highlights, and comments, **When** words are counted, **Then** the markup does not add words. | Counting tests |
| **Given** a `related` or `supersedes` link to a missing document, **When** a document is written through waymark, **Then** the write fails with `validation_failed`. | Link severity test (error finding). The mapping is tested in `mcp-server`. |
| **Given** a body link to a missing document, **When** a document is written through waymark, **Then** the write succeeds and `lint` reports a warning. | Link severity test (warning finding) |
| **Given** a sample vault, **When** the test suite runs, **Then** every document in it passes `lint`. | Conformance test, `vault/` |
| **Given** the conformance fixtures in `standard/conformance/`, **When** the test suite runs, **Then** every valid document passes `lint` and every invalid document fails it with a finding that names the broken rule. | Conformance test, `vault/` and `invalid/` |

## Test Cases

### Automated

- [ ] [auto] Every document of `standard/conformance/vault/` passes with no error finding (`tests/format_conformance.rs`)
- [ ] [auto] Every document of `standard/conformance/invalid/` fails, and the finding names the rule given by the file name (`tests/format_conformance.rs`)
- [ ] [auto] `render(parse(doc))` equals `doc` for every vault document (`tests/format_conformance.rs`)
- [ ] [auto] Words: titles of 15 and 16 words and descriptions of 50 and 51 words (`tests/format_counting.rs`)
- [ ] [auto] Words: a callout, an embed, a highlight, and a comment add no markup words. The text inside a callout, embed, or highlight counts. A comment's text does not (`tests/format_counting.rs`)
- [ ] [auto] Words: an unclosed `%%` hides the text to the end of the document (`tests/format_counting.rs`)
- [ ] [auto] Words: non-ASCII text and tokens equal words times 1.5, rounded up (`tests/format_counting.rs`)
- [ ] [auto] A generated document of 102400 bytes passes. One of 102401 bytes fails, and the message says to split the document or move details into a source (`tests/format_conformance.rs`)
- [ ] [auto] File names: uppercase and each of `# ^ [ ] | : * " < > ? \ /` fail, for the file and for a folder. A valid name passes (`src/format/id.rs`)
- [ ] [auto] The ID is the path without `.md` and does not depend on the title (`src/format/id.rs`)
- [ ] [auto] A missing `related` or `supersedes` target is an error. A missing body target is a warning (`tests/format_conformance.rs`)
- [ ] [auto] YAML traps: `yes` and `no` are not booleans, an unquoted date is read as a date, and a number as a title is a type finding (`src/format/frontmatter.rs`)
- [ ] [auto] All findings are returned for a document with several problems, not only the first (`tests/format_conformance.rs`)
- [ ] [auto] Unknown frontmatter fields are kept by `parse` and `render` (`src/format/fields.rs`)

### Manual

- [ ] [manual] The sample vault opens in the latest stable Obsidian desktop release and in Obsidian 1.4 (the floor).
  - Setup: install both versions. Open `standard/conformance/vault/` with "Open folder as vault". Use a copy of the folder if Obsidian writes files into it.
  - Steps: (1) Open `backend/payments/idempotency-keys.md`. Check that the properties panel shows all fields, and that `tags` and `related` show as lists. (2) Click a `related` link. It must open the linked document. (3) Open the graph view. Links between documents must show. (4) Open the document with callouts. Callouts, highlights, and embeds must render. (5) Check in the Obsidian changelog that 1.4 is the first version with the current property editor.
  - Expected: no error or warning in the developer console, no property shown as invalid, and every link and the graph work.
  - Teardown: delete the copy. Write the Obsidian versions and the result in the release notes.
  - When: at the first release, not before (see `decisions.md`).

## Notes

- The tests in this item check the findings of the `format` module. The mapping of an error finding to `validation_failed`, the save flow, and `lint` as a command are tested in `mcp-server` and `quality-checks`.
- Run the suite with `cargo test`. The tests read the fixtures from `standard/conformance/`.
