# Plan: Document names

## Overview

Three code changes and text changes. `add_document` gets `name`. The manual edit module stops returning one commit and returns one planned publish for each edit, with the rules of the matching write tool. A small helper rewrites short wikilinks when a write makes a second document with the same file name. The description of `sync` says what it is for, and the `AGENTS.md` suggestions get the `sync` rule.

## Components Affected

- `standard/waymark-1.0.md` — section 8 gets the rule for short wikilinks.
- `src/tools/add.rs` — the field `name`, and the short-link fixes in the same commit.
- `src/writes/slug.rs` — `check_name` for a name from the caller.
- `src/writes/linkfix.rs` — `Remap::wiki_target` also rewrites a short link to a document that does not move, when its file name stops being unique. `Remap::added` builds a remap for new IDs.
- `src/tools/move_documents.rs` — no change of code: the fixed `Remap` covers the new case.
- `src/manual/mod.rs` — `prepare` sorts each edit into `add`, `update`, `delete`, or `move`, runs `lint` and the extra rules of that tool, applies the verification and author fields, and plans one commit (two for a move) for each edit. `commit_before_write` uses the same plan and keeps back the edits that need approval.
- `src/manual/pending.rs` — the kind `Moved`, found from a deleted and a new file with the same text.
- `src/gitsync/sync.rs` — `Repo::sync` takes a hook that commits itself and returns the commit hashes.
- `src/writes/mod.rs` — `execute` passes `env` to `commit_before_write`.
- `src/gitsync/write.rs` — `Tx::forget`: after `commit_before_write` committed the published edits, a later failure of the agent's write does not put their files back.
- `src/gitsync/sync.rs` — `resolve_conflicts` finds the conflicts of all local commits with a trial merge, because a rebase stops at the first commit that conflicts.
- `src/activity/mod.rs` — a move that `sync` published (verb `sync` on both commits) is one entry, so `undo` takes back both.
- `src/tools/sync.rs` — the new description, the result with one `commit` for each entry, and the approval for the matching tools.
- `docs/local-install.md`, `CHANGELOG.md` — texts. `README.md` needs no change: it names `sync` and no file-name rule.
- Specs: `document-format`, `write-documents`, `manual-edits`, `move-documents`, `quality-checks` (the new finding rules of `sync`).

## New Files

- `specs/features/document-names/` — this spec (`requirements.md`, `plan.md`, `tasks.md`, `test.md`, `decisions.md`, `todo.md`).
- `src/writes/shortlinks.rs` — `add_fixes_for_new_ids`: the link fixes in other documents when new IDs appear, shared by `add_document` and `sync`.
- `tests/it_document_names.rs` — the integration tests of this spec.

## Technical Approach

1. **Name.** `slug::check_name(name)` returns the name, unchanged, or `invalid_arguments`. The rules are those of a file name in `format::id::check_path`, plus no `/` and at most 60 characters. `add_document` uses `<folder>/<name>` when `name` is present. A taken ID (compared in lowercase) is `validation_failed` with the ID in the message.
2. **Short links.** `Remap::wiki_target(name)` today returns a new path only for a moved target. The fix: for a short name that found exactly one old document, compute the new ID of that document (moved or not). If the new ID has the same file name and that file name is unique among the new IDs, the link stays. Otherwise the link gets the full new path. `Remap::added(old_ids, new_ids)` is a remap with no moves and extra IDs. `shortlinks::add_fixes_for_new_ids` walks all documents except the ones with pending edits and the new documents, calls `retarget_body` with the same ID before and after, and returns `FileChange::Write` entries (with `updated` and `updated_by` set) and the `links_updated` list. A linking document outside the `folders` limit is `access_denied`.
3. **Pending edits.** `pending::find` keeps finding new, changed, and deleted paths. A new pass pairs a deleted path with a new path when the text of the new file equals the text of the deleted path at `HEAD`. The pair becomes one edit of kind `Moved` with `from`.
4. **Rules per edit.** `manual::prepare(tx, memrepo, env, author, today)`:
   - Runs `lint` once on the new and changed IDs (as today), and the delete link check (as today).
   - For each edit in ID order, adds the rules of its tool: `env.effective.check_tool(tool, Kind::Read)` (only `tools_denied`, see `decisions.md`), `env.effective.allows_folder(id)` (and `from` for a move), and for new and changed files `titles::clashes` against the active titles, skipping the document itself and pending edits that did not pass yet.
   - For a passing new or changed file: sets `updated` and `updated_by`, and with verification on applies the trust rule of the tool.
   - For a passing move: computes the link fixes with `Remap::new` over all IDs, skipping documents with pending edits.
   - For a passing new file or move: adds the short-link fixes.
   - Returns `Planned` items: the edit, the files to write, the title, and the tool name. Nothing is committed here.
5. **Commits.** `manual::commit_planned(tx, planned)` writes the files of each item and commits it: one commit with the subject `waymark: sync "<title>"` and the line `Action: <verb>`. A move commits the rename first, then the link fixes.
6. **Approval.** `sync` collects the tools of the passing edits. If `sync` or any of them needs approval, it asks once through `writes::approve` with the first tool that needs it and the list of paths. `commit_before_write` moves the passing edits whose tool needs approval to a held list. They stay uncommitted, and a notice says "Run `sync`".
7. **Repo::sync.** The hook gets the transaction and returns `Vec<String>` of commit hashes. The pull and push stay as they are.
8. **Texts.** The new description of `sync` (facts only), and one line in both `AGENTS.md` suggestions.

## Dependencies on Other Specs

- `write-documents` — the rules of the write tools and `titles::clashes`.
- `manual-edits` — the pending edits and the flow of `sync`.
- `move-documents` — `Remap` and the two commits of a move.
- `verification` — the trust rule of `add_document` and `update_document`.
- `write-approval` — `writes::approve`.

## Risks

- **More commits.** `sync` with many edits makes many commits. Accepted: each edit can be undone on its own, and a sync of many hand edits is rare.
- **Slower writes.** Every write runs the rules on pending edits first. Most of the time there are none.
- **Rename detection.** Only a rename with the same text is a move. Obsidian keeps the text of a renamed file, so this is the common case.

## Open Questions

None.
