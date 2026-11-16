# Plan: Clean Git status

## Overview

Change the two `init` templates, and extend the list of automatic repairs from the settings file to two more files at the repository root. The repair flow in `src/writes/mod.rs` already makes one commit before the change of the caller. It now loops over the files.

## Components Affected

- `src/memrepo/templates/gitignore` — add the comment line of Obsidian, and the macOS block with `.DS_Store`.
- `src/memrepo/templates/obsidian-app.json` — remove the line end after the last `}`.
- `src/memrepo/repairs.rs` — each repair names its file. Two new repairs: `gitignore-ds-store` and `obsidian-app-json-format`. The function `repair` takes the file and returns the repairs for that file only.
- `src/writes/mod.rs` — the repair step (now only for `settings.toml`) loops over three files: the settings file, `.gitignore`, and `.obsidian/app.json`. One commit per file that changed. It checks the rule for uncommitted changes (see below).
- `src/gitsync/message.rs` — no change. `commit_message(Verb::Repair, ".gitignore")` already gives `waymark: repair ".gitignore"`.
- `tests/it_memory_repository.rs` — integration tests for the new repairs.
- `tests/it_create_memory_repository.rs` — test A7 checks the new content, and a new check that `.DS_Store` and an Obsidian save leave `git status` empty.
- `.gitignore` of this code repository — add `.claude/worktrees/`, so a Claude Code worktree does not show as an untracked folder. This is development only, not product behavior.

Spec files that change with this feature:

- `specs/features/create-memory-repository/requirements.md` — the table "What `init` does": the new content of `.gitignore` and `.obsidian/app.json`, with a link to this spec.
- `specs/foundation/memory-repository/requirements.md` — the layout line of `.gitignore`, the row "Repair" (the settings file and the two files of `init`), and a list item for the new repairs that links to this spec.

## New Files

- `specs/features/clean-git-status/requirements.md` — this feature.
- `specs/features/clean-git-status/plan.md` — this plan.
- `specs/features/clean-git-status/tasks.md` — the work.
- `specs/features/clean-git-status/test.md` — the verification.
- `specs/features/clean-git-status/decisions.md` — the decisions.
- `specs/features/clean-git-status/todo.md` — what is open.

No new source file.

## Technical Approach

### Repairs with a file

```rust
pub enum Target { Settings, Gitignore, ObsidianApp }

pub struct Repair {
    pub name: &'static str,
    pub target: Target,
    pub notice: &'static str,
    pub apply: fn(&str) -> Option<String>,
}

pub fn repair(target: Target, text: &str) -> Option<(String, Vec<&'static Repair>)>
```

`apply` stays a pure text function (no Git, no file system), as the existing repair is. The unit tests stay in `repairs.rs`.

- `add_ds_store(text)`: `None` if a trimmed line is `.DS_Store` or `**/.DS_Store`. Otherwise the text, a line end if the text does not end with one, an empty line (if the text is not empty), the comment line, and `.DS_Store\n`.
- `obsidian_format(text)`: parse with `serde_json`. `None` if it does not parse, if it is not an object with exactly the two keys and values, or if the text already equals the template. Otherwise the template (the same constant that `init` uses).

### The repair step in a write

For each target, in the order settings, `.gitignore`, `.obsidian/app.json`:

1. The path below the repository folder: the settings file below the root of the memory repository (as now), the two others at the repository folder.
2. `committed` = the text at `HEAD` (`repo.show_at("HEAD", rel)`). `working` = `tx.read(rel)`. If the file is missing in the working tree, skip it (no file is ever created).
3. If `working` differs from `committed` (an uncommitted change): skip, except for `.obsidian/app.json` when `working` already equals the template and `obsidian_format(committed)` is not `None`. In that case commit the file as it is.
4. Else run `repair(target, working)`. If it gives a text, write it, commit it with `waymark: repair "<name>"`, and add the notices. The name is `settings` for the settings file (as now) and the path for the two others.

The settings file keeps its current behavior: step 3 is new for it too, but `tools never change this file` already means it has no uncommitted change in normal use. If the person edits the settings file by hand and has not committed it, waymark now leaves it alone instead of committing the person's edit with the repair. This is a small fix of the old behavior, recorded in `decisions.md`.

### Templates

`include_str!` keeps the bytes of the file. The file `obsidian-app.json` must have no line end at the end. An editor may add one, so a unit test checks that the constant does not end with `\n` and equals the exact text of the requirements. (`serde_json::to_string_pretty` cannot be the reference: without its feature `preserve_order`, it sorts the keys.)

## Cross-Feature Dependencies

- `create-memory-repository` (implemented) — owns the content of the files that `init` writes. Its table gets the new content.
- `memory-repository` (implemented) — owns the rules of the automatic repairs. The new repairs follow them without change.
- `manual-edits` and `sync` — no change. `.DS_Store` was "skipped" before. Now Git does not list it, so it is not in the result at all.

## Risks

- **Obsidian changes its format.** Then the file shows as changed again. The repair would not fix it, because the template would be wrong. The risk is small: Obsidian uses `JSON.stringify` with two spaces and has done so for years. If it happens, a new template and a new repair fix it.
- **A team member on an older waymark.** The old `init` is gone for them, and the older binary has no repair. Nothing breaks: the old files work as before.
- **This Mac has no Rust toolchain.** The code cannot be compiled here. The build and the tests run on a machine with Rust, or with `make linux-test` (see `docs/development.md`).

## Open Questions

- None.
