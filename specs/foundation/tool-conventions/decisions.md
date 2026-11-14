# Decisions: Tool conventions

## One scheme: `<verb>_document`, `<verb>_folder`, bare verbs for the rest — 2026-11-15

**Decision (product owner, decided in review):** Write tools on one document are `<verb>_document`. Folder tools are `<verb>_folder`, and the tree is `folder_tree`. Read tools and tools for the whole memory repository keep a bare word.

**Alternatives considered:** Suffix `_document` on every tool, also the read tools (longer names in every call, no gain: the read tools are already clear). Keep the old names (an agent cannot guess `add_empty_folder` from `add`).

**Reason:** The name says what the tool acts on. A new tool has an obvious name.

## `move` keeps its name — 2026-11-15

**Decision (decided autonomously):** `move` is not renamed to `move_document`.

**Reason:** It moves a document or a whole folder. The name `move_document` would be wrong for a folder. The scheme gives a bare verb to a tool that acts on documents and folders.

## `verify` and `reject` become `verify_document` and `reject_document`; `review` stays — 2026-11-15

**Decision (decided autonomously):** The verification tools follow the scheme. `review` is a read tool and keeps its name.

**Reason:** The same rule as for the other tools.

## Old names work in calls until 0.5.0, and in limits for good — 2026-11-15

**Decision (decided autonomously):** A call with an old name still runs, with a notice. An old name in a limit counts as the new name, with no end date.

**Alternatives considered:** No alias, as in version 0.2.0 (the project broke names then without a fallback). Here that would be unsafe: a limit `tools_denied = ["delete"]` would stop working, and `delete_document` would be available again with no error. Failing on an unknown name in a limit (a larger change in the trust model, and it would block the server for a name that a newer or older waymark knows).

**Reason:** A limit must never loosen by a rename. The alias in limits costs one table. The alias in calls helps agents and allow lists during the move, and it ends in 0.5.0.

## Commit verbs do not change — 2026-11-15

**Decision (decided autonomously):** The commit subjects keep their verbs (`waymark: add`, `waymark: delete`, `waymark: add-folder`, ...). New verbs: `restore` and `rename-tag`.

**Reason:** Old commits stay readable by `activity` and `undo`. A verb in a subject is a record, not a tool name.

## `activity` and `history` page with `before`, the other lists with `offset` — 2026-11-15

**Decision (decided autonomously):** Lists of documents (`list`, `search`, `list_tags`) page with `offset`. Logs (`activity`, `history`) page with a commit.

**Alternatives considered:** `offset` for logs too (a new commit shifts the entries, so a page can repeat or skip an entry). A commit cursor for lists (a list has no natural commit).

**Reason:** Each cursor fits what is listed. The rule is written once, so that it is not a surprise.

## The budget of the tool list is raised — 2026-11-15

**Decision (decided autonomously):** The limits are 28000 bytes for the list and 9000 characters for the descriptions (before: 20000 and 6000).

**Reason:** Four new tools, a description for every parameter, and a "Returns" sentence in every description. Each of these removes guesses of the agent. The list is still about 7000 tokens.
