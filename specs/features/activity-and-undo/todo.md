# Todo: Activity and undo

## Bugs

- [ ] None known.

## Deferred

- [ ] Undo of another person's write by an owner.
- [ ] The console with a list of all people and an Undo button (server mode, see `web-console`).
- [ ] A redo shortcut.
- [ ] Rewrite of a history that holds a secret. Git history is never rewritten by waymark.

## Open Questions

- Needs from `git-sync`: `changed_files`, `later_commits_touching`, the message body, and revert helpers in `Repo`.
- Dependency on `move-documents`: the `Move:` line in the second commit of a move. `activity-and-undo` does not require that spec, so the test builds the commits by hand if needed.

## Done in the implementation (2026-11-14)

- All `[auto]` cases pass in `tests/it_activity_and_undo.rs` and the unit tests of `src/activity/mod.rs`.
