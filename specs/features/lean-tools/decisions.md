# Decisions: Lean tools

## What the review dropped — 2026-11-15

**Decision (product owner, decided in review):** The review of all tools against `tool-conventions` and the specs found ideas that earlier decisions had settled on purpose. They stay as they are:

- No tool for settings. `mcp-server` has the rule "no control tools", so an agent can never loosen its limits.
- No move of tools to a CLI. MCP is the only interface, and `personal-server` needs `init` over MCP.
- No merge of `deprecate_document`, `restore_document`, and `dispute_document` into one tool. The product owner chose a named `restore_document` so that an agent finds it (see `restore-and-replace`).
- No merge of `delete_document` and `delete_folder`. A team must be able to switch off the bulk delete alone (see `folders`).
- No merge of `folder_tree` or `list_tags` into `stats` (see `folders` and `manage-tags`).
- No `base` or count guard for `rename_tag` and `move` (see `manage-tags` and `move-documents`: they lose nothing, and `undo` reverts them).
- `activity` and `history` keep the cursor `before`, the lists keep `offset` (see `tool-conventions`).

**Reason:** Do not reopen a decision without a new fact.

## `list` becomes a mode of `search` — 2026-11-15

**Decision (product owner, decided in review):** One tool searches and lists. The query is optional.

**Alternatives considered:** Keep two tools (the filters and the paging are the same text twice, about 1.3 KB of context in every session). Make `list` the main tool and `query` a filter (it would hide that `search` is the tool for finding things).

**Reason:** `list` is `search` with no words. Both are read tools, so no limit or hint changes. The cost is one optional parameter.

## One default for `status`: `active` — 2026-11-15

**Decision (decided autonomously):** `search` uses `active` with and without a query. This replaces "`search` returns active documents by default" in `search-and-read`, where `list` used `any`.

**Alternatives considered:** `any` without a query (two defaults in one tool, and a deprecated document would fill the small cap).

**Reason:** One default is easier to learn. The alias `list` keeps `any` for old calls, and `status: any` is one parameter.

## The level depends on the query — 2026-11-15

**Decision (decided autonomously):** The default level is 2 with a query, 1 without. A snippet needs a query. Level 2 without a query would add the description of every document, which is what a plain list should not cost.

**Reason:** Both defaults are the ones that `search` and `list` had.

## `list` is an alias for one version, and for good in limits — 2026-11-15

**Decision (decided autonomously):** A call with `list` runs `search` until 0.6.0, with a notice. In limits, `list` counts as `search` with no end date.

**Alternatives considered:** No alias (agents with a saved `list` call would fail at once). Block `search` when `list` is denied (it is what the alias rule means: a denied tool stays denied, and the new tool replaces the old one).

**Reason:** The same rule as for the renames in 0.4.0 (see `tool-conventions`).

## The aliases of 0.4.0 end in 0.5.0 — 2026-11-15

**Decision (decided autonomously):** `tool-conventions` and the notice of every old-name call say that the old names stop working in 0.5.0. This is 0.5.0, so the call aliases of the nine renames are removed. The aliases in limits stay.

**Alternatives considered:** Keep them one more version (the notice and the changelog of 0.4.0 would be wrong).

**Reason:** Keep the promise that was made in public text.

## `update_document` gets `replace`, `append`, `add_tags`, and `remove_tags` — 2026-11-15

**Decision (product owner, decided in review):** Small changes get small parameters on the existing tool. No new tool.

**Alternatives considered:** A new tool `patch_document` (about 1 KB of context for the tool). A diff format (a new contract that an agent must write exactly). A list of operations (a large schema for a rare case). A merge of the list parameters in `tags` (the agent cannot remove a tag).

**Reason:** `update_document` already says "a small change should cost a small call", but it sends the whole body today. `replace` needs the passage to occur once, so a wrong edit fails and does not hit the wrong place. One of `body`, `replace`, and `append` per call keeps the meaning clear.

## `show` returns the deprecation reason — 2026-11-15

**Decision (decided autonomously):** `deprecation_reason` comes from the `Reason:` line of the newest `waymark: deprecate` commit of the document. It is read from Git only for a deprecated document.

**Alternatives considered:** A frontmatter field (the standard has none, and the format is fixed, see `write-documents`). Return it for every version (an old version may not be the one that was deprecated).

**Reason:** `write-documents` put the reason into the commit message and left "show the reason" for `search-and-read`. This closes that item at the cost of one Git call for deprecated documents only.

## `updates` pages with `offset`, and only with `since` — 2026-11-15

**Decision (decided autonomously):** `offset` needs `since`. The description names `until` as the value for the next `since`.

**Alternatives considered:** A cursor in the stored state (it breaks when the server keeps no state, see `personal-server`). `offset` without `since` (the first call moves the stored time, so the second call would see an empty window).

**Reason:** The client owns the window, so the call works in every mode.

## The stale-name test reads the source — 2026-11-15

**Decision (decided autonomously):** A test searches the Rust source for the retired tool names in the forms that a hint uses ("call delete", "use deprecate", "dispute and clear"), and for `list` in hints. It also lists every tool name that a hint contains in backticks or words with `_document`, and checks that each one is a tool of the build.

**Alternatives considered:** A runtime test of every error (many errors need special states to trigger).

**Reason:** The bug was a word in a string. A source scan finds the whole class and needs no state.

## Shorter parameter descriptions — 2026-11-15

**Decision (decided autonomously):** The descriptions of `preview`, `id`, and `base` become shorter, and the same text is used in every tool. (`folders` stays as it is: it is already short.) The rule of `tool-conventions` stays: every parameter has a description, with its default or allowed values.

**Reason:** About 13 tools repeat these words. The meaning does not change.
