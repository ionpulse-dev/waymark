---
type: foundation
status: in-progress
created: 2026-11-14
updated: 2026-11-14
---

# Terminology

## Technical Goal

waymark is long-term agent memory. Today the project calls the thing that
the user creates "a knowledge base". That word names only one use case. This
spec sets one vocabulary for the whole project and the rules to apply it, so
that the README, the specs, the docs, and the text that agents read in the code
all say the same thing.

## Dependents

- Every spec, doc, and user-visible string. They all use the words that this
  spec fixes.
- `knowledge-base-repo`, `create-knowledge-base`, `join-knowledge-base`: the
  folder names change with this spec (see "Scope").
- `distribution`: the formula description and the release notes use the tagline.

## Vocabulary

| Term | Meaning | Use it for |
|:-----|:--------|:-----------|
| **agent memory** | The concept: what outlives the short context window of an agent. | Headlines, the tagline, positioning. In running text, "memory" alone is fine when the context is clear. |
| **memory repository** | The Git repository that holds the memory. It has a URL, a history, and can be cloned. | Setup steps, tool descriptions, error messages, the settings and sync specs, multi-tenancy. Plural: "memory repositories". |
| **document** | One Markdown file in a memory repository. The unit of the memory. | Unchanged. Never "memory" for one document, and never "memories" for documents. |

Tagline: **Long-term agent memory.**

## Rules

1. **Write "memory repository" in full.** No "memory repo" in prose. Short
   identifiers (module names, variables) may be shorter.
2. **"Repository" alone means the code repository.** The old term "content
   repository" becomes "memory repository".
3. **"Knowledge base" is a use case, never the name of the thing.** It may
   appear only (a) as a named use case, for example "use a memory repository as
   your team's knowledge base", and (b) when text describes another tool or the
   wider field (for example in `docs/similar-projects.md`).
4. **"Knowledge" stays a word for content.** Decisions, contracts, and
   conventions are knowledge. Preferences and learnings of an agent are memory.
   Both live in one memory repository.
5. **Persona:** "Knowledge base owner" becomes "Memory owner".
6. **Dated records are not rewritten.** Entries in `decisions.md` files, the
   project decisions, and the superseded specs keep their words, because they
   record what was said at that time. A new entry may point to this spec.
7. **Normative text is rewritten.** `requirements.md`, `plan.md`, `tasks.md`,
   and `test.md` of every current spec describe the present state, so they use
   the new words.
8. **Error messages and tool descriptions** use "memory repository" where they
   name the container, and "memory" where they name the idea. Agents read them on
   every call, so they stay short.

## README structure

- The first sentence is the tagline and a one-sentence explanation: plain
  Markdown in your own Git repository, shared by your agents, your team, and you.
- A section "What you can build with it" lists three use cases, in this order:
  1. **A team knowledge base**: decisions, API contracts, owners, known
     problems. This is the most prominent example and carries the current
     "The problem" text.
  2. **A personal memory**: what each AI tool learned about you, shared across
     tools.
  3. **An agent's working memory**: what a long-running agent leaves for its
     next session.
- Everywhere else the README uses "memory" and "memory repository".

## Deliverables

- This vocabulary, written into the writing conventions of
  [AGENTS.md](../../../AGENTS.md) next to "Say 'document', not 'note'".
- README, `specs/PROJECT.md`, and `docs/` use the vocabulary.
- All current specs use the vocabulary in their normative files.
- Code: tool descriptions, error messages, the MCP server instructions, doc
  comments, and the text of tests use the vocabulary.
- The standalone `plain-git/` skill and the template text that waymark writes
  into a new memory repository (`AGENTS.md`, settings file comments) use the
  vocabulary.
- Renamed identifiers (see "Scope"): the local clone folder, the two plain-Git
  skills, and the slug fallback.

## Scope

**In scope:**
- All prose: README, specs, docs, `plain-git/`, `packaging/`, `scripts/`.
- All user-visible strings in `src/`, and the doc comments.
- Test names and test text that quote the old words.
- The text of templates that waymark writes into a user's repository.

