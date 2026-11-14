# Plan: Search and read

## Overview

A new `store` module loads the documents of the knowledge base from the working tree and answers questions about them: filter, rank, backlinks. Four thin tools (`search`, `show`, `list`, `stats`) call it. The read budget lives in the `Context` of `mcp-server`. `search-and-read` also adds the document-set loader that `quality-checks` and `updates` reuse (see `decisions.md`).

## Components Affected

- `format` — `parse`, `count` (words, tokens, chars), `links` (finds links for backlinks). No change.
- `config` — `EffectiveSettings` gives `max_results`, `max_tokens`, `max_documents_opened`, and `folders`. `config::glob` matches folder globs.
- `kb` — the root and the `exclude` list.
- `gitsync` — `Repo::refresh_if_stale` before every read. `gitsync::log` gives the last commit of a path. This spec adds `last_commit_info` and `renamed_to` to `src/gitsync/log.rs`.
- `mcp` — `Context` gets the read budget (`opened: HashSet<String>`). The registry gets four tools.

## New Files

- `src/store/mod.rs` — `Store`: loads every `.md` file below the root (skips dot paths and `.waymark/exclude` entries), parses it with `format::parse`, and keeps `Doc { id, path, front, body, raw_bytes, parse_findings }`. Methods: `docs()` (without index files), `all_docs()`, `get(id)`.
- `src/store/filter.rs` — `Scope`: the `folders` limit plus the `folders` argument. `Scope::allows(id)`.
- `src/store/rank.rs` — the matcher and the score.
- `src/store/backlinks.rs` — backlinks of one ID over all documents (body links and property wikilinks, index files ignored).
- `src/store/stats.rs` — the metrics and the grouping.
- `src/tools/search.rs` — the `search` tool.
- `src/tools/show.rs` — the `show` tool.
- `src/tools/list.rs` — the `list` tool.
- `src/tools/stats.rs` — the `stats` tool.
- `src/tools/read_common.rs` — level rendering, the cap by results and tokens, the read refresh and its notice.
- `tests/it_search_and_read.rs` — end-to-end tests through the MCP client.

## Technical Approach

1. **Refresh.** Every tool calls `Repo::refresh_if_stale()` first. A failed fetch adds the notice "The last update from the remote failed. The clone may be behind." and the tool goes on.
2. **Load.** `Store::load(root, exclude)` walks the tree, parses each file, and keeps the parse findings. It reads the working tree, so manual edits show. It is built per call. A call takes milliseconds for a few thousand files. A cache is deferred.
3. **Index files.** `Doc::is_index()` is true when the file name is `index.md` and `tags` holds `index`. `docs()` skips them.
4. **Scope.** `Scope` holds the limit globs and the argument folders. A document is in scope if it matches a limit glob (when any is set) and lies under an argument folder (when any is given). A folder `a/b` means the glob `a/b/**`.
5. **Search.** Split the query on white space and lowercase it. A document matches if every term occurs as a substring in at least one searched part. The score of a term is the sum over the parts where it occurs: title 5, tags 3, path 2, description 2, body 1. Sort by score, then `updated` descending, then `id`. The snippet is about 25 words around the first body match, with `...` at a cut end. It is made only at level 2.
6. **Caps.** Walk the sorted list. Stop when the count reaches `min(limit, max_results)`, or when one more entry would pass `max_tokens`. The tokens of an entry are `format::count::tokens` over the strings of its fields. The first entry is always returned. Set `truncated` and `total_matches`.
7. **Levels.** `read_common::render(doc, level)` makes the entry as in the requirements.
8. **Show.** Check the `Scope`. Check the budget: if the ID is not in `Context.opened` and the set is at `max_documents_opened`, return `access_denied`. Else add it. Read `version` with `gitsync::log::last_commit(path)`. `last_editor` comes from `last_commit_info`. `pending_edit` is true when `git status` lists the path. `replaced_by` lists the documents whose `supersedes` names the ID. `age_days` uses today's date.
9. **Moved documents.** If the ID is not found, call `gitsync::log::renamed_to(path)`. It runs `git log --diff-filter=R -M --name-status --format= -- <old path>` and follows the chain to the last name. If that file exists, `show` returns it, sets `moved_from`, and adds the notice "This document moved from `<old>` to `<new>`."
10. **List.** Filter, sort by `id`, skip `offset`, apply the caps, and set `next_offset`.
11. **Stats.** Sum the metric per document. `by: folder` groups by the first path segment below the given folder (documents directly in the folder get the key `.`). `by: author` groups by `created_by`. Sort and cut at 50.
12. **Time.** Add the `time` crate for dates (`today`, `age_days`). The same crate serves `updates`.
13. **Descriptions.** The tool descriptions are one or two plain sentences each and say what the tool returns.

## Cross-Feature Dependencies

- `mcp-server` — `Context`, `Tool`, `ToolOutput`, `ToolError`. Resolved: used as defined.
- `write-documents` — uses `show` `version` as `base`. No code shared except `gitsync::log::last_commit`.
- `quality-checks`, `updates`, `notifications-and-review`, and `manual-edits` — use `Store`. They need `search-and-read` to be implemented first (see `decisions.md`).

## Risks

- A cold scan of a large repository (more than 10 000 files) may take over a second. Mitigation: the plan keeps `Store` behind one function, so a cache can replace it. A test with 2 000 generated documents fails above 500 ms.
- Plain substring match finds "key" inside "monkey". Accepted for the first release. Ranking puts whole-term title matches first only by weight, not by word boundary.
- A path that Git reports as renamed twice. The chain follows up to 10 steps.

## Open Questions

None.
