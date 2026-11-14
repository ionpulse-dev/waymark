# Decisions: Document history

## A tool `history` and a `version` on `show` — 2026-11-15

**Decision (product owner, decided in review):** The versions of a document are listed by a new read tool. An old version is read with `show`.

**Alternatives considered:** A tool `show_version` (a second tool for the same output as `show`). A diff tool (an agent can compare two texts itself, and a diff format adds a new contract).

**Reason:** `show` already has the output format of a document. One new parameter reuses it.

## The version is the short commit hash — 2026-11-15

**Decision (decided autonomously):** A version is the short hash of a commit, as in `show` and `base` today.

**Reason:** No new concept. The `version` of an old entry is not a valid `base` for a write, and `show` says so with `is_current: false`.
