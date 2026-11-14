# Tasks: Move documents

`requires`: `write-documents`, `git-sync`.

- [x] `Plan.first` (a `FirstCommit`) is the first commit of a write: `writes::execute` makes it, runs the history check, and undoes it with `reset --mixed` if the check fails. `Repo::write` now also pushes commits that the closure made, when its own commit has no change. `Tx` removes empty folders after a rename and a rollback.
- [x] the rename map (`Remap`) and the relative path function are in `src/writes/linkfix.rs`, with unit tests (nested folders, depth change, a document that links to a moved one).
- [x] Extend `src/writes/linkfix.rs` with the retarget rules of the requirements: Markdown links, full-path wikilinks, short-name wikilinks, alias and heading kept, code skipped. Unit tests.
- [x] `src/tools/move_documents.rs`: input checks, `folders` check, the transaction, the pure-rename check, the history check, preview, and the result.
- [x] `src/tools/mod.rs`: register `move`.
- [x] `tests/it_move_documents.rs`: the cases of `test.md`.
- [x] Run the full suite.
