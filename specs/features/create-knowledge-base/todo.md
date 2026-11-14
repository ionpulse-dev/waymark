# Todo: Create a knowledge base

## Bugs

None.

## Deferred

- Create the remote repository through the API of a Git host (needs a token, which waymark never holds).
- A warning in `status` if the repository is public.
- Server-mode `init` for a hosted knowledge base (out of scope for now).
- Update `git-sync` and `activity-and-undo` for the verb `init` (see `decisions.md`).
- A `limits` template for typical teams (for example "read-only for agents").

## Open Questions

None.

## Notes 2026-11-14

- The activity entry of an `init` commit is not special. Check how `activity` shows it when `maintenance` is done.
- `git-sync` and `activity-and-undo` lists of verbs should name `init` (the code has it, the text of those specs has eight verbs).