- Identifiers that carry the old word. The project is not used anywhere yet, so
  there is no migration and no fallback for old names:
  - The default local clone folder: `~/Projects/<slug>-knowledge-base` becomes
    `~/Projects/<slug>-agent-memory`. The search for existing clones looks for
    the new suffix only.
  - The two plain-Git skills: `personal-knowledge-base` and
    `team-knowledge-base` become `personal-agent-memory` and
    `team-agent-memory`. This covers the folders in `plain-git/`, the `name` in
    their frontmatter, the marker line that `init` writes into `AGENTS.md`, the
    code, and the tests.
  - The slug fallback `knowledge-base` in `init`.
  - The spec folders: `knowledge-base-repo` becomes `memory-repository`,
    `create-knowledge-base` becomes `create-memory-repository`, and
    `join-knowledge-base` becomes `join-memory-repository`. The test files
    follow: `it_memory_repository.rs`, `it_create_memory_repository.rs`,
    `it_join_memory_repository.rs`, and `tests/common/kb.rs` becomes
    `tests/common/memrepo.rs`.
  - The code name `kb` becomes `memrepo`: the module `src/kb` becomes
    `src/memrepo`, the type is `MemRepo`, and the variable is `memrepo`. It is a
    code identifier, so the short form is allowed (rule 1).
  - The standard: `MKBase` becomes **Waymark Memory Standard**. The marker folder
    `.mkbase/` becomes `.waymark/`. The file `standard/mkbase-1.0.md` becomes
    `standard/waymark-1.0.md`. The conformance fixtures, the doc text, and every
    mention in specs, code, and tests follow. The settings file
    `.mkbase/waymark.toml` becomes `.waymark/settings.toml`, because the old name
    would repeat itself.

**Out of scope:**
- Tool names, argument names, and the result shapes of the MCP tools.

## Constraints

- The project has no users yet. No migration, no fallback for old names, and no
  repair step. Old names are replaced, not supported next to the new ones.
- Tool names, argument names, and results stay the same.
- A message that a test quotes changes in the same step as its test.
- Simple English (ASD-STE100 style), as in [AGENTS.md](../../../AGENTS.md).
- The repository stays public. Nothing in the new wording may name a company or
  an internal host.

## Acceptance Criteria

- **Given** the final text of the README, **When** a reader looks for the thing
  that waymark creates, **Then** it is called "memory repository", and
  "knowledge base" appears only as a named use case.
- **Given** the final text of the project, **When** a reader searches for
  "knowledge base", **Then** every hit is a named use case or a description of
  another tool, or a dated record.
- **Given** a new memory repository made by `init`, **When** a person looks at
  the clone folder and the `AGENTS.md`, **Then** the folder ends in
  `-agent-memory` and the marker line names `personal-agent-memory` or
  `team-agent-memory`.
- **Given** a waymark build, **When** an agent lists the tools and calls
  `status`, **Then** no description or message names the thing "knowledge base".
- **Given** the full test suite, **When** it runs after the sweep, **Then** it
  passes with no change of behavior.
- **Given** the writing conventions, **When** an author reads them, **Then** the
  three terms and the "repository alone" rule are there.

## Open Questions

_None._

## Decided

- The code name is `memrepo`. Other names considered: `memory` (too generic, and
  it suggests RAM), `memory_repository` (too long), `agent_memory` (names the
  idea, not the repository).
- No automated guard against the old phrase. The conventions in `AGENTS.md` and
  the final search are enough.
- The settings file is `.waymark/settings.toml`.
- The standard is renamed to **Waymark Memory Standard**, with the marker folder
  `.waymark/` and the file `waymark-1.0.md`. Other names considered: MAM
  (Markdown Agent Memory) and AMS (Agent Memory Standard).
- The vocabulary lives in `AGENTS.md`.
- There is no check script and no allowlist.
- A new memory repository has no default display name. The `name` input stays
  required. Docs and examples use `waymark-agent-memory` as the example
  repository name.
- No migration of existing memory repositories (there are none in use).
