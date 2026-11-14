# Todo: Git sync

## Bugs

None.

## Deferred

- Background pull and the workspace of a shared server (server mode).
- Offline queue and retries.
- Signed commits.

## Open Questions

None.

## Notes from the implementation (2026-11-14)

- The criterion "No network, `search` answers from the clone with a notice" is tested at the Git level in `tests/it_git_sync.rs` (`a_read_without_network_gives_a_notice_and_does_not_fail`). The test through the `search` tool is in `search-and-read`.
