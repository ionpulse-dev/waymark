# Decisions: Restore and replace documents

## A tool of its own, `restore_document` — 2026-11-15

**Decision (product owner, decided in review):** Reactivating a document is a tool, `restore_document`.

**Alternatives considered:** `deprecate_document` with `clear: true`, as `dispute_document` does (one tool less, but an agent that looks for "undo deprecate" will not find it, and the approval text would say "deprecate" for a restore). Setting `status` through `update_document` (the status is not a field that `update_document` may change, so that a deprecation is always a visible act).

**Reason:** A clear name makes an agent find it. The cost is one more tool description.

## A restore of a replaced document is allowed, with a notice — 2026-11-15

**Decision (decided autonomously):** Restoring a document that an active document replaces works, and the result says which one replaces it.

**Alternatives considered:** Refuse. The standard says that the replaced document SHOULD be deprecated, not MUST. Refusing would block a person who knows that the replacement was wrong.

**Reason:** The person decides. The notice makes the conflict visible.

## `restore_document` resets trust — 2026-11-15

**Decision (decided autonomously):** With verification on, a restored document is `unverified`.

**Reason:** `reject_document` clears the trust. A document that was rejected and is back needs a new review.

## `supersedes` in `update_document` adds, and does not replace — 2026-11-15

**Decision (decided autonomously):** The new ID is added to the list. There is no way to remove an entry with this parameter.

**Reason:** A document can replace several older ones. Removing an entry is rare, and `update_document` already changes `related` as a whole list. A later change can add it if needed.
