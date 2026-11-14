# Tasks: Create a knowledge base

`requires`: `join-knowledge-base`, `plain-git-access`.

- [ ] `integration-testing`: add `World::empty()` (change in `tests/common/world.rs`).
- [ ] `src/kb/layout.rs`: `repo_is_empty` with unit tests (no commit, host files only, other files, untracked files).
- [ ] `src/kb/settings.rs`: the `to_toml` writer and the strict reader for the `init` input. Unit tests, including a name with a quote.
- [ ] `src/kb/init.rs`: `derive_slug`, `kb::init`, the `create_new` guard. Unit tests.
- [ ] `gitsync`: support a clone with no commit in `Repo::write` and the first push with `-u`. Add the verb `init` to `message.rs`.
- [ ] `join`: add `Startup::set_ready`, and use `repo_is_empty` for the state `empty`.
- [ ] `src/tools/init.rs`: schema, validation, write call, result. Register it in `src/tools/mod.rs`.
- [ ] `activity-and-undo`: `undo` refuses the verb `init` (note for that spec).
- [ ] `tests/it_create_knowledge_base.rs`: the acceptance criteria.
- [ ] Run the full suite.
