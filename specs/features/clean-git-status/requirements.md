---
type: feature
status: implemented
created: 2026-11-16
updated: 2026-11-17
requires:
  - create-memory-repository
  - memory-repository
---

# Clean Git status

## User Story

> As a person who opens the memory repository on a Mac or in Obsidian, I want `git status` to show only my real changes, so that I do not have to find out where strange files come from.

## Personas

- **Private person** and **Team member without Git knowledge** — open the memory repository in Finder or Obsidian. They do not know why Git shows files that they did not change.
- **Memory owner** — wants a memory repository that stays clean without manual work.

## The problem

Two files show up in `git status` that the person did not change:

1. **`.DS_Store`.** macOS (Finder) writes this file in every folder that it opens. The `.gitignore` that `init` writes does not ignore it. So Git shows it as a new file. waymark never commits it, but `sync` lists it as "skipped" every time.
2. **`.obsidian/app.json`.** `init` writes the file with a line end after the last `}`. Obsidian writes the same settings without that line end. When Obsidian saves the file, Git shows a change, but the settings are the same.

## Behavior

### New memory repositories

`init` writes these two files with a new content. The other files do not change.

| File | Content |
|---|---|
| `.gitignore` | See below. |
| `.obsidian/app.json` | Exactly the bytes that Obsidian writes: two spaces of indent, no line end after the last `}`. See below. |

`.gitignore`:

```text
# Obsidian: share only the link settings. All other files are personal.
.obsidian/*
!.obsidian/app.json

# macOS: Finder writes these files in every folder that it opens.
.DS_Store
```

`.obsidian/app.json` (the file ends after the `}`, with no line end):

```json
{
  "useMarkdownLinks": true,
  "newLinkFormat": "relative"
}
```

### Existing memory repositories

Two new automatic repairs fix a memory repository that an older waymark made. They follow the rules of the automatic repairs in [memory-repository, section "Versions and updates"](../../foundation/memory-repository/requirements.md): they run at the first call that writes, never on a read or a preview, they ask no approval, and they come only from the list in the binary.

1. **`.gitignore`.** If the file exists at the repository root and no line in it is `.DS_Store` or `**/.DS_Store`, the repair adds the empty line, the comment line, and the line `.DS_Store` at the end. Every other line stays as it was.
2. **`.obsidian/app.json`.** If the file holds exactly the two waymark settings (`useMarkdownLinks: true`, `newLinkFormat: "relative"`, and no other key), but not in the bytes above, the repair writes the bytes above. A file with other keys or other values belongs to the person, and the repair leaves it alone.

Each repaired file gets its own commit before the change of the caller: `waymark: repair ".gitignore"` or `waymark: repair ".obsidian/app.json"`, under the author of the person, pushed with that change. `undo` can revert it. The result of the call has a notice:

- "Repaired .gitignore: Git now ignores the .DS_Store files of macOS."
- "Repaired .obsidian/app.json: it now has the format that Obsidian writes."

## Acceptance Criteria

- **Given** a new memory repository from `init`, **When** a person creates `.DS_Store` files in the root and in a folder, **Then** `git status --porcelain` lists nothing.
- **Given** a new memory repository from `init`, **When** Obsidian (or a test that writes the same bytes) saves `.obsidian/app.json` with the same settings, **Then** `git status --porcelain` lists nothing.
- **Given** a memory repository whose `.gitignore` has no `.DS_Store` line, **When** a write tool runs, **Then** the three lines are added at the end in a commit `waymark: repair ".gitignore"` before the commit of the write, every other line is unchanged, and the result has the notice.
- **Given** a `.gitignore` with a line `.DS_Store` or `**/.DS_Store`, **When** a write tool runs, **Then** no repair commit is made for it.
- **Given** a memory repository whose `.obsidian/app.json` is the old waymark file (with a line end at the end), **When** a write tool runs, **Then** the file gets the bytes above in a commit `waymark: repair ".obsidian/app.json"` and the result has the notice.
- **Given** an `.obsidian/app.json` with the old waymark content, and Obsidian already saved its own form in the working tree (the file shows as changed), **When** a write tool runs, **Then** the repair commits the file and `git status --porcelain` no longer lists it.
- **Given** an `.obsidian/app.json` with other keys or values, **When** a write tool runs, **Then** the file is not changed.
- **Given** a memory repository with no `.gitignore` or no `.obsidian/app.json` (for example one that `init` adopted), **When** a write tool runs, **Then** no file is created.
- **Given** a repair is needed, **When** a read tool runs or a write runs with `preview: true` or under read-only access, **Then** nothing is committed.
- **Given** a repair was made, **When** the same write runs again, **Then** no repair commit is made (a repair is safe to run twice).

## Constraints & Edge Cases

- A `.gitignore` or an `.obsidian/app.json` with other uncommitted changes is not repaired, so that waymark never commits work of the person. The one exception is an `app.json` whose working-tree text is already the bytes above (Obsidian saved it): the repair commits it.
- The repair adds `.DS_Store` only. Other files of other systems (for example `Thumbs.db` of Windows) are out of scope (see `todo.md`).
- An adopted memory repository (`.waymark` folder only) gets no new files. This follows the rule of `create-memory-repository`: adoption touches no other file.
- The Waymark Memory Standard does not change. Section 11 recommends the two Obsidian settings, not the bytes of the file, and it says nothing about `.gitignore`.

## Open Questions

- None.
