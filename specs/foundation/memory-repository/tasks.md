# Tasks: Memory repository

`requires`: `document-format`.

- [x] Add `toml`, `serde`, `uuid` to `Cargo.toml`.
- [x] `src/memrepo/settings.rs`: structs, defaults, validation, unknown keys as notices.
- [x] `src/memrepo/layout.rs`: root finding and the `init` file set.
- [x] Write the three templates in `src/memrepo/templates/`.
- [x] `src/memrepo/mod.rs`: `MemRepo::open(path)` and `MemRepo::init(path, name, slug)`.
- [x] Unit tests for settings parsing and layout.
- [x] `tests/it_memory_repository.rs` (after `mcp-server` exists).
- [x] Run the full suite.
- [x] The key `schema`: parse it, refuse a higher number with `setup_problem`, write `schema = 1` in `init`, tests, and docs (2026-11-14).
- [x] Automatic repairs of the settings file: the list in `src/memrepo/repairs.rs`, the call in the write flow, the verb `repair`, tests (2026-11-14).
- [ ] Migrations and the tool `migrate` (designed in "Versions and updates"; build them with the first schema 2).
