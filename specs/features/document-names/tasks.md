# Tasks: Document names

- [x] `standard/waymark-1.0.md`, section 8: the short-link rule. Then `document-format` (requirements and decisions).
- [x] `src/writes/linkfix.rs`: `Remap::wiki_target` for a target that does not move, and `Remap::added`. Unit tests.
- [x] `src/writes/shortlinks.rs`: `add_fixes_for_new_ids`. Register it in `src/writes/mod.rs`.
- [x] `src/writes/slug.rs`: `check_name`. Unit tests.
- [x] `src/tools/add.rs`: the field `name`, the taken-name error, the short-link fixes, and `links_updated`.
- [x] `src/gitsync/sync.rs`: `Repo::sync` with a hook that returns commit hashes.
- [x] `src/manual/pending.rs`: the kind `Moved` from an exact pair.
- [x] `src/manual/mod.rs`: rules per edit, planned commits, `commit_planned`, and `commit_before_write` with `env` and held edits.
- [x] `src/writes/mod.rs`: pass `env` to `commit_before_write`.
- [x] `src/tools/sync.rs`: the description, approval for the matching tools, and the result per edit.
- [x] Update the existing tests for one commit per edit and the new texts.
- [x] `tests/it_document_names.rs`: the cases of `test.md`.
- [x] Specs: `write-documents`, `manual-edits`, `move-documents`, `quality-checks`, `INDEX.md`.
- [x] Docs: `docs/local-install.md` (both `AGENTS.md` suggestions) and `CHANGELOG.md`.
- [x] Review 1 fixes: `Tx::forget`, conflicts of several commits, `activity` grouping of a `sync` move, `tools_denied` only in `prepare`, early return, the newest planned text in short-link fixes, verification by hand, the old path of a blocked or approved rename, and `it_git_sync.rs`.
- [x] Review 2 fixes: the title test, `activity` grouping after a rebase, `forget` after each commit, verification fields, the strict name, the error text of a failed trial merge, standard rule 6.6, and the specs `write-approval` and `activity-and-undo`.
- [x] Review 3 fixes: deletes last and checked against the committed text of blocked edits, held edits hold every later edit, a stricter `activity` fallback, the error of a failed merge, and spec gaps.
- [x] Run `cargo fmt`, then `make check` and `make linux-test`. Done on 2026-11-17 with Rust 1.100.0 (installed with mise): `make check` passes after one clippy fix. `make linux-test` passes on Linux arm64 (Colima with containerd and `nerdctl`, 2026-11-17) after a test fix in `personal-server` (see its `decisions.md`). The x86_64 run (`ARCH=amd64`) is not done.
