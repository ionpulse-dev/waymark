# Decisions: Document format

> **Note 2026-11-11:** Mentions of the CLI now mean the MCP server. No decision in this file changes.

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

**Decision:** The document ID is its path without `.md`. (File names never change was replaced, see below.) Typed links are only `related` and `supersedes`. Folders and tags give implicit links. Backlinks are computed.

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

## Authors always have name and email — 2026-11-14

**Decision:** `created_by` and `updated_by` always hold `"Name <email>"`. A knowledge base cannot choose "name only". This keeps the rule of the MKBase Document Standard and the decision "Author fields in the frontmatter".

**Alternatives considered:**
- A setting for "name only" — the email is public in a public repository, but it is already in the Git history. A setting would give two formats, and `lint` could no longer compare the field with Git history in one simple way.

**Reason:** The person is responsible for each write, and the email identifies the person. Git history holds the same data, so the field adds no new exposure. One format keeps `lint` and the conformance fixtures simple.

## The standard stays at 1.0 until the first release — 2026-11-14

**Decision:** The MKBase Document Standard is version 1.0 and stays 1.0 until waymark is released. Before the release, the standard is changed in place: no new version number, no changelog, and no frozen copies. The version file name stays `standard/mkbase-1.0.md`. The first public release freezes 1.0. From then on, minor versions (1.1) add things that valid 1.x documents still pass. Major versions (2.0) break them and need a migration note. Each version is a frozen file with a changelog entry, and tools refuse a major version they do not know. The rules go into a "Versioning" section of the standard at the first release.

**Alternatives considered:**
- Version every change from now on — nobody uses the standard yet, so the numbers and the changelog would only record drafts.

**Reason:** The project is not released, so no tool and no knowledge base depends on a version number. Versioning costs effort and gives no value before there are users.

## Patch versions for minor changes — 2026-11-14

**Supersedes:** The version rules of "The standard stays at 1.0 until the first release" (the pre-release rule stays)

**Decision:** The standard uses three numbers from the first release: major.minor.patch.
- **Patch (1.0.1):** a clarification, a typo, or a new example. It changes no rule and no fixture result.
- **Minor (1.1.0):** an addition that valid 1.x documents still pass, such as a new optional field.
- **Major (2.0.0):** a change that makes a valid document invalid, or changes what a field means. It needs a migration note.

A claim of conformance and the `version` file in `.mkbase` name only major.minor ("MKBase 1.0"), because a patch never changes what is valid. The patch number shows in the header of the standard and in its changelog. The file name stays `mkbase-<major>.<minor>.md`, and a patch edits that file in place. Until the first release, the standard stays "1.0" as decided above.

**Alternatives considered:**
- Two numbers only — a typo fix would need a new minor version, or would have no version at all.

**Reason:** Small fixes are frequent and should be visible in the changelog without implying a change to the rules.

## Obsidian baseline for the release check — 2026-11-14

**Decision:** The release check of the sample vault uses the latest stable Obsidian desktop release on the day of the check. The version is written in the release notes. The floor is Obsidian 1.4, the first version with the current property editor. The floor must be verified before this spec is approved.

**Alternatives considered:**
- Only a fixed version — it ages and hides new problems.
- Only "latest" — there is no floor for people with older installs.

**Reason:** The check should find problems in the version that people use now, and the floor says which older versions are supported. The floor can be raised without a spec change.

## The unit is called a document — 2026-11-08

**Decision:** The unit of a knowledge base is a "document", not a "note".

**Reason:** A document can be long, and "note" suggests a short text.

## Anchors are typed strings — 2026-11-06

**Decision:** `anchors` is a list of strings `"<type>:<target>"` with the types `git` and `url`. The check that flags documents whose anchored code changed comes after the first release. Web pages change often, so a later `url` check may give false alarms.

**Reason:** Anchors are free to write now, so early documents already carry them.

## The format follows MKBase — 2026-11-08

