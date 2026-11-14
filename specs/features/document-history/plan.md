# Plan: Document history

## Overview

One new Git helper reads the history of a file with `--follow`. `history` formats it. `show` uses it to check a version and to read the text at that version.

## Components Affected

- `src/gitsync/log.rs`: `Repo::file_history(rel, max, before)`.
- `src/tools/show.rs`: the `version` parameter.
- `src/tools/mod.rs`: registers `history`.

## New Files

- `src/tools/history.rs`: the `history` tool.
- `tests/it_document_history.rs`: the cases of `test.md`.

## Technical Approach

1. `Repo::file_history` runs `git log --follow --no-merges --name-only --format=<marker>%H<sep>%h<sep>%an<sep>%ae<sep>%aI<sep>%s -- <path>`. Each commit gets the path of the file at that commit. It returns `(LogEntry, path_at_commit)` newest first. `before` cuts the list after the given commit.
2. `history` finds the document (as `show` does, with the limit scope), asks for `limit + 1` entries, and fills `verb` with `message::parse_subject`.
3. `show` with `version` calls `file_history` for the document, finds the entry whose hash starts with the version, reads the text with `show_at(hash, path_at_commit)`, and parses it. The current version is `last_commit`.
4. The read budget counts the document ID once, as before.

## Cross-Feature Dependencies

- `search-and-read`: `show` and its read budget.
- `activity-and-undo`: the subject parser.

## Risks

- `--follow` works for one path only, and Git guesses renames. The move tool already checks that Git follows a move (`history_reaches`), so the normal case is safe.

## Open Questions

None.
