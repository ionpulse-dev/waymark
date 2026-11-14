# Changelog

What each release is about, newest first. The text of an entry is the text of the GitHub release. Write a short description for people: what is new, and what to do. Do not list commits. Process: [specs/foundation/distribution](specs/foundation/distribution/requirements.md), section "Release process".

## 0.3.1 (2026-11-15)

One fix. At the start of every session, the instructions said "No memory repository is set up: The first clone is still running.", also when the memory repository was already cloned. The server mixed up opening an existing clone with a real clone. Now the state `cloning` shows only while a real clone runs, so the instructions name your memory repository at once.

What to do: update with `brew upgrade waymark` and restart your agent tool.

## 0.3.0 (2026-11-14)

Two changes. You must act on the first one if your entry has no `--repo` or `--path`, or if you relied on the old clone folder.

**The memory repository lives where you say.** Your MCP entry must name it with `--repo` (the Git URL), `--path` (a folder), or both. `--repo` alone clones into `~/.waymark/<name>`, where `<name>` is the repository name from the URL. `--path` sets the folder, and a missing or empty folder is cloned into. The old default `~/Projects/<name>-agent-memory`, the scan of `~/Projects`, and the search of the working folder are gone, and so is the entry without arguments. A folder that holds another remote is an error that names the fix.

**The `index` tool is gone,** and so is the `index-stale` check of `lint`. `folders`, `list`, and `search` already give an agent the overview that index files gave. A file `index.md` is now a normal document: `search`, `list`, and `updates` show it, and `delete` refuses while it links to the target.

What to do:
- If you used the old default folder, add `--path ~/Projects/<name>-agent-memory` to your entry, or move the clone to `~/.waymark/<name>`. If your entry had no arguments, add `--repo` or `--path`.
- If a memory repository holds generated `index.md` files, they stay as documents. Delete them if you do not want them, and remove `mcp__waymark__index` from the allow list of your agent tool.
- Update with `brew upgrade waymark` and restart your agent tool.

## 0.2.0 (2026-11-14)

waymark is now long-term agent memory, and the names follow. A Git repository that holds the memory is a memory repository, no longer a "knowledge base". This release renames what you can see, with no fallback for the old names, so it needs a few steps from you:

- In each memory repository, rename the folder `.mkbase` to `.waymark` and the file `waymark.toml` in it to `settings.toml`.
- In your MCP entry, rename the argument `--kb-id` to `--memory-repository-id`.
- Agents that read the `status` result now find the key `memory_repository`, and the state `not_a_memory_repository`.
- `init` now creates the content folder `agent-memory/` (it was `knowledge/`), and the default clone folder is `~/Projects/<name>-agent-memory`.
- The plain-Git skills are now `personal-agent-memory` and `team-agent-memory`. The format is now called the Waymark Memory Standard.

Update with `brew upgrade waymark` and restart your agent tool.

## 0.1.5 (2026-11-14)

The settings file that `init` writes now explains every setting in a comment, and it no longer has a header that calls every key optional (`id` is required). When waymark adds the missing `schema` key to an older file, a header comment stays on top. Nothing to do: files you already have keep their text.

## 0.1.4 (2026-11-14)

waymark now tells you when an update is needed, even when your MCP entry has to change. An unknown or wrong argument in the entry no longer makes waymark exit. It starts and shows "WAYMARK NEEDS AN UPDATE" with the exact fix in three places: the text at the start of a session, the description of `status`, and the error of every tool. A knowledge base that needs a newer waymark is shown the same way. Update with `brew upgrade waymark` and restart your agent tool.

## 0.1.3 (2026-11-14)

The settings file `.waymark/settings.toml` has a version number, `schema = 1`. A knowledge base with a higher number stops an older waymark and says that a newer one is needed, so a team cannot damage a shared file by mixing versions. waymark also repairs a settings file that lacks the key, at your next write, in its own commit (`waymark: repair "settings"`). It adds only what is missing and never changes your limits.

## 0.1.2 (2026-11-14)

Links stay consistent. A write never creates a broken link to another document, and `delete` refuses while other documents link to the target and lists them, so you can fix the links first. `sync` applies the same rule to documents that you edited by hand. Three new tools manage folders: `folders` shows the structure, `add_empty_folder` creates an empty folder, and `remove_folder` removes a folder with its documents in one commit that `undo` can revert.

## 0.1.1 (2026-11-14)

`init` can now set up a knowledge base that was made for the older MKBase standard: it adds only `.waymark/settings.toml` and changes no document. `waymark --help` explains every option and shows examples. The standard now says that a value with a colon must be in quotes, because unquoted colons made documents unreadable.

## 0.1.0 (2026-11-14)

The first release. waymark is a local MCP server for macOS on Apple silicon, installed with Homebrew. A knowledge base is a Git repository of Markdown documents that your agent searches, reads, writes, moves, disputes, deprecates, and deletes, with `undo`, `sync`, `lint`, and limits for access, approval, and pause. Install with `brew install ionpulse-dev/tap/waymark`.
