# Todo: Maintenance

## Bugs

None.

## Deferred

- The `/health` endpoint for operators (out of scope for now, server mode).
- Update of a server image (server mode).
- A check that the `AGENTS.md` of the repository has the pinned version of the plain-Git rules (needs `plain-git-access` first).
- A check that warns when the repository is public (see `create-memory-repository`).
- A check of the free disk space and the size of the clone.
- A `waymark doctor` command for people without an agent. Decided against for now.

## Open Questions

None.

## Notes 2026-11-14

- A `status` call takes up to 10 seconds on a bad network (the remote probe). Accepted.
- The 5 second time-out of `git --version` is built (`git_check`), but no test makes it fire.
