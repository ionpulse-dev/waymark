# Tasks: Git sync

`requires`: `knowledge-base-repo`.

- [ ] `src/gitsync/cmd.rs`: isolated `git` runs and error mapping.
- [ ] `src/gitsync/message.rs`: commit message format.
- [ ] `src/gitsync/lock.rs`: the file lock with timeout.
- [ ] `src/gitsync/log.rs`: last commit of a path, file at a commit, commits by author.
- [ ] `src/gitsync/mod.rs`: `Repo::open`, `Repo::clone_from`, author setup, no-remote mode.
- [ ] `src/gitsync/write.rs`: the write transaction with base check, commit, push, and preview.
- [ ] `src/gitsync/sync.rs`: pull, rebase, push, and the conflict rule.
- [ ] Unit tests for each file with temporary repositories.
- [ ] `tests/it_git_sync.rs`: the acceptance criteria with two clones.
- [ ] Run the full suite.
