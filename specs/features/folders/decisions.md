# Decisions: Folders

## A new read tool, `folders`, and not a mode of `stats` — 2026-11-14

**Decision (product owner):** The tree of folders is a tool of its own, `folders`, with a `depth` parameter. It is not a mode of `stats`.

**Alternatives considered:** A mode `by: tree` of `stats` (one tool less in the context, but `stats` is for counting). The top-level folders in the session-start text (rejected: a knowledge base with 1,000 top-level folders would cost thousands of tokens in every session, and the text is stale after the first new folder).

**Reason:** A clear name makes an agent find the tool. The cost is one more tool description.

## Folders have no description — 2026-11-14

**Decision (product owner):** A folder has no description field. The name must be descriptive enough.

**Reason:** One more thing to write and to keep current for little gain.

## Three tools: `folders`, `add_empty_folder`, `remove_folder` — 2026-11-14

**Decision (product owner):** `folders` reads. `add_empty_folder` creates an empty folder and `remove_folder` removes a folder with its content. Renaming and moving a folder stay in `move`. A person can structure the knowledge base as they like.

**Alternatives considered:** One tool for reading and writing (rejected: it breaks the `allow` list of the agent tool, `readOnlyHint`, and `access = "read-only"`). One write tool with the actions `add_empty` and `remove_empty` (rejected after the product owner asked for removal with content: a bulk delete needs `destructiveHint` and its own entry in `tools_denied` and `approval.required_for`, and creating a folder must not carry them). A parameter on `folders` (a read tool must not write).

**Reason:** The two write tools have different risk, like `add` and `delete`. The name `add_empty_folder` says when to use it: `add` creates the folder of a new document by itself. This replaces an earlier idea of removing only empty folders.

## `remove_folder` needs the document count — 2026-11-14

**Decision (decided autonomously):** The caller must pass `expect_documents`. A wrong number fails with the real number. The call refuses while a document outside the folder links into it, like `delete`.

**Alternatives considered:** A `force` flag (an agent sets it without thought). No guard (one wrong path deletes a whole area). Only empty folders (the product owner wants whole folders removed).

**Reason:** The agent must look at the number first, which is what `preview` shows. A wrong path or a folder that grew since the preview is caught. Everything is one commit, so `undo` is one step.

## A folder exists with a document, a subfolder, or a `.gitkeep` — 2026-11-14

**Decision (decided autonomously):** A folder that was made on purpose stays after its last document is gone. A folder that only came from `add` does not. There is no tool to remove an empty folder yet.

**Reason:** Git stores no empty folders, and the person who creates a folder wants it to stay. Nobody asked for the folders that appear only because of a document.
