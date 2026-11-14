# Tasks: Move documents

`requires`: `write-documents`, `git-sync`.

- [ ] Confirm that `Repo::write` accepts several commits, a check step before the push, and a rollback of this transaction's unpushed commits. If not, add this to `git-sync` first.
- [ ] `src/writes/moveplan.rs`: the rename map and the relative path function, with unit tests (nested folders, depth change, a document that links to a moved one).
- [ ] Extend `src/writes/linkfix.rs` with the retarget rules of the requirements: Markdown links, full-path wikilinks, short-name wikilinks, alias and heading kept, code skipped. Unit tests.
- [ ] `src/tools/move_documents.rs`: input checks, `folders` check, the transaction, the pure-rename check, the history check, preview, and the result.
- [ ] `src/tools/mod.rs`: register `move`.
- [ ] `tests/it_move_documents.rs`: the cases of `test.md`.
- [ ] Run the full suite.
