# Plan: Lean tools

## Overview

Seven small changes in existing tools. `list` moves into `search`. The old-name table of `tool-conventions` gets a second use: one entry for `list`, and the end of the call aliases of 0.4.0. `update_document`, `show`, and `updates` get parameters or fields. Three hints and the shared parameter texts change.

## Components Affected

- `src/tools/search.rs`: the query is optional. Without it, the tool lists by `id`.
- `src/tools/list.rs`: removed. `src/tools/mod.rs` no longer registers it.
- `src/config/limits.rs`: `RENAMED_TOOLS` gets `("list", "search")`. It has 10 entries.
- `src/mcp/tool.rs`: `Registry::call` takes a name only if it is a current tool. The old names of 0.4.0 are unknown tools, and so is `list` since 0.6.0. (In 0.5.x, the call `list` got the defaults `status: any` and `level: 1` and a notice.)
- `src/tools/update_document.rs`: `replace`, `append`, `add_tags`, `remove_tags`.
- `src/tools/show.rs` and `src/gitsync/log.rs`: `Repo::deprecation_reason(rel)`.
- `src/tools/updates.rs`: `offset` and `next_offset`.
- `src/tools/delete.rs`, `src/tools/remove_folder.rs`, `src/tools/verify.rs`: the hints.
- All tool files: the shared parameter texts (`preview`, `id`, `base`).
- `README.md`, `docs/install.md`, `CHANGELOG.md`: the tool list and the allow rules.

## New Files

- `tests/it_lean_tools.rs`: the cases of `test.md`.

## Technical Approach

1. **Search without a query.** `terms.is_empty()` no longer fails. The hits are all documents that pass the filters, with no score, sorted by `id`. The `entry` function builds each result. The snippet is made only with terms. The message is "N of M documents." without a query.
2. **Defaults.** `status` defaults to `active`. `level` defaults to 2 with terms and 1 without.
3. **The alias `list`.** `Registry::call` maps `list` to `search` and fills `status` and `level` if the call has none. It adds the notice. Any other old name of the nine is not mapped, so the registry returns `UnknownTool`. `canonical_tool` keeps all ten for limits.
4. **`replace` and `append`.** They build the new body from the old body. They set `secret_fields` and the `body` entry of `changed`, so that the checks of `write-documents` run unchanged. `replace` counts the occurrences with `matches` and fails if the count is not 1.
5. **Tags.** `add_tags` and `remove_tags` start from the current tags, check each new tag with the check of `manage-tags`, and set the list. `changed` lists `tags` only when the list differs.
6. **Deprecation reason.** `Repo::deprecation_reason` runs `git log --follow --no-merges --format=<marker>%s<sep>%b -- <path>`, and takes the first record whose subject starts with `waymark: deprecate` and whose body has a `Reason:` line. `show` calls it only for a deprecated, current document.
7. **`updates`.** The sorted list is cut after `offset` and at 50 entries. `next_offset` is `offset + shown` when more remain.
8. **Stale-name test.** A test walks `src/**/*.rs`, takes the string literals that follow `with_next(`, `Error::` constructors, and `"message"`, and checks the retired names. It also checks that each tool name in a hint is a tool of the registry.
9. **Parameter texts.** The three texts that most tools repeat (`preview`, `id`, `base`) are shortened in every tool file. The test `every_parameter_has_a_description` stays. The tool list went from 20223 to about 19460 bytes.

## Cross-Feature Dependencies

- `search-and-read`, `search-filters`: the contract of `search` and `list`.
- `tool-conventions`: aliases, the budget, and the description rules.
- `write-documents`, `verification`, `manage-tags`: the checks that `update_document` reuses.
- `updates`, `document-history`, `restore-and-replace`: the contracts that change.

## Risks

- Many tests call `list`. They move to `search` in this item. The alias has its own test.
- A passage for `replace` that an agent copies from a `show` result has the line ends of the file (LF). The call is changed to LF too, so Windows-style input still matches.
- The alias list for limits grows by one. A test covers every old name.

## Open Questions

None.
