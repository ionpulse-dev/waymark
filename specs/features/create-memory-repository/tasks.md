# Tasks: Create a memory repository

`requires`: `join-memory-repository`, `plain-git-access`.

- [x] `integration-testing`: add `World::empty()` (change in `tests/common/world.rs`).
- [x] `src/memrepo/layout.rs`: `repo_is_empty` with unit tests (no commit, host files only, other files, untracked files).
- [x] `src/memrepo/settings.rs`: the `to_toml` writer and the strict reader for the `init` input. Unit tests, including a name with a quote.
- [x] `src/memrepo/init.rs`: `derive_slug`, `memrepo::init`, the `create_new` guard. Unit tests.
- [x] `gitsync`: support a clone with no commit in `Repo::write` and the first push with `-u`. Add the verb `init` to `message.rs`.
- [x] `join`: add `Startup::set_ready`, and use `repo_is_empty` for the state `empty`.
- [x] `src/tools/init.rs`: schema, validation, write call, result. Register it in `src/tools/mod.rs`.
- [x] `activity-and-undo`: `undo` refuses the verb `init` (note for that spec).
- [x] `tests/it_create_memory_repository.rs`: the acceptance criteria.
- [x] Run the full suite.
- [x] Adopt a folder that has the `.waymark` marker and no `settings.toml`: write only the settings file (`src/tools/init.rs`).
