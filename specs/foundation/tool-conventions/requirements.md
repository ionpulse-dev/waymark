---
type: foundation
status: implemented
created: 2026-11-15
updated: 2026-11-17
requires:
  - mcp-server
---

# Tool conventions

## Technical Goal

All tools follow one set of rules for names, descriptions, parameters, paging, and dates. An agent that has learned one tool can guess the next one. This item states the rules, renames the tools that break them, and keeps old names working for a while.

## Dependents

- Every feature spec that defines a tool. Their tool names change with this item.
- `agent-tool-setup`: the permission advice names every tool.
- `trust-model` and `write-approval`: limits and approval rules name tools.

## Deliverables

- The naming scheme (below), and the renamed tools.
- Descriptions in one form, and a description for every parameter.
- One paging style and one date style for all tools.
- The old tool names as aliases, with a notice.
- Tests that check the rules for every tool, so that a new tool cannot break them.

## Scope

**In scope:** names, descriptions, parameter descriptions, paging parameters, date parameters, the aliases of renamed tools, and the budget of the tool list.

**Out of scope:** what a tool does (see the feature specs), and the commit subjects (see decisions).

## Constraints

- **Names** are lowercase `snake_case`.
- A write tool that acts on exactly one document is `<verb>_document`: `add_document`, `update_document`, `delete_document`, `deprecate_document`, `restore_document`, `dispute_document`, `verify_document`, `reject_document`.
- A tool that acts on folders is `<verb>_folder` (`create_folder`, `delete_folder`). The read tool of the tree is `folder_tree`.
- A tool that acts on tags is `list_tags` or `rename_tag`.
- A tool that acts on documents and folders, or on the whole memory repository, has a bare verb: `move`, `sync`, `undo`, `init`.
- A read tool has a bare noun or verb: `status`, `search`, `show`, `history`, `stats`, `updates`, `lint`, `activity`, `review`.
- **Renamed tools (old name, new name):** `add` to `add_document`; `delete` to `delete_document`; `deprecate` to `deprecate_document`; `dispute` to `dispute_document`; `verify` to `verify_document`; `reject` to `reject_document`; `add_empty_folder` to `create_folder`; `remove_folder` to `delete_folder`; `folders` to `folder_tree`.
- **Aliases of calls.** The old names of the nine renames stopped working in version 0.5.0: a call with one is an unknown tool. The alias `list` (merged into `search`, see `lean-tools`) stopped working in version 0.6.0. No call alias remains. Limits and approval checks use the new name. The tool list shows only new names.
- **Aliases in limits.** An old name (also `list`) in `tools_denied` or `approval.required_for` (settings files, `--tools-denied`, `--no-delete`) counts as the new name. This stays, because a switched-off tool must never come back by a rename.
- **Description form.** A description is plain and short. It says what the tool does, then what it returns ("Returns ..."), and for a write tool that it commits and pushes. It names no advice. Read tools end with "Changes nothing."
- **Parameters.** Every parameter has a `description`. It says the meaning, and the default or the allowed values when there are any.
- **Paging.** A tool that returns a list takes `limit` and one cursor. `search` and `list_tags` use `offset` and return `next_offset` when `truncated` is true. `activity` and `history` read a log, which gets new entries at the top, so they use `before` (a commit) and return `next_before`.
- **Dates.** A parameter that takes a time (`since`) accepts `YYYY-MM-DD` or an RFC 3339 time. A parameter that filters a document date (`updated_after`, `updated_before`) takes `YYYY-MM-DD`.
- **Budget.** The tool list stays below 28000 bytes, and the descriptions below 9000 characters, for all tools together.

## Acceptance Criteria

- **Given** a running server, **When** a client lists the tools, **Then** every write tool that acts on one document is named `<verb>_document`, and none of the old names is listed.
- **Given** a running server, **When** a client lists the tools, **Then** every parameter of every tool has a non-empty description.
- **Given** a running server, **When** a client lists the tools, **Then** every description says what the tool returns, and every write tool says that it commits and pushes.
- **Given** a call with `list` or with one of the nine names of 0.4.0, **When** the call runs, **Then** it is an unknown tool.
- **Given** a limit `tools_denied = ["delete"]`, **When** a client lists the tools or calls `delete_document`, **Then** the tool is not listed and the call is denied.
- **Given** an `activity` call, **When** `since` is an RFC 3339 time, **Then** the call works like a call with a date.
- **Given** the tool list, **When** its size is measured, **Then** it is below the budget.

## Open Questions

None.
