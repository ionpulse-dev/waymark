# Decisions: Search filters and paging

## `author` matches the creator or the last editor — 2026-11-15

**Decision (decided autonomously):** `author` is a text that must occur in `created_by` or in `updated_by`, ignoring case.

**Alternatives considered:** Only `created_by` (misses the person who keeps a document current). Exact match of the whole `Name <email>` text (an agent rarely knows it exactly). The Git author of the last commit (needs Git history for every document, which is slow).

**Reason:** Both fields are in the frontmatter, so the filter needs no Git call. A part of a name or an email is enough.

## The date filters use the `updated` field — 2026-11-15

**Decision (decided autonomously):** `updated_after` and `updated_before` compare with `updated`, and both ends count.

**Reason:** `updated` is the date that tells how current a document is. `updates` already answers "what changed since a time" from Git, so these filters stay simple.
