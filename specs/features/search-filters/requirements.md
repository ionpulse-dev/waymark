---
type: feature
status: implemented
created: 2026-11-15
updated: 2026-11-15
requires:
  - search-and-read
  - tool-conventions
---

# Search filters and paging

## User Story

> As a developer, I want to read all matches of a search and to narrow `search` by author, by date, and by dispute, so that I find the right document without loading many.

## Personas

- **Developer** — reads the memory with a coding agent and wants few, exact results.
- **Memory owner** — looks for what one person wrote or what changed in a period.

## Behaviour

- `search` takes `offset`. When more matches exist than one call returns, the result says `truncated: true` and gives `next_offset`.
- `search` takes three more filters: `author`, `updated_after`, and `updated_before`.
- `search` takes `disputed`.
- All filters combine with `folders`, `status`, and `tag`. A document must match all of them.

## Interfaces & Contracts

### `search`

| Parameter | Meaning |
|---|---|
| `offset` | How many matches to skip. 0 or more. |
| `disputed` | `true` returns only disputed documents. |
| `author` | Text. A document matches when `created_by` or `updated_by` contains it, ignoring case. A name or an email part. |
| `updated_after` | `YYYY-MM-DD`. A document matches when `updated` is on this day or later. |
| `updated_before` | `YYYY-MM-DD`. A document matches when `updated` is on this day or earlier. |

**Result of `search`:** `results`, `total_matches`, `truncated`, and `next_offset` (only when `truncated` is true).

**Errors:** `invalid_arguments` for a negative `offset`, a date that is not `YYYY-MM-DD`, or `updated_after` later than `updated_before`.

## Acceptance Criteria

- **Given** more matches than one call returns, **When** `search` is called with `offset: 0` and then with the `next_offset` of the result, **Then** the second call returns the next matches and no match is repeated.
- **Given** documents by two authors, **When** `search` is called with `author`, **Then** only the documents of that author are returned, for a name part and for an email part, in any case.
- **Given** documents updated on different days, **When** `updated_after` and `updated_before` are set, **Then** only documents in the range (both days included) are returned.
- **Given** a disputed and a normal document, **When** `search` is called with `disputed: true`, **Then** only the disputed document is returned.
- **Given** a wrong date, a negative `offset`, or a reversed range, **When** the call runs, **Then** it fails with `invalid_arguments` and a `next` hint.
- **Given** the `folders` limit, **When** filters are used, **Then** documents outside the limit never appear.

## Constraints & Edge Cases

- The result cap (`max_results`, `max_tokens`) still applies to each page.
- `offset` past the last match returns an empty list with `truncated: false`.
- A filter never reveals documents that a limit hides.

## Open Questions

None.
