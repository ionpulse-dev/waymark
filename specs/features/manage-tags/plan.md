# Plan: Manage tags

## Overview

`list_tags` is a read tool over the `Store`. `rename_tag` builds one `Plan` with a `FileChange::Write` for each changed document, like `move` and `remove_folder` do for many files.

## Components Affected

- `src/gitsync/message.rs`: the verb `rename-tag`.
- `src/tools/mod.rs`: registers both tools.
- `src/writes/mod.rs`: a helper for a valid tag (`check_tag`) that `add_document` and `update_document` can use later.

## New Files

- `src/tools/list_tags.rs`: the `list_tags` tool.
- `src/tools/rename_tag.rs`: the `rename_tag` tool.
- `tests/it_manage_tags.rs`: the cases of `test.md`.

## Technical Approach

1. `list_tags` loads the store, applies the scope and the status filter, counts tags in a `BTreeMap`, sorts by count and name, and pages with `offset` and `limit`.
2. `rename_tag` checks the tags (`check_tag`: no `#`, no whitespace, not empty). It collects the visible documents that have `from`. For each, it replaces `from` by `to`, removes duplicates, keeps the order, calls `writes::touch`, and checks the document with `writes::check_document`. A problem in any document stops the call.
3. The plan has one change for each document. The commit subject is `waymark: rename-tag "<from> to <to>"`.
4. The approval summary lists the first line and the changed IDs.

## Cross-Feature Dependencies

- `write-approval`: the approval text for a write with many paths.
- `activity-and-undo`: the new verb is read back and undone like the others.

## Risks

- A rename that touches many documents in one call. The guards are `preview`, the approval question, the folder limit, and `undo`.

## Open Questions

None.
