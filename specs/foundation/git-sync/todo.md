# Todo: Git sync

## Bugs

- **A clone with commits does not pick up a later `init` of the remote.** Found on 2026-11-17 by the tests of `menu-bar-app`. What happens: a person clones a repository that has commits but no memory repository yet. Another person runs `init` and pushes. A `waymark` that starts in the first clone answers every tool with "This repository is empty. Call `init`." and does not pull. What should happen: the start, or the next call, updates the clone (as `refresh_if_unborn` does for a clone with no commit) and finds the memory repository. Rare in practice: people join after `init`.
- **`updates` can miss a change in the same second as the stored time.** Found on 2026-11-17 by the tests of `menu-bar-app`. Commit times have whole seconds, and a change in the second of the stored `since` is not listed. Low impact. A fix compares with `>=` and removes duplicates.

## Deferred

- Background pull and the workspace of a shared server (server mode).
- Offline queue and retries.
- Signed commits.

## Open Questions

None.

## Notes from the implementation (2026-11-14)

- The criterion "No network, `search` answers from the clone with a notice" is tested at the Git level in `tests/it_git_sync.rs` (`a_read_without_network_gives_a_notice_and_does_not_fail`). The test through the `search` tool is in `search-and-read`.
