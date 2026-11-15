# Changelog

What each release is about, newest first. The text of an entry is the text of the GitHub release. Write a short description for people: what is new, and what to do. Do not list commits. Process: [specs/foundation/distribution](specs/foundation/distribution/requirements.md), section "Release process".

## Unreleased

A personal server, as a first version.


- **`waymark --mode personal-server`** runs waymark as a remote MCP server for one person, over HTTPS (the host provides the certificate). Chat apps and MCP clients sign in once with the one allowed GitHub account. The server keeps no database and no stored state, so it also runs on a host that starts it for each request.
- It follows the MCP authorization standard (revision 2026-07-28): client documents, automatic registration, and one manual client ID and secret (`waymark --print-client`).
- Git access is a GitHub App installed on the one memory repository. The commit author is the person who signed in, with a verified email.
- A container image (`Dockerfile`) and a [setup guide](docs/personal-server-setup.md).
- **Not yet tested against the real apps** (Claude, ChatGPT, Gemini) and a real GitHub App. It is tested against a fake GitHub. The guide marks what is still to test.
- A write that the Git host rejects is now tried once more after a pull, in every mode.

## 0.5.1 (2026-11-15)

waymark now builds and runs on Linux, and this release fixes a serious bug on Linux. There is no change to the tools, so on a Mac you only need to update.

**Fixed (Linux):** when `git` did not answer in time, waymark stopped it with a `kill` call that the Linux `kill` can read as "kill every process of the user". The call is now safe. If you ran an earlier waymark build on Linux, update.

**What is new:**
- **Linux (x86_64 and arm64) works** for local mode, and all tests pass there. This release has a download for Linux x86_64 (`waymark-0.5.1-x86_64-unknown-linux-gnu.tar.gz`, with a checksum). The install steps are in the [install guide](docs/install.md). They are not yet tested on a real machine. For Linux arm64, build from source (see [development.md](docs/development.md)).
- The system settings file is `/etc/waymark/config.toml` on Linux. On macOS nothing changes.
- The update hint says `brew upgrade waymark` only when waymark was installed with Homebrew. The Git install hint fits your system.
- For developers: a Linux development container, and `make linux-test` and `make linux-check` run all work in Linux on a Mac.

What to do:
- Update with `brew upgrade waymark` and restart your agent tool.

## 0.5.0 (2026-11-15)

Fewer tools and smaller calls. You must act on this release if your agent tool, a script, or a limit still uses `list` or one of the nine old tool names of 0.4.0.

**What changed for you:**
- **`list` is now part of `search`.** A `search` with no `query` lists the documents that pass the filters, sorted by ID, at level 1. Like `search`, it shows only active documents unless you send `status: any` (`list` used to show all). A call named `list` still works until 0.6.0, with its old defaults and a notice. A limit `tools_denied = ["list"]` now switches off `search`.
- **The old names of 0.4.0 are gone,** as announced: `add`, `delete`, `deprecate`, `dispute`, `verify`, `reject`, `add_empty_folder`, `remove_folder`, and `folders` no longer work in calls. In limits (`tools_denied`, `approval.required_for`, `--tools-denied`) they still count as the new names, so a switched-off tool stays off.
- **`update_document` makes small changes with small calls.** `replace` changes one passage of the body (the passage must occur once), `append` adds text at the end, and `add_tags` and `remove_tags` change single tags. You no longer have to send the whole body or the whole tag list.
- **`show` tells you why a document was deprecated** (`deprecation_reason`).
- **`updates` reads long windows.** It takes `offset` and returns `next_offset`. The result has `until`: pass it as `since` in your next call.
- **Fixed:** several error hints named tools that do not exist (`delete`, `remove_folder`, `dispute`, `folders`, `list`). They name the current tools now, and a test keeps it so.
- The tool list is smaller: 23 tools in about 4 percent fewer bytes than in 0.4.0, and the shorter parameter texts say the same.

What to do:
- Update with `brew upgrade waymark` and restart your agent tool.
- Remove `mcp__waymark__list` from your allow rules in your agent tool. `mcp__waymark__search` covers it.
- Call `search` instead of `list`, and the new names instead of the old ones.

## 0.4.0 (2026-11-15)

Clearer tool names, four new tools, and better search. You must act on the first part if your agent tool has allow or ask rules for waymark tools.

**Nine tools have new names.** One rule now: a write on one document is `<verb>_document`, a folder tool is `<verb>_folder`.

| Old name | New name |
|---|---|
| `add` | `add_document` |
| `delete` | `delete_document` |
| `deprecate` | `deprecate_document` |
| `dispute` | `dispute_document` |
| `verify` | `verify_document` |
| `reject` | `reject_document` |
| `add_empty_folder` | `create_folder` |
| `remove_folder` | `delete_folder` |
| `folders` | `folder_tree` |

A call with an old name still works until 0.5.0, with a notice. An old name in a limit (`tools_denied`, `approval.required_for`, `--tools-denied`) counts as the new name, so a switched-off tool stays off. Your agent tool sees only the new names, so `mcp__waymark__add` in an allow list no longer matches.

**New tools and features:**
- `restore_document` brings a deprecated document back.
- `update_document` takes `supersedes`: say later that a document replaces an older one, which is then deprecated.
- `history` lists the versions of a document, and `show` with `version` reads an older one.
- `list_tags` shows the tags with counts, and `rename_tag` renames or merges a tag in all documents in one commit.
- `search` pages with `offset`. `list` and `search` filter by `author`, `updated_after`, and `updated_before`, and `search` takes `disputed`.
- Every tool description says what the tool returns, and every parameter has a description. `activity` accepts an RFC 3339 time in `since`, like `updates`.

What to do:
- Update with `brew upgrade waymark` and restart your agent tool.
- In your agent tool, replace the old tool names in your allow and ask rules with the new names. The list in the [install guide](docs/install.md) has all of them, with the new tools.

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
