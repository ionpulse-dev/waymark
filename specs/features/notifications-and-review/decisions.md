# Decisions: Notifications and review list

> **Note 2026-11-11:** "Notifications come from the app" is superseded by "Notices in results" below. "One review list" moves to the console.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Notifications come from the app — 2026-11-06

**Decision:** The CLI has no push. Notifications and dialogs come from the menu bar app. The first design had no popups at all.

**Reason:** The app closes the "no push" gap. Problems should be visible.

## One review list — 2026-11-06

**Decision:** One list for disputes. The pull request reviewer item was dropped.

**Reason:** One place to look.

## Notices in results — 2026-11-11

**Decision:** The server puts notices into tool results. The review list is in the console on a server.

**Reason:** no app and no background process in local mode. Alternative considered: email or webhook messages. They are a later option.

## Local mode has no review list tool — 2026-11-14

**Decision:** The review list of the requirements is `updates` plus the open notices in `status`. The console list is deferred with server mode. No new tool.

**Alternatives considered:** A `review_list` tool (rejected: the tool set stays small, and `updates` already lists disputes of the person with their reason).

**Reason:** One place to look is already there. Decided autonomously.

## A repeat rule: once per session, at most 3 sessions — 2026-11-14

**Decision:** A notice appears once per session. A notice about documents appears in at most 3 sessions, or until the person runs `updates` without `since`. The count lives in the state file under `notice_sessions`.

**Alternatives considered:**
- Repeat until seen (rejected: a person who never runs `updates` gets noise forever).
- Show once ever (rejected: an agent may ignore it in the first session).

**Reason:** Answers the open question. Three sessions give the person enough chances. Decided autonomously.

## Notices pass one filter after every call — 2026-11-14

**Decision:** The registry calls `notices::finalize` after every call, also for errors. It adds system notices and drops repeats, including repeats of notices that other specs make.

**Alternatives considered:** Each tool adds its own notices (rejected: no common once-per-session rule).

**Reason:** One place keeps the rule and the wording. Decided autonomously.

## "Of the person" means `created_by` — 2026-11-14

**Decision:** A document is of the person when `created_by` has the email of the person. A document that the person only edited does not count.

**Alternatives considered:** Anyone who ever committed to it (rejected: needs history for every document at every call, and creates noise).

**Reason:** It is cheap, clear, and matches "documents I wrote". Decided autonomously.

## Notices for documents are checked when `HEAD` changes — 2026-11-14

**Decision:** The check runs at the first call of a session and whenever `HEAD` differs from the last checked commit. The result is cached.

**Reason:** One `git diff` per new commit costs little. A check per call would cost more for no gain. Decided autonomously.

## Foundation additions — 2026-11-14

**Decision:** `Repo` gets `last_fetch_failed()`. `Context` gets the `NoticeBook`. The registry calls `finalize`. `status` calls `open_items`.

**Reason:** Reported to `git-sync`, `mcp-server`, and `maintenance`. Decided autonomously.

## Implementation notes — 2026-11-14

- Files: `src/notices/mod.rs` (`NoticeBook`, `finalize`, `open_items`, `invalidate`) and `src/notices/text.rs`. The plan's `mine.rs` is part of `mod.rs` (`document_notices`). `Repo::last_fetch_failed` and `Context.notices` are added. The registry passes every result, also an error, through `finalize`. Decided autonomously.
- Any notice that contains "may be behind" (the text that `refresh_if_stale` makes) is replaced by the fixed text `CLONE_BEHIND`, so that it appears once per session.
- `status` is not filtered: `finalize` adds `open_items` (the clone notice and the notices about documents, with no cap) and does not mark them as seen.
- The notices for documents use `changes_since` (see `updates`). A change that only touched `updated`, `updated_by`, and the dispute fields counts as no change (`Change.same_content`), so a dispute that was set and cleared gives no notice. `updates` itself still lists such a change. Decided autonomously.
- A new document of the person is never a notice. The window is the stored `last_updates` time, or 7 days.
- A session is one server process. The count of sessions is in the state file under `notice_sessions` (a key is the first 8 hex characters of the SHA-256 of the notice text). A key that is no longer open is removed at the next write.
- `pause-and-resume` still inserts the literal text "Writes are paused." in the registry. It passes through `finalize`, so it appears once. The constant `notices::text::PAUSED` is not used by the registry yet.
- No separate unit tests for `mine`. The end-to-end tests cover disputed, changed, own change, and cleared dispute.
