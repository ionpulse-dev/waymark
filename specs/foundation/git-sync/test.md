# Tests: Git sync

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| Two people write different documents, both reach the remote | `two_writers_different_files` |
| Old base gives `stale_base` | `stale_base_rejected` |
| A real conflict keeps both versions and aborts the rebase | `conflict_keeps_both_versions` |
| A rejected push gives `sync_failed`, no retry | `rejected_push_not_retried` |
| The author comes from waymark's settings | `author_from_settings` |
| `sync` waits for the lock | `sync_waits_for_lock` |
| `preview: true` writes nothing | `preview_writes_nothing` |
| No network, `search` answers from the clone with a notice | `offline_read_has_notice` |
| A read sees a document that another clone pushed a moment ago | `read_sees_fresh_push` |
| Read calls at the same time share one fetch | `parallel_reads_share_one_fetch` |
| A failed fast-forward or rebase is tried again by the next read | `failed_update_retried` |

## Test Cases

- [x] [auto] The first eight criteria above (`tests/it_git_sync.rs`)
- [x] [auto] The three criteria of 2026-11-17: fresh push, shared fetch, retry after a failed update (`tests/it_git_sync.rs`, `src/gitsync/sync.rs`)
- [x] [auto] The repository is never left in a rebase or merge state after any failure (`tests/it_git_sync.rs`)
- [x] [auto] The conflict copy has a unique title, `disputed: true`, and the name rule (`tests/it_git_sync.rs`)
- [x] [auto] The commit message has the format `waymark: <verb> "<title>"` and no agent tag (`src/gitsync/message.rs`)
- [x] [auto] A repository without a remote works for writes and shows a notice (`tests/it_git_sync.rs`)
- [x] [auto] A fresh clone from `--repo` happens once and is reused (`tests/it_git_sync.rs`)
- [x] [auto] A missing credential fails fast with no prompt (`tests/it_git_sync.rs`, remote path does not exist)
- [x] [auto] Two processes on one clone: the second waits for the lock, then succeeds (`src/gitsync/lock.rs`)
- [ ] [manual] A real Git host with the person's own login: write a document and see it on the host. Steps: configure the MCP entry with an SSH URL, write a document through an agent, open the repository on the host. Expected: one commit with the person's name and email.

## Notes

The manual item belongs to the final manual checklist.
