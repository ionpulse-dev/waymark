# Decisions: Document format

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## One topic per file — 2026-11-06

**Decision:** One topic per file, with YAML frontmatter. Planned fields (type, status, dates, sources) were extended later with author fields and links. The `owners` field was dropped, because the folder is the owner.

**Alternatives considered:**
- Several facts per file — more merge conflicts and harder staleness tracking.

**Reason:** Fewer merge conflicts, cheap reads, and staleness tracked per topic.

## Everything is Obsidian compatible — 2026-11-06

**Decision:** Every knowledge base is a valid Obsidian vault and everything waymark writes works in Obsidian without changes. Consequences: only flat properties, `anchors` is a list of strings, `related` and `supersedes` hold full-path wikilinks, `disputed` is a checkbox plus a reason text.

**Alternatives considered:**
- Nested YAML for anchors and disputes — Obsidian cannot edit nested properties.
- A custom format — no free browser and editor for people.

**Reason:** People get a polished browser, editor, and graph for free, and the content is never locked into waymark.

## The ID is the path — 2026-11-06

**Decision:** The document ID is its path without `.md`. File names never change. Typed links are only `related` and `supersedes`. Folders and tags give implicit links. Backlinks are computed.

**Alternatives considered:**
- A separate ID system or stored backlinks — extra state and write conflicts.

**Reason:** Stable links, no extra ID system, and no central file that every writer edits.

## A required description and a size cap instead of a word limit — 2026-11-07

**Decision:** Every document has a required `description`. The first design had a 150-word limit for the body. It was replaced by a hard size cap (100 KB) plus limits for title and description. There is no notice at a word count. Binary data is not stored; a document describes it and links to it.

**Alternatives considered:**
- A 150-word body limit — an architecture decision record has 500 to 2,000 words and must not be split.
- No cap at all — no guard against noise.

**Reason:** The agent sees path, file name, title, and description before the body, so reading stays cheap. The skill tells agents to keep documents small. Noise risk is accepted and measured during dogfooding.

## Author fields in the frontmatter — 2026-11-06

**Decision:** `created_by` and `updated_by` hold `"Name <email>"`. Only the CLI writes them. `lint` compares them with Git history and warns on a difference. There is no field for the agent that wrote the document.

**Alternatives considered:**
- Authors only in Git — invisible in Obsidian.
- An `agent` property or Git trailer — a self-reported agent name can be wrong or forged, and the CLI cannot detect the calling tool.

**Reason:** The author must be visible in Obsidian. The person is responsible, so only the person is recorded.

## The unit is called a document — 2026-11-08

**Decision:** The unit of a knowledge base is a "document", not a "note".

**Reason:** A document can be long, and "note" suggests a short text.

## Anchors are typed strings — 2026-11-06

**Decision:** `anchors` is a list of strings `"<type>:<target>"` with the types `git` and `url`. The check that flags documents whose anchored code changed comes after the first release. Web pages change often, so a later `url` check may give false alarms.

**Reason:** Anchors are free to write now, so early documents already carry them.

## The format follows MKBDS — 2026-11-08

**Decision:** The document format is defined in the Markdown Knowledge Base Document Standard (MKBDS) 1.0, in the `waymark-trail` repository. This item implements it and links to it. A format change is made in the standard first.

**Alternatives considered:**
- Keep the rules in both projects and sync them by hand — they drift.

**Reason:** One source of truth, shared with the standalone skills.
