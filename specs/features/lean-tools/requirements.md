---
type: feature
status: implemented
created: 2026-11-15
updated: 2026-11-17
requires:
  - search-and-read
  - search-filters
  - write-documents
  - updates
  - document-history
  - restore-and-replace
  - tool-conventions
---

# Lean tools

## User Story

> As a person with an agent, I want the tools to cost less context and to make small changes with small calls, so that the agent keeps room for my work and never follows a hint to a tool that does not exist.

## Personas

- **Developer** — the agent edits long documents and reads what changed.
- **Every person** — pays for the tool list in the context of every session.

## Behaviour

Seven changes. Each one comes from a review of all tools against `tool-conventions`. The review dropped ideas that earlier decisions had settled on purpose (see `decisions.md`).

1. **`list` becomes a mode of `search`.** `search` takes an optional `query`. With a query it works as before. Without a query it lists the documents that pass the filters, sorted by `id`. The tool `list` is removed from the tool list.
2. **The call aliases of 0.4.0 end.** `tool-conventions` promised that the old names of the nine renamed tools stop working in 0.5.0. They do. Old names in limits keep working for good.
3. **`update_document` makes small changes with small calls.** `replace` changes one passage of the body. `append` adds text at the end. `add_tags` and `remove_tags` change single tags.
4. **`show` tells why a document is deprecated.** The reason is in the commit message. `show` returns it as `deprecation_reason`.
5. **`updates` can read a long window.** It takes `offset`, and the result says how to continue.
6. **No hint names a tool that does not exist.** Three error hints used old names. A test keeps it so.
7. **Shorter parameter descriptions.** The same words are repeated in many tools. They are cut to what an agent needs.

## Interfaces & Contracts

### `search` without a query (replaces `list`)

| Parameter | Change |
|---|---|
| `query` | Optional. A document must contain all words. Without it, `search` lists documents. |
| `in` | Used only with a query. |
| `level` | Default 2 with a query, and 1 without a query (no snippet is made without a query). |
| `status` | Default `active` for both modes (before: `list` used `any`). |

All other filters (`folders`, `tag`, `disputed`, `author`, `updated_after`, `updated_before`, `limit`, `offset`) work in both modes. Without a query the order is by `id`. The caps of `search` (`max_results`, `max_tokens`) apply. The result has the fields of `search`: `results`, `total_matches`, `truncated`, and `next_offset`. The message is "N of M documents." without a query.

**The old name `list`.** From 0.5.0 to 0.5.1, a call with the name `list` ran `search` with no query, with the old defaults and a notice that the old name stops working in 0.6.0. Since 0.6.0, a call named `list` is an unknown tool, like the old names of 0.4.0. In limits, `list` counts as `search` for good: `tools_denied = ["list"]` switches off `search`, because a switched-off tool must never come back by a rename.

### Aliases of 0.4.0

A call with an old name of the nine renamed tools (`add`, `delete`, `deprecate`, `dispute`, `verify`, `reject`, `add_empty_folder`, `remove_folder`, `folders`) is an unknown tool. The names in limits (`tools_denied`, `approval.required_for`, `--tools-denied`, `--no-delete`) still count as the new names.

### `update_document`

| Parameter | Meaning |
|---|---|
| `replace` | An object `{old, new}`. `old` is a passage of the current body. It must occur exactly once. `new` is the text instead. `new` may be empty. |
| `append` | Text added at the end of the body. A line end is added first if the body does not end with one. |
| `add_tags` | Tags to add. A tag that is already there is ignored. |
| `remove_tags` | Tags to remove. A tag that is not there is ignored. |

- Only one of `body`, `replace`, and `append` may be sent. `tags` cannot be sent with `add_tags` or `remove_tags`. A tag in both `add_tags` and `remove_tags` is an error. These cases fail with `invalid_arguments`.
- `replace` with an `old` that does not occur fails with `validation_failed` and the `next` sentence "Read the document again with show, then send a passage that occurs once." A passage that occurs more than once fails the same way, with the count, and the `next` sentence says to add more text around it.
- The checks of `write-documents` run on the result, as for `body`. A change by `replace` or `append` counts as a change of the body, also for `trust` (see `verification`) and for new body links.
- A tag follows the rules of `manage-tags` (no `#`, no spaces, not empty).
- `changed` lists `body` for `replace` and `append`, and `tags` for `add_tags` and `remove_tags`.
- A call that changes nothing returns "No change." and commits nothing, as before.

