# Changelog

What each release is about, newest first. The text of an entry is the text of the GitHub release. Write a short description for people: what is new, and what to do. Do not list commits. Process: [specs/foundation/distribution](specs/foundation/distribution/requirements.md), section "Release process".

## 0.1.5 (2026-11-14)

The settings file that `init` writes now explains every setting in a comment, and it no longer has a header that calls every key optional (`id` is required). When waymark adds the missing `schema` key to an older file, a header comment stays on top. Nothing to do: files you already have keep their text.

## 0.1.4 (2026-11-14)

waymark now tells you when an update is needed, even when your MCP entry has to change. An unknown or wrong argument in the entry no longer makes waymark exit. It starts and shows "WAYMARK NEEDS AN UPDATE" with the exact fix in three places: the text at the start of a session, the description of `status`, and the error of every tool. A knowledge base that needs a newer waymark is shown the same way. Update with `brew upgrade waymark` and restart your agent tool.

## 0.1.3 (2026-11-14)

The settings file `.mkbase/waymark.toml` has a version number, `schema = 1`. A knowledge base with a higher number stops an older waymark and says that a newer one is needed, so a team cannot damage a shared file by mixing versions. waymark also repairs a settings file that lacks the key, at your next write, in its own commit (`waymark: repair "settings"`). It adds only what is missing and never changes your limits.

## 0.1.2 (2026-11-14)

Links stay consistent. A write never creates a broken link to another document, and `delete` refuses while other documents link to the target and lists them, so you can fix the links first. `sync` applies the same rule to documents that you edited by hand. Three new tools manage folders: `folders` shows the structure, `add_empty_folder` creates an empty folder, and `remove_folder` removes a folder with its documents in one commit that `undo` can revert.

## 0.1.1 (2026-11-14)

`init` can now set up a knowledge base that was made for the older MKBase standard: it adds only `.mkbase/waymark.toml` and changes no document. `waymark --help` explains every option and shows examples. The standard now says that a value with a colon must be in quotes, because unquoted colons made documents unreadable.

## 0.1.0 (2026-11-14)

The first release. waymark is a local MCP server for macOS on Apple silicon, installed with Homebrew. A knowledge base is a Git repository of Markdown documents that your agent searches, reads, writes, moves, disputes, deprecates, and deletes, with `undo`, `sync`, `lint`, and limits for access, approval, and pause. Install with `brew install ionpulse-dev/tap/waymark`.
