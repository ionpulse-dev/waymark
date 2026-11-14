# Todo: Manual edits and sync

_Known bugs, deferred improvements, and open questions. Updated throughout the feature lifecycle._

## Bugs

None.

## Deferred

- [ ] A helper for people without agents (a small command or an Obsidian plugin that runs the same steps). Blocked by demand.
- [ ] A commit after a quiet minute while the server runs. Rejected for now (see `decisions.md`). Revisit if people leave edits unpublished.
- [ ] A check of links that a hand move or delete breaks in other documents, as part of `sync`. Today `lint` finds them.
- [ ] Commit non-document files such as images with a rule for size. Blocked by a decision on attachments.
- [ ] The background pull and the editing flow on a server. Out of scope for now (server mode).

## Open Questions

None.

## Conflicts to resolve in other specs

- `mcp-server` plan, step 4, hides write tools in read-only and paused states. `sync` must stay listed (see `decisions.md`).
- `git-sync` plan: `Repo::sync` needs a hook parameter and `Repo::write` needs an installed pending-edit hook and a check for uncommitted target paths.
- `git-sync` requirements say "`sync` commits pending manual edits". This spec fixes the details: lint first, partial commit, message `waymark: sync "<title>"`.

## Done in the implementation (2026-11-14)

- 17 of 18 `[auto]` cases pass in `tests/it_manual_edits.rs`. T7 (approval for `sync`) waits for `write-approval` and is `#[ignore]`.
- The two `[manual]` Obsidian checks (T20, T21) are for the product owner.

- `sync` and the new body link rule of `write-documents` (decided 2026-11-14): a hand-edited document with a new link to a missing document should fail the check like a tool write. Check the current behaviour, and decide.
