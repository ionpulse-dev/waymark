# Decisions: Search and read

> **Note 2026-11-11:** "A small read budget" stays valid and is now enforced as result caps and a per-session read budget. All other decisions stay valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## A small read budget — 2026-11-06

**Decision:** Defaults: at most 5 documents and about 800 tokens. If nothing matches, return nothing. Configurable per knowledge base and per call.

**Reason:** Protects the agent context. Learned from a similar project.

## No folder means everything — 2026-11-06

**Decision:** Every command with folders treats an empty list as the root. Followed folders were dropped.

**Reason:** With this rule followed folders have no job. A personal default can come later if people ask.

## Detail levels — 2026-11-07

**Decision:** `--level 0|1|2` and `--in` let the agent scan cheaply and open a body only when worth it.

**Reason:** Reading is the main cost. Path, title, and description cost little.

## Statistics for the budget — 2026-11-07

**Decision:** `stats` counts words, tokens, characters, or documents with a breakdown.

**Reason:** Helps the agent check whether a folder fits into its read budget, shows growth and noise, and can feed the app later.

## Pull before read — 2026-11-06

**Decision:** A read pulls first if the clone is older than a few minutes.

**Reason:** Agents never read an old clone, even if the background sync is off.

## Caps and a read budget are limits — 2026-11-11

**Decision:** `search` is capped by `max_results` and `max_tokens`. `show` is capped by `max_documents_opened` per session. The advice "read at most 5 documents" is gone, because the server enforces it.

**Reason:** make the wrong action impossible instead of asking the agent not to do it. The numbers are defaults that a layer can tighten.

## A shared `store` module — 2026-11-14

**Decision:** `search-and-read` adds the module `src/store/`: it loads all documents from the working tree and gives filter, rank, and backlinks. `lint`, `updates`, `notifications-and-review`, and `manual-edits` reuse it. This is a module that the foundation specs do not list. The specs `quality-checks` and `updates` add `search-and-read` to `requires` because of it.

**Alternatives considered:**
- Each tool reads files by itself — rejected: the same scan, filter, and parse code would exist five times.
- Put the loader in `format` — rejected: `format` has no I/O by design.
- Put it in `kb` — rejected: `kb` does not read documents by design.

**Reason:** One loader keeps the tools small and the rules the same. Decided autonomously.

## Search is a scan of the working tree with fixed weights — 2026-11-14

**Decision:** `search` splits the query into terms, requires all terms, and ranks by weights: title 5, tags 3, path 2, description 2, body 1. Ties go to the newer `updated`, then to `id`. It scans local files at every call, and it sees uncommitted edits.

**Alternatives considered:**
- A local index (tantivy or SQLite) — rejected for now: more code and state, and a scan of some thousand files takes milliseconds.
- OR of terms — rejected: it returns too many documents for the small cap.

**Reason:** Answers the open question "is plain text match good enough". It is, until real numbers say otherwise (see `todo.md`). Decided autonomously.

## The default budget stays 20 opened documents — 2026-11-14

**Decision:** `max_documents_opened` stays 20 per session. Opening the same document again costs nothing. A layer can tighten it.

**Alternatives considered:** 10 (rejected: too tight for a task over several topics), 50 (rejected: it no longer guards the context).

**Reason:** Answers the open question. The number is a first guess and can change after use. Decided autonomously.

## Level 0 is only the ID — 2026-11-14

**Decision:** Level 0 returns only `id`. The facts `updated`, `disputed`, and `dispute_reason` come with level 1 and 2, and with `show`. `search` defaults to level 2 with a snippet, `list` to level 1.

**Alternatives considered:** Always return `disputed` (rejected: the criterion says level 0 shows only the ID).

**Reason:** Level 0 is the cheapest scan. An agent that wants facts uses level 1. Decided autonomously.

## `search` returns active documents by default — 2026-11-14

**Decision:** `search` has `status` with the default `active`. `list` uses `any`. `show` opens any status.

