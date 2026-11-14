# Decisions: Manage tags

## Two tools, `list_tags` and `rename_tag`, and a merge by rename — 2026-11-15

**Decision (product owner, decided in review):** Tags get a read tool and a write tool. A merge is a rename to a tag that exists.

**Alternatives considered:** `stats` with `by: tag` (it counts, but a tag list needs paging and a status filter). A separate `merge_tag` (the same code as a rename).

**Reason:** Tags are free text, so near-duplicates are certain. The two tools cover discovery and repair.

## No `expect_documents` guard — 2026-11-15

**Decision (decided autonomously):** `rename_tag` has no count guard like `delete_folder`.

**Alternatives considered:** A required count. It protects against loss of data, and a rename loses nothing: one `undo` reverts the commit.

**Reason:** `preview`, the approval question, and `undo` are enough.

## Documents outside the `folders` limit are not changed — 2026-11-15

**Decision (decided autonomously):** The tool changes only documents that the caller may see. It says nothing about hidden documents.

**Reason:** A hidden document must stay hidden, also its count. The owner can run the rename with wider access.
