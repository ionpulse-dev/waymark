---
type: feature
status: implemented
created: 2026-11-14
updated: 2026-11-14
requires:
  - search-and-read
  - write-documents
---

# Folders

> **Scope:** local mode on macOS (Apple silicon) only, like the other specs (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a person, I want my agent to see how the knowledge base is organized and to create a folder before it has a document for it, so that it files each document in the right place.

## Personas

- **Agent** — looks at the folders before it writes, and puts a document in the right folder.
- **Knowledge base owner** — prepares the structure of a knowledge base before anyone writes documents.

## Behaviour

Three tools: `folders` (read), `add_empty_folder` (write), and `remove_folder` (write, destructive). Renaming or moving a folder is `move` (see `move-documents`). 
A folder has no description. Its name must say what it holds. A person can structure the knowledge base as they like: see the folders, create one (also an empty one), rename or move one with `move`, and remove one.

### What a folder is

A folder exists when it holds a document, a subfolder, or a file `.gitkeep`. A folder that only comes from `add` has no `.gitkeep`, so it is gone when its last document is deleted or moved. A folder that `add_empty_folder` or `init` made has a `.gitkeep` and stays until `remove_folder` removes it. A `.gitkeep` is an empty file. It is not a document: `lint`, `list`, `search`, `stats`, and the size limits ignore it. Folder names follow the rule of `init`: parts of lowercase letters, digits, and `-`, joined by `/`.

### `folders` (read)

Shows the folders of the knowledge base as a tree, with the number of documents of each folder.

| Input | Type | Required | Default |
|---|---|---|---|
| `folder` | string, a folder path to start below | no | the root |
| `depth` | integer, 1 to 10: how many levels below the start | no | 2 |

- `documents` of a folder counts the documents directly in it. `total` counts the documents in it and in all folders below it.
- A folder below the depth is not listed. Its parent shows `has_more: true` when folders below the depth exist.
- The parameter is named `folder`, not `path`, because `path` is the name of a repository argument that no tool may take (see `project-binding`).
- At most 200 folders are listed, in path order, a parent before its children. If more exist, `truncated: true` and the `next` sentence says to start below a folder, or to use a lower depth.
- The `folders` limit applies. A folder that no limit allows is not listed. A parent of an allowed folder is listed with its own counts for the allowed part only.
- Index files and `.gitkeep` files are not counted as documents.
- With an unknown start folder the call fails with `not_found`.

Result `data`:

```json
{
  "folder": "",
  "depth": 2,
  "folders": [
    { "path": "ops", "documents": 4, "total": 9, "has_more": true },
    { "path": "ops/deploy", "documents": 5, "total": 5, "has_more": false },
    { "path": "work", "documents": 2, "total": 2, "has_more": false }
  ],
  "truncated": false
}
```

### `add_empty_folder` (write)

Creates a folder that has no document yet. An agent needs it only for an empty folder. `add` creates the folder of a new document by itself.

| Input | Type | Required |
|---|---|---|
| `folder` | string, the folder path below the root. Nested parts are created too | yes |
| `preview` | boolean | no |

- The tool writes `<folder>/.gitkeep` (empty). It commits `waymark: add-folder "<folder>"` and pushes, under the author of the person. It follows the write flow of `git-sync`: limits, lock, fetch, approval, commit, push.
- If the folder exists (it holds a document, a subfolder, or a `.gitkeep`), the call fails with `validation_failed` and changes nothing.
- The path must follow the folder name rule above, and at most 8 levels deep. A bad path is `invalid_arguments` and names the field `folder`.
- The `folders` limit applies to the path (`access_denied`).
- `preview: true` writes nothing.
- `undo` can revert the commit.
- Result `data`: `{ "folder", "folders": [<the folder>], "commit", "pushed", "preview" }`.

### `remove_folder` (write, destructive)

Removes a folder with everything in it: all documents in it and below it, and all `.gitkeep` files. Everything goes in one commit.

| Input | Type | Required |
|---|---|---|
| `folder` | string, the folder path below the root | yes |
| `expect_documents` | integer, at least 0: how many documents the caller expects to delete (those in the folder and in all folders below it) | yes |
| `preview` | boolean | no |

- **Count guard.** If `expect_documents` differs from the real number, the call fails with `validation_failed` and changes nothing. The message gives the real number. An empty folder needs `0`. The normal flow is a call with `preview: true`, which shows the list and the number, and then the real call with that number.
- **Links.** If a document outside the folder links to a document inside it, the call fails with `validation_failed` and changes nothing. `data.linked_from` lists each linking document with `id`, `where` (`body` or `property`), `count`, and `to` (the document inside that it links to). `next`: "Remove or replace the links with update_document, then call remove_folder again." Links between documents inside the folder are no problem, because they go away together.
- **Limits.** The `folders` limit applies to every document inside. If a limit does not allow one, the call fails with `access_denied` and names the first document.
- **Other files.** If the folder holds a file that is not a document and not a `.gitkeep`, the call fails with `validation_failed` and names the file.
- **Protected.** The root and `.waymark` are not valid folder names (`invalid_arguments`). A path that is not a folder is `not_found`.
- **Approval.** The question shows the path, the number, and the list of IDs (at most 50, then "and N more").
- **Commit.** `waymark: remove-folder "<folder>"`. The body of the message lists the number of documents. The files stay in the Git history, and `undo` reverts the whole commit.
- **Preview.** `preview: true` runs all checks, writes nothing, and returns `data.documents` (the IDs) and `data.folders`.
- **Read-only access, `paused`, and `tools_denied`** work as for every write tool. A team that does not want bulk removal sets `tools_denied = ["remove_folder"]`.

Result `data`: `{ "folder", "documents": [<IDs>], "folders": [<paths>], "commit", "pushed", "preview" }`.

`move` of a folder carries its `.gitkeep`.

## Acceptance Criteria

- **Given** a knowledge base with nested folders, **When** `folders` runs with no input, **Then** it lists the folders down to depth 2 with `documents` and `total`.
- **Given** `depth: 1`, **When** `folders` runs, **Then** only the top level is listed, and a folder with subfolders has `has_more: true`.
- **Given** more than 200 folders, **When** `folders` runs, **Then** 200 are listed, `truncated` is true, and `next` names the way to narrow the call.
- **Given** the `folders` limit `["ops/**"]`, **When** `folders` runs, **Then** no folder outside `ops` is listed.
- **Given** a start folder that does not exist, **When** `folders` runs, **Then** it fails with `not_found`.
- **Given** a new folder path, **When** `add_empty_folder` runs, **Then** `<path>/.gitkeep` exists in one pushed commit, and `folders` lists the folder with 0 documents.
- **Given** that folder, **When** `add` puts a document in it, **Then** the document is listed, and `.gitkeep` is not counted.
- **Given** that folder, **When** its last document is deleted, **Then** the folder is still listed.
- **Given** an existing folder, **When** `add_empty_folder` runs, **Then** it fails with `validation_failed` and changes nothing.
- **Given** a path with an uppercase letter, `..`, or an absolute path, **When** `add_empty_folder` runs, **Then** it fails with `invalid_arguments`.
- **Given** `access = "read-only"`, **When** `add_empty_folder` runs, **Then** it fails with `access_denied`. **Given** `paused`, **Then** it fails with `paused`.
- **Given** `preview: true`, **When** `add_empty_folder` runs, **Then** nothing is written.
- **Given** a folder with a `.gitkeep`, **When** `lint`, `list`, `stats`, and `search` run, **Then** the file causes no finding and is not counted.
- **Given** a folder with documents and subfolders, **When** `remove_folder` runs with the right `expect_documents`, **Then** all documents and `.gitkeep` files are gone in one pushed commit, and `undo` brings them back.
- **Given** a wrong `expect_documents`, **When** `remove_folder` runs, **Then** it fails with `validation_failed`, the message has the real number, and nothing changes.
- **Given** an empty folder, **When** `remove_folder` runs with `expect_documents: 0`, **Then** the folder is gone and `folders` no longer lists it.
- **Given** a document outside the folder that links to one inside, **When** `remove_folder` runs, **Then** it fails with `validation_failed` and `linked_from` lists the linking document.
- **Given** documents that link only to each other inside the folder, **When** `remove_folder` runs, **Then** it succeeds.
- **Given** a document inside that the `folders` limit does not allow, **When** `remove_folder` runs, **Then** it fails with `access_denied` and nothing changes.
- **Given** the root or `.waymark`, **When** `remove_folder` runs, **Then** it fails with `invalid_arguments`.
- **Given** `preview: true`, **When** `remove_folder` runs, **Then** nothing is written and the result lists the documents.
- **Given** `tools_denied = ["remove_folder"]`, **When** the client lists tools, **Then** `remove_folder` is not listed and `add_empty_folder` is.
- **Given** `remove_folder`, **When** the client lists tools, **Then** it has `destructiveHint: true`, and `folders` has `readOnlyHint: true`.
- **Given** `add_empty_folder`, **When** the client lists tools, **Then** its description says that `add` creates the folder of a document by itself.

## Constraints & Edge Cases

- Git does not store an empty folder. That is why the folder holds a `.gitkeep`.
- Removing a folder is a bulk delete by one call. The guards are the count, the links rule, the approval question, `undo`, the Git history, and `tools_denied`.
- The three tool descriptions add to the context of every session. Each stays under 300 characters.
- A folder name that looks like a document ID of an existing document (`ops/deploy` and a document `ops/deploy.md`) is allowed by Git, but the tool refuses it: the call fails with `validation_failed`.

## Open Questions

None.
