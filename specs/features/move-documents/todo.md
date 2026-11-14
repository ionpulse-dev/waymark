# Todo: Move documents

## Bugs

- [ ] None known.

## Deferred

- [ ] A report of inbound links from other repositories. Today a fixed notice says they are not checked.
- [ ] A way to merge a folder into an existing folder.
- [ ] A move of a folder that keeps the old IDs working as redirects in `show` (needs `search-and-read`).

## Open Questions

- Conflict with `search-and-read`: its criterion "`show` with an old ID names the new ID" is not in this spec, because `move-documents` does not require `search-and-read`. `search-and-read` must cover it with `git log --follow`.
- Needs from `git-sync`: several commits in one `Repo::write`, a check step before the push, and a rollback of the unpushed commits of the transaction.