**Alternatives considered:** `any` for both (rejected: a deprecated document is no longer true and would fill the small cap).

**Reason:** A deprecated document is rarely what a question needs. An agent can ask for `any`. Decided autonomously.

## Generated index files are not search results — 2026-11-14

**Decision:** `search`, `list`, `stats`, and backlinks skip a file named `index.md` that has the tag `index`. `show` opens it.

**Alternatives considered:** Include them (rejected: an index lists every document and would match almost every query).

**Reason:** An index is a map, not knowledge. The rule is defined together with `index` in `quality-checks`. Decided autonomously.

## Moved documents are followed by `show` — 2026-11-14

**Decision:** `show` with an old ID that Git history says was renamed returns the document at the new ID, with `moved_from` and a notice. It does not return `not_found`.

**Alternatives considered:** `not_found` with a `next` sentence naming the new ID (rejected: one more call for the agent).

**Reason:** The requirements say `show` follows history. One call is cheaper. Decided autonomously.

## `show` reports `last_editor` from Git — 2026-11-14

**Decision:** `created_by` and `updated_by` come from the frontmatter. `last_editor` is the author of the last commit of the file. A difference shows a manual edit or a history rewrite, and `lint` warns about it (see `quality-checks`).

**Reason:** The requirements ask for the name and email from Git. The frontmatter is the cheap source, and Git stays the proof. Decided autonomously.

## `time` crate for dates — 2026-11-14

**Decision:** Add the `time` crate for dates and ages.

**Alternatives considered:** `chrono` (larger), own date code (error-prone).

**Reason:** Small and enough for RFC 3339 and day counts. Decided autonomously.

## Foundation additions — 2026-11-14

**Decision:** `gitsync::log` gets `last_commit_info(path)` and `renamed_to(path)`. `mcp::context::Context` gets the read budget. These are additions, not changes of rules.

**Reason:** Needed for `show`. They are reported to `git-sync` and `mcp-server`. Decided autonomously.

## The fetch age is a variable that tests can set — 2026-11-14

**Decision:** The 3 minute freshness window of `Repo::refresh_if_stale` can be set with the environment variable `WAYMARK_FETCH_MAX_AGE` (seconds). The default is 180. Tests set it to 0.

**Alternatives considered:** A fake clock in the code (rejected: tests run the real binary).

**Reason:** Tests must not wait 3 minutes. The variable is a setting of the person and cannot loosen a limit. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously.**

- **`chrono` instead of `time`.** `chrono` is already a dependency (it serves `write-documents`). A second date crate would add size for no gain. This replaces the decision "`time` crate for dates".
- **`Repo::log(1, ...)` instead of `last_commit_info`.** It already returns the author of the last commit of a path. Only `renamed_to` was added to `gitsync::log`. It reads the rename lines of the whole history, newest first, and follows the chain up to 10 steps.
- **The read budget is a set.** `Context.opened` is a `HashSet` of IDs, so that the same document costs once.
- **Group keys of `stats` with `by: "folder"`.** The key is the path of the folder below the base folder (`backend/payments`). A document that lies directly in a base folder has the base folder as its key, and `.` for the root.
- **Scope.** `store::Scope` applies the `folders` limit and the `folders` argument. `show` uses the limit only, and hides a document that the limit hides, also in `backlinks`.
- **A file that cannot be parsed** is skipped by the read tools. `show` fails with `validation_failed` and names the first finding.
- **A repository without a remote** gives the notice "This repository has no remote, so nothing is shared." on every read (from `Repo::refresh_if_stale`).
- **Test hook.** `WAYMARK_FETCH_MAX_AGE` (seconds) sets the freshness window. The tests use `0`.

## Index files are normal documents — 2026-11-14

**Decision:** `search`, `list`, `stats`, and backlinks no longer skip a file named `index.md` with the tag `index`. It is a normal document. This replaces "Generated index files are not search results" above. See [remove-index](../../foundation/remove-index/decisions.md).
