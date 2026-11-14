# Plan: Restore and replace documents

## Overview

`restore_document` is a copy of the pattern of `deprecate_document` with the reverse change. `update_document` reuses the `supersedes` code of `add_document`.

## Components Affected

- `src/gitsync/message.rs`: the verb `restore`.
- `src/tools/mod.rs`: registers the tool.
- `src/tools/update_document.rs`: the `supersedes` parameter.
- `src/writes/mod.rs`: a helper `apply_supersedes` that `add` and `update_document` share (sets the list entry, deprecates the old document).
- `src/tools/add.rs`: uses the helper.

## New Files

- `src/tools/restore.rs`: the `restore_document` tool.
- `tests/it_restore_and_replace.rs`: the cases of `test.md`.

## Technical Approach

1. `restore_document` reads the document with `writes::read_doc`, checks `base`, refuses an active document, and sets `status: active`. With verification on, it sets `trust: unverified` and removes `verified_by` and `verified`. It calls `writes::touch` and `writes::check_document`.
2. It looks for active documents that list the ID in `supersedes` (`Store::load`) and adds a notice.
3. The verb is `restore`. The commit subject is `waymark: restore "<title>"`. A given `reason` goes into the commit body.
4. `apply_supersedes(tx, memrepo, env, new_doc, old_id, ...)` checks the rules (not itself, visible, no loop), appends the link, and returns the change that deprecates the old document.
5. `update_document` calls the helper when `supersedes` is set. Then the link check skips the old ID, as `add_document` does.

## Cross-Feature Dependencies

- `write-documents`: `add_document` with `supersedes` keeps working.
- `verification`: the trust reset.

## Risks

- A loop (A replaces B and B replaces A) must be refused. The check reads the `supersedes` list of the other document.

## Open Questions

None.
