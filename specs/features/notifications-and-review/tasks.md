# Tasks: Notices and review

_Prerequisite: `updates` is `implemented` or `done`._

- [x] `src/gitsync/mod.rs`: add `Repo::last_fetch_failed()` and set it in `refresh_if_stale` and in the write flow. Unit test.
- [x] `src/notices/text.rs`: the constants and builders. Unit tests for the "and N more" cut.
- [x] `src/notices/mine.rs`: `my_documents_notices` over `changes_since`. Unit tests: disputed, changed, own change ignored, cleared dispute.
- [x] `src/notices/mod.rs`: `NoticeBook`, `finalize`, `open_items`, `invalidate`, and the 3 session cap with the state key `notice_sessions`. Unit tests: once per session, cap, no notice means no field.
- [x] `src/mcp/context.rs`: add the `NoticeBook`.
- [x] `src/mcp/tool.rs`: call `finalize` after every call, also for errors.
- [x] `src/tools/updates.rs`: call `invalidate` after the stored time is written.
- [x] `src/tools/read_common.rs`: use `notices::text::CLONE_BEHIND`.
- [x] `tests/it_notifications.rs`: the test cases of `test.md`.
- [x] Run the full suite (`make check`).
