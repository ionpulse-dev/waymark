# Tasks: Session start

_`requires`: `project-binding`._

- [x] `src/mcp/session.rs`: `settings_json` and its unit tests (key order, types, defaults).
- [x] `src/mcp/session.rs`: `local_notices` with unit tests for each notice.
- [x] `src/mcp/instructions.rs`: `sanitize`, `build` for both forms, the cut, and unit tests (lengths, the word list, the empty-list words `none` and `all`).
- [x] `src/mcp/session.rs`: `facts` from the `Context`.
- [x] `src/mcp/result.rs`: use `settings_json` for every result.
- [x] `src/tools/status.rs`: the fields of this spec, including the `problem` form.
- [x] `src/mcp/mod.rs`: set `instructions` in the `initialize` result.
- [x] `tests/it_session_start.rs`: every test case of `test.md`.
- [x] Run the full suite.
