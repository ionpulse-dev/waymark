# Decisions: Folders

## A new read tool, `folders`, and not a mode of `stats` — 2026-11-14

**Decision (product owner):** The tree of folders is a tool of its own, `folders`, with a `depth` parameter. It is not a mode of `stats`.

**Alternatives considered:** A mode `by: tree` of `stats` (one tool less in the context, but `stats` is for counting). The top-level folders in the session-start text (rejected: a knowledge base with 1,000 top-level folders would cost thousands of tokens in every session, and the text is stale after the first new folder).

**Reason:** A clear name makes an agent find the tool. The cost is one more tool description.

## Folders have no description — 2026-11-14

**Decision (product owner):** A folder has no description field. The name must be descriptive enough.

**Reason:** One more thing to write and to keep current for little gain.

## The tool for an empty folder is `add_empty_folder` — 2026-11-14

**Decision (product owner):** The write tool is named `add_empty_folder`, with the input `path`. It writes a `.gitkeep`.

**Alternatives considered:** `add_folder` or `create_folder` (an agent might call them before every `add`, which wastes a call and a commit). A parameter on `folders` (a read tool must not write, and the limits cannot then tell a read from a write).

**Reason:** The name says when to use it. `add` already creates the folder of a document.

## A folder exists with a document, a subfolder, or a `.gitkeep` — 2026-11-14

**Decision (decided autonomously):** A folder that was made on purpose stays after its last document is gone. A folder that only came from `add` does not. There is no tool to remove an empty folder yet.

**Reason:** Git stores no empty folders, and the person who creates a folder wants it to stay. Nobody asked for the folders that appear only because of a document.
