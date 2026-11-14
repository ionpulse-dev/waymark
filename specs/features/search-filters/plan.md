# Plan: Search filters and paging

## Overview

A shared `Filter` in `read_common.rs` reads the new arguments and tests a document. `search` and `list` use it. `search` gets the paging code of `list`.

## Components Affected

- `src/tools/read_common.rs`: `Filter` (`from_args`, `matches`), `offset_arg`, and `page_data`.
- `src/tools/search.rs`: `offset`, `disputed`, and the filters. The result gets `next_offset`.
- `src/tools/list.rs`: the filters. It uses `offset_arg` and `page_data`.

## New Files

- `tests/it_search_filters.rs`: the cases of `test.md`.

## Technical Approach

1. `Filter::from_args(args)` reads `author`, `updated_after`, `updated_before`, `disputed`. It checks the dates with `format::fields::is_valid_date` and the order of the range.
2. `Filter::matches(doc)` compares the author text with `created_by` and `updated_by` in lowercase, and the dates with the `updated` field. Dates compare as text (`YYYY-MM-DD`).
3. `search` ranks all hits, skips `offset`, takes `limit + 1`, and applies the cap, as `list` does. `next_offset = offset + shown` when more remain.
4. Descriptions of the new parameters follow `tool-conventions`.

## Cross-Feature Dependencies

- `tool-conventions`: parameter descriptions and the paging rule.

## Risks

- The ranking must be stable between two calls, or pages repeat entries. `rank::sort` sorts by score and then by ID, so it is stable.

## Open Questions

None.
