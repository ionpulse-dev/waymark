# Todo: Document names

## Bugs

None.

## Deferred

- [ ] A rename with a text change by hand is a delete and a new file. A similarity rule could make it a move. Only if people ask for it.
- [ ] A pending hand edit of a document that links to a renamed document is not rewritten. If it still links to the old ID, its broken link counts as old, because the rename is already in the folder. Obsidian rewrites such links itself. Check this in real use.
- [ ] If the link commit of a hand rename fails after the rename commit, the rename stays committed without its link fixes. Take the rename back with `undo_commit_keep_files` in that case.
- [ ] The approval question of `sync` lists the edits, not the other documents that link fixes change.
- [ ] With `approval.required_for = ["sync"]`, an agent write still publishes hand edits without a question. Only the matching tools hold an edit back. Decide whether `sync` in the list should hold them too.
- [ ] Measure the tool list (`it_context_budget.rs`). This change adds about 300 bytes. The margin is about 460 bytes.
- [ ] Freeze and version the Waymark Memory Standard (see `document-format/decisions.md`). The product owner decides when.

## Open Questions

None.