### `show`

A deprecated document has `deprecation_reason`: the `Reason:` line of the newest commit that deprecated it (`deprecate_document` or `reject_document`). It is `null` when no reason is found, for example after a change by hand. An active document has no such field. For an older version (`show` with `version`) the field is left out.

### `updates`

| Parameter | Meaning |
|---|---|
| `offset` | How many changes to skip. Default 0. Needs `since`. |

- The result has `next_offset` when `truncated` is true.
- To read the rest of a long window, repeat the call with the same `since` and `offset` set to `next_offset`.
- A call with `offset` above 0 and no `since` fails with `invalid_arguments`, because a call without `since` moves the stored time, and the window would be lost.
- The description says that `until` is the value to pass as `since` in a later call. The stored time stays as a convenience for a person on one machine. A client that keeps its own time does not need it.

### Hints

The `next` sentences of `delete_document`, `delete_folder`, and `verify_document` name `delete_document`, `delete_folder`, and `dispute_document`. The hints of `show`, `history`, and the write tools name `search` and not `list`. The hints of `move`, `delete_folder`, and `folder_tree` name `folder_tree` and not `folders`. The stale-name test found the last four of these.

## Acceptance Criteria

- **Given** a client lists the tools, **When** the list is read, **Then** `list` is not in it, and `search` has no required parameter.
- **Given** documents in two folders, **When** `search` runs with no `query` and `folders`, **Then** it lists the documents of that folder sorted by `id`, at level 1, with the filters applied.
- **Given** a deprecated document, **When** `search` runs with no `query`, **Then** it is not listed. **When** it runs with `status: any`, **Then** it is listed.
- **Given** a call named `list`, **When** it runs, **Then** it is an unknown tool (since 0.6.0).
- **Given** `tools_denied = ["list"]`, **When** the client lists the tools, **Then** `search` is not listed and a call to it is denied.
- **Given** a call named `add`, `folders`, or `delete`, **When** it runs, **Then** it is an unknown tool.
- **Given** `tools_denied = ["delete", "folders"]`, **When** the client lists the tools, **Then** `delete_document` and `folder_tree` are not listed.
- **Given** a body with one passage, **When** `update_document` runs with `replace`, **Then** only that passage changes, the rest of the body is byte for byte the same, and `changed` lists `body`.
- **Given** a passage that does not occur, or occurs twice, **When** `update_document` runs with `replace`, **Then** it fails with `validation_failed` and changes nothing.
- **Given** `append`, **When** `update_document` runs, **Then** the text is at the end of the body and the old body is unchanged.
- **Given** `body` and `append` together, or `tags` and `add_tags` together, **When** `update_document` runs, **Then** it fails with `invalid_arguments`.
- **Given** a document with tags `a` and `b`, **When** `update_document` runs with `add_tags: ["c"]` and `remove_tags: ["a"]`, **Then** the tags are `b` and `c`, in one commit.
- **Given** a verified document and verification on, **When** `replace` changes the body, **Then** `trust` is `unverified`.
- **Given** `replace` adds a link to a missing document, **When** it runs, **Then** it fails with `validation_failed`.
- **Given** a document deprecated with a reason, **When** `show` runs, **Then** `deprecation_reason` is that reason.
- **Given** a restored document, **When** `show` runs, **Then** it has no `deprecation_reason`.
- **Given** more than 50 changes, **When** `updates` runs with `since`, and then with `since` and `next_offset`, **Then** the second call returns the next changes and none twice.
- **Given** `updates` with `offset` and no `since`, **When** it runs, **Then** it fails with `invalid_arguments`.
- **Given** the source of the tools, **When** the stale-name test runs, **Then** no hint or message names a retired tool name.
- **Given** the tool list, **When** its size is measured, **Then** it is smaller than before this item, and below the budget of `tool-conventions`.

## Constraints & Edge Cases

- The tool list has 23 tools (26 with verification on), one fewer than in 0.4.0.
- `replace` compares the passage as written, after line ends of the call are changed to LF. It does not trim.
- `search` without a query ignores `in`, and does not fail on it.
- `list` as a call alias worked for 0.5.x only and ended in 0.6.0. Its name in limits stays.

## Open Questions

None.
