---
type: foundation
status: implemented
created: 2026-11-14
updated: 2026-11-14
---

# Remove Index

## Technical Goal

waymark is long-term agent memory, and mostly agent storage. The `index` tool
generates `index.md` files for people who browse a memory repository. It
duplicates `folders`, `list`, and `search`, it adds a write tool to the context
of every agent, and it forces special cases in six other tools. This spec
removes the tool and every special rule for generated index files.

## Dependents

- `quality-checks`: loses the `index` tool and the `index-stale` check.
- `search-and-read`, `updates`, `folders`, `write-documents`: lose the rule that
  skips generated index files.
- `pause-and-resume`, `write-approval`, `mcp-server`: lose `index` in their
  tool lists.
- README, `docs/install.md`: lose `index` in the tool table and the allow list.

## Scope

**Removed:**
- The tool `index`, the module that builds index files, and the `index` tool
  description.
- The lint check `index-stale`.
- The rule "a file `index.md` with the tag `index` is a generated index file".
  After this spec, `index.md` is a normal document and the tag `index` has no
  special meaning. `search`, `list`, `stats`, backlinks, `updates`, `delete`,
  `folders`, and `remove_folder` treat it like every other document.

**Kept:**
- The sentence in the Waymark Memory Standard that a tool MAY generate index
  files. It is permission, not a requirement.
- The rule "no central index file that writers edit" (see `document-format`).
  The plain-Git skills keep "Do not write a central index file".
- A future "memory browser" (see `docs/ideas.md`) can give people the map of a
  memory repository, without a file and without a write tool.

## Constraints

- The project has no users yet. No migration and no fallback. A generated
  `index.md` that exists in a memory repository is a normal document from now on.
  The owner can keep it or delete it.
- Dated records are not rewritten: the old entries in `decisions.md` stay. New
  entries record the removal.
- This is a breaking change of the tool list, so the next release is 0.3.0.

## Acceptance Criteria

- **Given** a waymark build, **When** a client lists the tools, **Then** `index`
  is not listed, with any `access` setting.
- **Given** a call of `index`, **When** the server handles it, **Then** it fails
  as an unknown tool.
- **Given** a document `ops/index.md` with the tag `index`, **When** `search`,
  `list`, `stats`, and `updates` run, **Then** it is treated like any other
  document.
- **Given** a document that links to `ops/index`, **When** it is deleted, **Then**
  `delete` refuses it like any linked document.
- **Given** `lint`, **When** it runs, **Then** no result has the rule
  `index-stale`.
- **Given** the specs, README, and docs, **When** a reader looks for the `index`
  tool, **Then** only dated records and this spec mention it.
- **Given** the full test suite, **When** it runs, **Then** it passes.

## Open Questions

_None._

## Decided

- The standard keeps its "MAY generate index files" sentence.
- A tag `index` has no meaning. No check warns about it.
- The release is 0.3.0.
