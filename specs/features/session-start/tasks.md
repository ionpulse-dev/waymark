# Tasks: Session start

_`requires`: `project-binding`._

- [ ] `src/mcp/session.rs`: `settings_json` and its unit tests (key order, types, defaults).
- [ ] `src/mcp/session.rs`: `local_notices` with unit tests for each notice.
- [ ] `src/mcp/instructions.rs`: `sanitize`, `build` for both forms, the cut, and unit tests (lengths, the word list, the empty-list words `none` and `all`).
- [ ] `src/mcp/session.rs`: `facts` from the `Context`.
- [ ] `src/mcp/result.rs`: use `settings_json` for every result.
- [ ] `src/tools/status.rs`: the fields of this spec, including the `problem` form.
- [ ] `src/mcp/mod.rs`: set `instructions` in the `initialize` result.
- [ ] `tests/it_session_start.rs`: every test case of `test.md`.
- [ ] Run the full suite.
