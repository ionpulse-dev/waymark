# Tasks: Knowledge base repository

`requires`: `document-format`.

- [x] Add `toml`, `serde`, `uuid` to `Cargo.toml`.
- [x] `src/kb/settings.rs`: structs, defaults, validation, unknown keys as notices.
- [x] `src/kb/layout.rs`: root finding and the `init` file set.
- [x] Write the three templates in `src/kb/templates/`.
- [x] `src/kb/mod.rs`: `KnowledgeBase::open(path)` and `KnowledgeBase::init(path, name, slug)`.
- [x] Unit tests for settings parsing and layout.
- [x] `tests/it_knowledge_base_repo.rs` (after `mcp-server` exists).
- [x] Run the full suite.
- [x] The key `schema`: parse it, refuse a higher number with `setup_problem`, write `schema = 1` in `init`, tests, and docs (2026-11-14).