**Decision:** The document format is defined in the MKBase Document Standard 1.0, in the `waymark-trail` repository. This item implements it and links to it. A format change is made in the standard first.

**Superseded by:** The standard lives in this repository (the location changed, the rule "a format change is made in the standard first" stays).

**Alternatives considered:**
- Keep the rules in both projects and sync them by hand — they drift.

**Reason:** One source of truth, shared with the standalone skills.

## Six document types, no `team` type — 2026-11-08

**Superseded by:** No `type` field

**Decision:** The default list of `type` has six values: `fact`, `decision`, `contract`, `howto`, `glossary`, and `gotcha`. The value `team` was removed, because the `_team.md` convention was rejected (see `knowledge-base-repo`). This follows the MKBase Document Standard.

**Alternatives considered:**
- Keep `team` as an unused value — it invites a convention that nobody wants.

**Reason:** The format should hold only what is used.

## No `type` field — 2026-11-08

**Supersedes:** Six document types, no `team` type

**Decision:** Documents have no `type` field and there is no list of document types. The list was never reviewed, and agents had no rule for choosing a type. This follows the MKBase Document Standard. The commands lose the `--type` option, `init` loses `--types`, and the settings file loses `types`.

**Alternatives considered:**
- Make `type` optional — it keeps an unreviewed list.
- Keep it required and define the values — it makes the unreviewed list official.

**Reason:** The format should hold only what the project decided. A field can be added later.

## A rename is a move — 2026-11-09

**Decision:** A file name does not change only because the title changed, but a person may rename a document. A rename is a move: it changes the ID, uses `git mv`, and fixes every link.

**Reason:** A rename is nothing else than a move. "Never rename" forced people to keep names that no longer fit. The history stays with `git mv`.

**Supersedes:** The part "File names never change" of the ID decision

## A verbatim copy of the standard lives in `standard/` — 2026-11-11

**Decision:** The folder `standard/` holds a verbatim copy of the MKBase Document Standard and its conformance fixtures from `waymark-trail` (commit `e0761d6`). It is never edited in this repository. When the standard changes in `waymark-trail`, the whole folder is replaced and the commit is noted here.

**Alternatives considered:**
- Link to the other repository only. The specs and the conformance tests then depend on a remote file.
- A Git submodule. It adds a Git concept that contributors must learn.

**Reason:** The specs and the tests can link to a local file. One source of truth stays in `waymark-trail`, so there is no fork of the standard.

**Superseded by:** The standard lives in this repository.

## The standard lives in this repository — 2026-11-11

**Decision:** The MKBase Document Standard is part of waymark. It lives in `standard/` in this repository, with its conformance fixtures, and it is maintained here. The copy is no longer verbatim and read-only: a change to the standard is made here, in the standard first, and then in this spec. The same text exists in the `waymark-trail` repository until that repository is retired when this one goes public. That copy is not updated after the retirement is decided.

**Supersedes:** "The format follows MKBase" (the location only) and "A verbatim copy of the standard lives in `standard/`".

**Alternatives considered:**
- Keep the standard in `waymark-trail` and copy it here. The copy would drift or block changes, and the other repository is to be taken down.
- A separate repository only for the standard. One more repository to keep in step, for a file that this project owns.

**Reason:** The standard and the product that implements it change together, under one spec workflow. One repository is one source of truth.

**Source of the first text:** `waymark-trail` at commit `7828ae4` (the standard folder), copied on 2026-11-11.

## Counting rules for size, words, and tokens — 2026-11-14

**Decision:** The standard (section 9.1) fixes how to count. Size is UTF-8 bytes of the whole file. A word is a run of non-white-space characters. Title and description are counted in their value. Body counts remove the markup of callouts, embeds, highlights, and comments. Estimated tokens are words times 1.5, rounded up, and no rule depends on them.

**Alternatives considered:** Counting characters instead of words (rejected: the limits are already in words). Counting real tokens with a model tokenizer (rejected: it ties the standard to one vendor and needs a library).

