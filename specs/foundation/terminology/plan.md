# Plan: Terminology

## Overview

Fix the vocabulary in the writing conventions first, then sweep the text in
layers, from the most visible (README) to the least (comments), and rename the
identifiers that carry the old word. The project has no users, so old names
are replaced with no fallback.

## Components Affected

- `AGENTS.md` — add the vocabulary to "Writing conventions".
- `README.md` — new tagline, use-case section, and wording.
- `specs/PROJECT.md` — vision, personas, core concepts ("Knowledge base"
  becomes "Memory repository").
- `specs/INDEX.md` and every current spec (`requirements.md`, `plan.md`,
  `tasks.md`, `test.md`) — wording. About 370 occurrences.
- `docs/` — wording. `docs/similar-projects.md` keeps the old word where it
  describes other tools.
- `plain-git/` — the standalone skill. About 45 occurrences.
- `src/` — tool descriptions, error messages, the MCP server instructions, doc
  comments. About 80 occurrences.
- `tests/` — test names and quoted text. About 15 occurrences.
- `packaging/`, `scripts/`, `Makefile` — descriptions and messages.
- `src/mcp/binding.rs`, `src/mcp/repo_url.rs` — clone folder suffix
  `-agent-memory`.
- `standard/mkbase-1.0.md` and the conformance fixtures — renamed to the Waymark
  Memory Standard (`standard/waymark-1.0.md`), with the marker folder `.waymark/`.
  The document-format spec follows the standard.
- Every use of `.mkbase` and `MKBase` in `src/`, `tests/`, `specs/`, `docs/`,
  `plain-git/`, and `README.md`.
- `src/kb/agents_file.rs`, `src/kb/init.rs` — skill names, marker line, and the
  slug fallback.
- `plain-git/personal-knowledge-base/`, `plain-git/team-knowledge-base/` —
  renamed to `plain-git/personal-agent-memory/` and
  `plain-git/team-agent-memory/`, with the `name` in their frontmatter.

Not touched: `decisions.md` entries, project decisions, superseded specs,
`target/`, `dist/`, `tmp/`.

## New Files

- `specs/foundation/terminology/` — this spec (six files).

## Technical Approach

1. **Conventions first.** Write the three terms and the rules into
   `AGENTS.md`. Every later step reads them.
2. **Public face.** README (tagline, use-case section, wording) and
   `specs/PROJECT.md`.
3. **Specs and docs.** Apply the rules by hand with review of each file. A blind
   search and replace is wrong here, because the same phrase needs "memory"
   in one sentence and "memory repository" in the next (rule 8). Skip dated
   records (rule 6).
4. **Code strings.** Tool descriptions and errors first, with their tests in the
   same step. The settings error is the main contract:
   `This knowledge base needs a newer waymark.` becomes
   `This memory repository needs a newer waymark.` Tests match the substring
   `needs a newer waymark`, so they stay valid. Then the MCP server
   instructions, then comments.
5. **Templates that waymark writes.** The `AGENTS.md` and settings comments
   that `init` writes into a new memory repository. Check the fixtures and any
   test that compares them.
6. **Identifiers.** Clone folder suffix, skill folders and names, marker line,
   slug fallback, spec folders, and code names, each with its tests and links.
   The standard goes first in this layer: rename the standard and its fixtures,
   then the `document-format` spec, then the code that reads the marker folder
   and the settings file, then the tests. A change of the standard starts in the
   standard, as AGENTS.md requires.
   Update every relative link to a renamed spec folder.
7. **`plain-git/`** skill text.
8. **Final search** for "knowledge base" and run the full test suite.

## Dependencies on Other Foundation Items

- `knowledge-base-repo`, `mcp-server`, `distribution` — their text changes. They
  are `implemented`, so each change is a wording change that keeps the status.
  No spec reverts to `in-progress`.

## Risks

- A blind replace gives wrong sentences ("memory repository of the memory
  repository"). Review each file.
- A changed error message breaks a test that quotes more than the substring.
  Run the suite after each code step.
- Renaming spec folders breaks relative links. Search for every link to a renamed
  folder.
- The marker folder name is in many places (code, fixtures, tests, specs, docs).
  Search for `.mkbase` and `MKBase` and check that none is left.

## Open Questions

- None.
