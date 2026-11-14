# Tasks: Knowledge base repository

`requires`: `document-format`.

- [ ] Add `toml`, `serde`, `uuid` to `Cargo.toml`.
- [ ] `src/kb/settings.rs`: structs, defaults, validation, unknown keys as notices.
- [ ] `src/kb/layout.rs`: root finding and the `init` file set.
- [ ] Write the three templates in `src/kb/templates/`.
- [ ] `src/kb/mod.rs`: `KnowledgeBase::open(path)` and `KnowledgeBase::init(path, name, slug)`.
- [ ] Unit tests for settings parsing and layout.
- [ ] `tests/it_knowledge_base_repo.rs` (after `mcp-server` exists).
- [ ] Run the full suite.
