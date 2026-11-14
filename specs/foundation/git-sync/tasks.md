# Tasks: Git sync

`requires`: `memory-repository`.

- [x] `src/gitsync/cmd.rs`: isolated `git` runs and error mapping.
- [x] `src/gitsync/message.rs`: commit message format.
- [x] `src/gitsync/lock.rs`: the file lock with timeout.
- [x] `src/gitsync/log.rs`: last commit of a path, file at a commit, commits by author.
- [x] `src/gitsync/mod.rs`: `Repo::open`, `Repo::clone_from`, author setup, no-remote mode.
- [x] `src/gitsync/write.rs`: the write transaction with base check, commit, push, and preview.
- [x] `src/gitsync/sync.rs`: pull, rebase, push, and the conflict rule.
- [x] Unit tests for each file with temporary repositories.
- [x] `tests/it_git_sync.rs`: the acceptance criteria with two clones.
- [x] Run the full suite.
