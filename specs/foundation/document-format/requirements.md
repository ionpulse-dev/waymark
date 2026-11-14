---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-14
---

# Document format

## Technical Goal

Define what one document of a knowledge base looks like, so that every person, agent, and tool reads and writes it the same way. A knowledge base is a folder of Markdown files that is also a valid Obsidian vault.

The format is defined by the [MKBase Document Standard 1.0](../../../standard/mkbase-1.0.md). The standard is part of this project. It lives in `standard/` in this repository, together with its conformance fixtures, and it is maintained here. Any other tool may use it. This item implements the standard. If the two differ, the standard wins, and this item changes to match. A format change is made in the standard first.

## Dependents

- `knowledge-base-repo` — the repository layout holds these documents.
- `mcp-server` — every read and write tool uses this format.
- `quality-checks` — `lint` checks documents against this format.
- `plain-git-access` — the plain-Git rules teach the same format to agents without waymark.

## Deliverables

- The frontmatter schema, with types and rules.
- The rules for document IDs, file names, and links.
- The size limits and the counting rules.
- The Obsidian compatibility rules.
- The MKBase Document Standard 1.0 and its conformance fixtures in `standard/` (see [standard/mkbase-1.0.md](../../../standard/mkbase-1.0.md)).
- A sample vault used by tests and by the check in Obsidian before each release.

## Scope

**In scope:**
- One document: file name, frontmatter, body, links, sizes.
- The meaning of status, disputed, and trust fields.

**Out of scope:**
- The folder layout of a repository (see `knowledge-base-repo`).
- Binary data. A document describes it and links to it.

## Constraints

- Documents are UTF-8 Markdown with LF line ends and the `.md` extension.
- File names use lowercase letters, digits, and `-` only. They never contain `# ^ [ ] | : * " < > ? \ /`.
- Frontmatter is YAML, because Obsidian reads properties only from YAML.
- All properties are flat: text, number, checkbox, date (`YYYY-MM-DD`), or a list of strings. There are no nested objects, because the Obsidian property editor cannot edit them.
- One topic per document.

### Frontmatter fields

| Field | Required | Meaning |
|---|---|---|
| `title` | yes | One sentence, at most 15 words. |
| `description` | yes | What the document contains and when to read it. At most 50 words. |
| `status` | yes | `active` or `deprecated`. |
| `created`, `updated` | yes | Dates. Set by waymark. |
| `created_by`, `updated_by` | yes | Text `"Name <email>"`. Set by waymark. Git history stays the source of truth. |
| `disputed` | no | Checkbox. `true` when someone found the document wrong. |
| `dispute_reason` | no | Text. Set together with `disputed: true`. |
| `sources` | no | List of strings: ticket IDs or URLs. |
| `tags` | no | List of strings, no `#`, no spaces. |
| `related` | no | List of full-path wikilinks: read these too. |
| `supersedes` | no | List of full-path wikilinks: this document replaces an older one. The older one becomes `deprecated`. |
| `anchors` | no | List of strings `"<type>:<target>"` that point to the file or page the document describes. Types: `git` (`<repo>/<path>@<commit>`) and `url`. |
| `trust`, `verified_by`, `verified` | no | Only when the knowledge base turns on verification. `trust` is `unverified` or `verified`. |

### Document ID and links

- The ID of a document is its path below the root of the knowledge base without `.md`. The root is the folder that contains the marker folder `.mkbase` (in a waymark repository: `knowledge/`), for example `backend/payments/idempotency-keys`. A file name is a short form of the title. It does not change only because the title changed. A rename is a move: it changes the ID and follows the rules of `move-documents`.
- Links in the body are relative Markdown links with `.md`, or wikilinks. waymark reads both and writes relative Markdown links.
- Links in properties are wikilinks with the full path, for example `"[[backend/payments/retry-policy]]"`, so they are clickable and show in the graph.
- Backlinks are computed on demand. No document stores its backlinks, and there is no central index file.

### Size limits

| Part | Default | Setting |
|---|---|---|
| `title` | 15 words | `documents.title_max_words` |
| `description` | 50 words | `documents.description_max_words` |
| Whole document | 100 KB, hard cap | `documents.max_bytes` |

There is no limit on words in the body. Headings are allowed. A full architecture decision record fits in one document. Counting rules for words, characters, and estimated tokens (words times a fixed factor) are fixed here so that the server and `lint` agree.

### Obsidian compatibility

- Callouts, embeds, highlights, and comments written by people are allowed. `lint` does not reject them, and word counts ignore their markup.
- A person can edit documents in Obsidian. These edits bypass the write checks. `lint` reports problems later.
- Tags are a `tags` property, not inline `#tags` written by waymark.

## Acceptance Criteria

- **Given** a document that waymark wrote, **When** it is opened in the current Obsidian version, **Then** the properties, links, graph, and tags show without errors.
- **Given** a document with a nested frontmatter object, **When** `lint` runs, **Then** it fails the document.
- **Given** a file name with an uppercase letter or a forbidden character, **When** `lint` runs, **Then** it fails the document.
- **Given** a document without `description`, **When** it is written through waymark, **Then** the write fails with `validation_failed`.
- **Given** a document over the hard cap, **When** it is written, **Then** the write fails and tells the agent to split the document or move details into a source.
- **Given** a title change, **When** the document is saved, **Then** the file name and the ID stay the same.
- **Given** a sample vault, **When** the test suite runs, **Then** every document in it passes `lint`.
- **Given** the conformance fixtures in `standard/conformance/`, **When** the test suite runs, **Then** every valid document passes `lint` and every invalid document fails it with a finding that names the broken rule.

## Open Questions

- Which Obsidian version is the baseline for the release check?
