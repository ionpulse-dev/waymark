# Plan: Notices and review

## Overview

A `notices` module sits between a tool and its result. After every call, the registry hands the notices that the tool made to `notices::finalize`. It adds the system notices, drops repeats, and applies the 3 session cap. `status` uses `notices::open_items` for the full list. No new tool.

## Components Affected

- `mcp` — `Context` holds the `NoticeBook`. The registry in `src/mcp/tool.rs` calls `finalize` after every call, also for errors.
- `store::changes` and `state` (from `updates`) — the check for documents and the stored time.
- `config` — the `paused` flag.
- `gitsync` — `Repo` gets `last_fetch_failed()` (a flag set by `refresh_for_read` and by the write flow).
- `search-and-read` — its read refresh uses the text constant of this spec.
- `maintenance` and `session-start` — `status` calls `open_items`.

## New Files

- `src/notices/mod.rs` — `NoticeBook`, `finalize`, `open_items`, `invalidate`.
- `src/notices/text.rs` — the notice texts as constants and small builders (`disputed(ids)`, `changed(ids)`, `CLONE_BEHIND`, `PAUSED`).
- `src/notices/mine.rs` — `my_documents_notices(ctx) -> Vec<Notice>` over `store::changes::changes_since`.
- `tests/it_notifications.rs` — end-to-end tests with two clones.

## Technical Approach

1. **Notice type.** `Notice { kind, text, key }`. `kind` is `Disputed`, `Changed`, `CloneBehind`, `Paused`, or `Other` (a notice from a tool). The key is the first 8 hex characters of the SHA-256 of the text.
2. **Finalize.** Input: the tool name, the notices of the tool, and the `Context`. Steps: (a) add `Paused` if `EffectiveSettings` says paused; (b) add `CloneBehind` if `Repo::last_fetch_failed()`; (c) unless the tool is `updates`, add the notices for documents (step 4); (d) drop any notice whose key is in `NoticeBook.seen`; (e) add the kept keys to `seen`; (f) return the texts, or none.
3. **Cap across sessions.** When a notice of kind `Disputed` or `Changed` is shown for the first time in this process, add 1 to `notice_sessions[key]` in the state file. Before showing, skip it if the count is 3 or more. Keys that are no longer current are removed from the file at each write.
4. **Documents.** `my_documents_notices` runs when `HEAD` differs from `NoticeBook.head` or the book was invalidated. It calls `changes_since(last_updates or now - 7 days)`, keeps entries with `mine: true`, and makes one `Disputed` notice for the disputed ones and one `Changed` notice for the rest. It names at most 3 IDs, then "and N more". The result is cached in the book until `HEAD` changes.
5. **Invalidate.** The `updates` tool calls `NoticeBook::invalidate()` after it stores the time.
6. **Open items.** `open_items(ctx)` returns the same notices as `finalize` would add, with no `seen` filter and no cap, plus the pending notices of the tool. `status` returns them in its result.
7. **Errors.** Any error inside step (b) or (c) is logged to stderr and ignored.
8. **Wording.** The texts are plain English, with no Git words. IDs are in backticks.

## Cross-Feature Dependencies

- `updates` — `changes_since`, `State`, `invalidate`. Listed in `requires`.
- `maintenance` and `session-start` — must call `open_items` in `status`. Reported as a note for those specs.
- `pause-and-resume` — must not add its own "paused" notice. It uses `notices::text::PAUSED` through `finalize`.
- `manual-edits` — adds its own notice texts through the tool notices. They pass through `finalize`.

## Risks

- A repeated notice annoys the person. Mitigation: once per session and at most 3 sessions.
- The check costs a `git diff` after each new commit. Mitigation: it runs only when `HEAD` changed. The result is cached.
- A person who never runs `updates` keeps getting notices for 3 sessions. Accepted.

## Open Questions

None.
