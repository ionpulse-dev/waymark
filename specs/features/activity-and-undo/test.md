# Tests: Activity and undo

_`[auto]` items are verified by running the automated test suite. `[manual]` items require human verification in a real environment._

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:----------|:----------|
| A written document is first in `activity` | `activity_lists_newest_first` |
| A clean undo makes a pushed revert commit under my account | `undo_reverts_and_pushes` |
| A conflicting later change gives `stale_base` | `undo_conflict_stale_base` |
| A later change to another part reverts cleanly | `undo_clean_with_later_change` |
| Writes by others do not show | `activity_only_own_writes` |
| `read-only` gives `access_denied` | `undo_read_only` |
| A move is undone in one commit | `undo_move_one_commit` |
| An undone write has `undone: true` | `activity_marks_undone` |
| A second undo fails | `undo_twice_fails` |
| Paging with `limit` and `before` | `activity_paging` |
| `preview: true` commits nothing | `undo_preview` |

## Test Cases

- [ ] [auto] All criteria above, through the real binary and the `World` harness (`tests/it_activity_and_undo.rs`)
- [ ] [auto] The undo commit has the subject `waymark: undo "<title>"`, the body `Reverts: <hash>`, and the author of the person (`tests/it_activity_and_undo.rs`)
- [ ] [auto] Undo of `add` removes the file, of `delete` restores it, of `update` restores the old text, of `dispute` and `deprecate` restores the fields (`tests/it_activity_and_undo.rs`)
- [ ] [auto] Undo of an undo (a redo) works, and the first entry shows `undone: false` again (`tests/it_activity_and_undo.rs`)
- [ ] [auto] The commit of another person gives `not_found` (`tests/it_activity_and_undo.rs`)
- [ ] [auto] `since` leaves out older entries (`tests/it_activity_and_undo.rs`)
- [ ] [auto] A plain Git commit without the `waymark:` subject does not show (`tests/it_activity_and_undo.rs`)
- [ ] [auto] Pending manual edits on the same file give `stale_base` with `next` "Run `sync` first." (`tests/it_activity_and_undo.rs`)
- [ ] [auto] After a refused undo, `git status` is clean and no commit was added (`tests/it_activity_and_undo.rs`)
- [ ] [auto] `later_changes` is `true` after another person changed the file (`tests/it_activity_and_undo.rs`)
- [ ] [auto] Undo of a `delete` outside `folders` gives `access_denied` (`tests/it_activity_and_undo.rs`)
- [ ] [auto] Entry model, grouping, and `undone` on a fixture history (unit tests in `src/activity/mod.rs`)

## Notes

Conflict cases use the second clone of the `World`: person B changes and pushes the same file, then person A calls `undo`. Move cases need the `move` tool, so `move-documents` must be implemented first or the move test creates the two commits by hand with the documented message format.
