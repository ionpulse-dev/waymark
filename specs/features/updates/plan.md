# Plan: Updates

## Overview

A function `changes_since` compares the tree at the start of the window with `HEAD` and classifies each changed document. The `updates` tool calls it, filters, and stores the time of the call in a small state file. `notifications-and-review` reuses both the function and the state file.

## Components Affected

- `store` — `Store` loads the title and frontmatter at `HEAD`. The new `changes` file lives here.
- `gitsync` — `refresh_for_read`, and a new `diff` file that runs `git diff` and `git log` for the window.
- `config` — the `folders` limit.
- `mcp` — the registry gets `updates`.

## New Files

- `src/tools/updates.rs` — the `updates` tool.
- `src/store/changes.rs` — `changes_since(repo, root, since) -> Vec<Change>` and the classification.
- `src/gitsync/diff.rs` — `base_commit_before(ts)`, `changed_paths(base, root)` (`git diff --name-status -M`), and `last_change_by_others(path, base, own_email)`.
- `src/state/mod.rs` — `State`: a JSON file for each person and memory repository. Read, update one key, write atomically (temp file and rename).
- `tests/it_updates.rs` — end-to-end tests with two clones.

## Technical Approach

1. **Window.** Parse `since` (`time` crate). Without it, read `last_updates` from the state file, or use now minus 7 days. A date alone means 00:00 UTC. A `since` in the future is `invalid_arguments`.
2. **Base commit.** `git rev-list -1 --before=<since> HEAD` gives the base. If there is none, use the empty tree `4b825dc642cb6eb9a060e54bf8d69288fbee4904`.
3. **Changed paths.** `git diff --name-status -M <base> HEAD -- <root>` lists `A`, `M`, `D`, and `R` entries. Keep `.md` files that are not dot paths, and not excluded. Map each path to an ID.
4. **Classify.** `A` is `new`. `D` is `deleted` (the title comes from the base version, read with `git show`). `R` is `moved` with `from`. `M` compares the frontmatter at the base and at `HEAD`: `disputed` false to true is `disputed` (with `dispute_reason`), `status` active to deprecated is `deprecated`, else `changed`. A `R` entry with a changed `disputed` flag stays `moved`.
5. **Author of the change.** For each path, `git log --format=%an <%ae>|%as <base>..HEAD -- <path>` gives the commits in the window. Drop the path if all commits have the email of the person. Else `by` and `date` come from the newest commit by someone else.
6. **Filter.** Apply the `Scope` of `store::filter`: the `folders` limit and the `folders` argument. A moved document passes if its old or new ID passes.
7. **Mine.** `created_by` email (lowercase) equals the author email from `gitsync`.
8. **Order and cap.** Sort as in the requirements. Cut at 50.
9. **Mark.** Without `since`, write `last_updates` with the time of this call to the state file. A write failure is a notice, not an error.
10. **State file.** The folder is `$WAYMARK_STATE_DIR`, else `$XDG_STATE_HOME/waymark`, else `~/.local/state/waymark`. The file is `<memory repository id>.json`. Format: `{ "last_updates": "2026-11-14T09:30:00Z" }`. Other keys stay as they are (see `notifications-and-review`).
11. **Message.** "`N` changes since `<date>`." plus one sentence per disputed document of the person, plus "waymark did not check these changes for truth."

## Cross-Feature Dependencies

- `search-and-read` — `Store`, `Scope`, and `time`. Resolved: listed in `requires`.
- `git-sync` — `gitsync::diff` is new and is added here.
- `notifications-and-review` — uses `changes_since` and `State`. It lists `updates` in `requires`.

## Risks

- A `git diff` between base and `HEAD` hides a change that was made and reverted. Accepted: the result is the net change.
- Clock skew between machines makes `--before` shift by minutes. Accepted.
- A rename with heavy edits drops under the similarity limit and shows as `deleted` and `new`. Accepted. `show` still follows moves that Git detects.

## Open Questions

None.
