# Tasks: Notices and review

_Prerequisite: `updates` is `implemented` or `done`._

- [ ] `src/gitsync/mod.rs`: add `Repo::last_fetch_failed()` and set it in `refresh_if_stale` and in the write flow. Unit test.
- [ ] `src/notices/text.rs`: the constants and builders. Unit tests for the "and N more" cut.
- [ ] `src/notices/mine.rs`: `my_documents_notices` over `changes_since`. Unit tests: disputed, changed, own change ignored, cleared dispute.
- [ ] `src/notices/mod.rs`: `NoticeBook`, `finalize`, `open_items`, `invalidate`, and the 3 session cap with the state key `notice_sessions`. Unit tests: once per session, cap, no notice means no field.
- [ ] `src/mcp/context.rs`: add the `NoticeBook`.
- [ ] `src/mcp/tool.rs`: call `finalize` after every call, also for errors.
- [ ] `src/tools/updates.rs`: call `invalidate` after the stored time is written.
- [ ] `src/tools/read_common.rs`: use `notices::text::CLONE_BEHIND`.
- [ ] `tests/it_notifications.rs`: the test cases of `test.md`.
- [ ] Run the full suite (`make check`).