**Reason:** The server and `lint` must give the same result. The factor 1.5 is a rough planning value and can change without a spec change, because nothing depends on it.

## No shell checker in the standard — 2026-11-14

**Decision:** Annex B of the standard no longer names `tests/check-document.sh`. It says a tool follows the standard when it accepts every document of `vault/` and rejects every document of `invalid/` with the rule named by the file name. The Rust test `tests/format_conformance.rs` runs this set.

**Alternatives considered:** Write the shell checker here (rejected: a second implementation of the same rules that can drift, and a shell cannot check every rule).

**Reason:** The standard stays neutral about tools, and there is one implementation to maintain. The script did not exist in this repository.

## An unclosed comment runs to the end — 2026-11-14

**Decision:** In word counts, a `%%` with no closing `%%` starts a comment that runs to the end of the document. Section 9.1 also says which markup is removed with its text (comments) and which only loses its markers (callouts, highlights, embeds).

**Alternatives considered:** Treat an unclosed `%%` as plain text (rejected: it differs from what Obsidian shows).

**Reason:** The count matches what a person sees in Obsidian. Body counts cannot fail a document, so a stray `%%` has a small effect.

## Broken links: error in properties, warning in the body — 2026-11-14

**Decision:** A link to a missing document is an error when it is in a property (`related`, `supersedes`), and a write through waymark fails. In the body it is a warning, and the write goes through. The standard is not changed: it says only that a link SHOULD point to an existing document. The severity is a waymark rule. `quality-checks` must use the same severities.

**Alternatives considered:** Error for every broken link (rejected: two new documents that link to each other cannot be written one by one). Warning for every broken link (rejected: a missing `supersedes` target leaves the old document active, and `related` drives reading).

**Reason:** Property links drive reading, the graph, and deprecation, so a broken one does real harm. A body link is only text.

## The Obsidian floor is verified at the release — 2026-11-14

**Supersedes:** The sentence "The floor must be verified before this spec is approved" in "Obsidian baseline for the release check". The rest of that decision stays.

**Decision:** The check that Obsidian 1.4 is the right floor is part of the first release check, not a gate for approving this spec. The check opens `standard/conformance/vault/` in Obsidian 1.4 and looks at properties, list properties, the graph, and callouts. It also checks in the Obsidian changelog that 1.4 is the first version with the current property editor. Until then the floor is a claim, not a fact.

**Alternatives considered:** Verify before approval (rejected: needs a person with Obsidian 1.4 installed, and no code depends on the floor). Drop the floor (rejected: a floor tells people which older versions are supported).

**Reason:** Nothing in the code depends on the floor, and the real check needs Obsidian on a real machine.

## One crate, and yaml-rust2 for frontmatter — 2026-11-14

**Decision:** The format code is a module `format` in the one `waymark` crate. It moves into its own crate only when a second consumer exists. Frontmatter is parsed with `yaml-rust2` into its untyped value tree. The module checks types itself and writes frontmatter by hand, because the format is flat. Frontmatter does not use `serde`.

**Alternatives considered:**
- `serde_yaml` (rejected: archived since 2024).
- `serde_yml` (rejected: RUSTSEC-2025-0068, unsound and unmaintained).
- `serde_norway`, `serde_yaml_ng` (rejected: forks of `serde_yaml` on `unsafe-libyaml`, and reportedly no release since 2024).
- `serde-saphyr` (rejected for now: it has no untyped value, and the checks must see nested maps and wrong types before any struct, e.g. a number as a title).
- A separate `mkbase` crate in a workspace (rejected: more setup, and the standard is already open to other tools).

**Reason:** The checks need the raw types, not a struct. `yaml-rust2` is pure Rust and follows YAML 1.2, so `yes` and `no` are not booleans, which removes one trap. It gets only basic maintenance, so the crate is used in one file (`src/format/frontmatter.rs`) and can be swapped for `saphyr` without a spec change. Source: web search on 2026-11-14, mostly secondary guides. Not checked on crates.io. Release dates are checked before the first implementation task (see `todo.md`).
