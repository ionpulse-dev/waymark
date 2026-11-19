# Tool reference

waymark gives agents its functions as MCP tools. This page lists every tool, what it does, and its parameters. It is for people who set up waymark, write rules in `AGENTS.md`, or set permission rules in an agent tool.

The agent itself does not need this page. It reads the description and the input schema of each tool from the server.

This page describes waymark 0.6.0.

## Index

- [Overview](#overview)
- [Common rules](#common-rules)
- [Read tools](#read-tools): [`status`](#status), [`search`](#search), [`show`](#show), [`history`](#history), [`stats`](#stats), [`folder_tree`](#folder_tree), [`list_tags`](#list_tags), [`updates`](#updates), [`lint`](#lint), [`activity`](#activity)
- [Write tools](#write-tools): [`add_document`](#add_document), [`update_document`](#update_document), [`dispute_document`](#dispute_document), [`deprecate_document`](#deprecate_document), [`restore_document`](#restore_document), [`move`](#move), [`rename_tag`](#rename_tag), [`create_folder`](#create_folder), [`undo`](#undo), [`sync`](#sync), [`init`](#init)
- [Destructive tools](#destructive-tools): [`delete_document`](#delete_document), [`delete_folder`](#delete_folder)
- [Verification tools](#verification-tools): [`review`](#review), [`verify_document`](#verify_document), [`reject_document`](#reject_document)
- [Results and errors](#results-and-errors)

## Overview

| Tool | Kind | What it does |
|---|---|---|
| `status` | read | Shows the memory repository in use, the limits, and checks with a fix for each problem |
| `search` | read | Finds documents by words, or lists them without a query |
| `show` | read | Returns one document, or an older version of it |
| `history` | read | Lists the versions of one document |
| `stats` | read | Counts words, tokens, characters, or documents |
| `folder_tree` | read | Shows the folders with the number of documents |
| `list_tags` | read | Lists the tags with the number of documents |
| `updates` | read | Lists what others changed since a time |
| `lint` | read | Checks documents for problems |
| `activity` | read | Lists your recent writes |
| `add_document` | write | Creates a document |
| `update_document` | write | Changes fields or the body of a document |
| `dispute_document` | write | Marks a document as wrong, or ends the dispute |
| `deprecate_document` | write | Retires a document and keeps the file |
| `restore_document` | write | Brings a deprecated document back |
| `move` | write | Moves or renames a document or a folder, and fixes links |
| `rename_tag` | write | Renames or merges a tag in all documents |
| `create_folder` | write | Creates an empty folder |
| `undo` | write | Reverts one of your writes |
| `sync` | write (listed as read) | Publishes manual edits, pulls, and pushes |
| `init` | write | Sets up an empty repository as a memory repository |
| `delete_document` | destructive | Deletes a document |
| `delete_folder` | destructive | Deletes a folder with all its documents |
| `review` | read | Lists unverified and disputed documents (verification only) |
| `verify_document` | write | Marks a document as verified (verification only) |
| `reject_document` | write | Rejects a document (verification only) |

There are 23 tools. With verification on, there are 26.

## Common rules

- **Document ID.** The ID of a document is its path below the root of the memory repository, without `.md`. Example: `ops/runbooks/deploy`.
- **Version and base.** `show` returns the `version` of a document: the short commit hash of the last change of its file. A write tool that changes an existing document needs this version as `base`. If the document changed since, the call fails with `stale_base`. Call `show` again and retry.
- **Preview.** Every write tool takes `preview: true`. That is a dry run: the tool checks the call and returns the result, but writes nothing.
- **Commit and push.** Every write makes a Git commit with the author of the person, and pushes it. The history stays.
- **Paging.** `search`, `list_tags`, and `updates` take `offset` and return `next_offset` when the result is `truncated`. `history` and `activity` take `before` and return `next_before`.
- **Dates.** A date is `YYYY-MM-DD`. Where a time is allowed, it is RFC 3339.
- **Kind hint.** Every tool tells the client its kind: read-only, write, or destructive. An agent tool can use this to ask the person before a write (see [Agent tools in the install guide](local-install.md#agent-tools)).
- **Hidden tools.** Limits can switch tools off. A switched-off tool is not listed. With `access = "read-only"` or paused writes, the write tools are not listed (`sync` stays: it then only pulls). `tools_denied` switches off single tools. No tool changes the limits.
- **Approval.** A memory repository can require that the person approves some tools before they run (`approval.required_for`). The person then sees a dialog in the agent tool or in the web console.

In the tables below, a parameter in **bold** is required.

## Read tools

Read tools change nothing. The one exception: `updates` stores the time of your last call.

### `status`

Shows the memory repository in use, the limits that apply, and checks with a fix for each problem. Call it first when something does not work.

Returns `healthy`, `version`, `memory_repository`, `checks`, `limit_sources`, and `author`.

No parameters.

### `search`

Searches the documents for words and returns the best matches, ranked, with a snippet. Without a query it lists the documents sorted by ID.

Returns `results` (`id`, `title`, `status`, `updated`, `description`, `snippet`), `total_matches`, `truncated`, and `next_offset`.

| Parameter | Type | Description |
|---|---|---|
| `query` | string | One or more words. A document must contain all of them. Leave it out to list documents. |
| `in` | array | Where to look, with a query: `path`, `title`, `description`, `tags`, `body`. Default: all of them. |
| `folders` | array | Only these folders. Default: all. |
| `tag` | string | Only documents with this tag. |
| `author` | string | Only documents whose `created_by` or `updated_by` contains this text, ignoring case. |
| `status` | `active`, `deprecated`, `any` | Which documents. Default `active`. |
| `disputed` | boolean | `true` returns only disputed documents. |
| `updated_after` | string | Only documents updated on this day or later. |
| `updated_before` | string | Only documents updated on this day or earlier. |
| `level` | 0, 1, 2 | Detail of each result: 0 ID only, 1 adds title, status, and updated, 2 adds description and snippet. Default 2 with a query, 1 without. |
| `limit` | integer | How many results. Default and maximum: the `max_results` limit. |
| `offset` | integer | How many results to skip. Default 0. |

The limits `max_results` and `max_tokens` cap the size of the result.

### `show`

Returns one document: properties, body, version, backlinks, `replaced_by`, and for a deprecated document `deprecation_reason`. With `version` it returns an older version.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| `version` | string | A version from `history`. Default: the current version. |

The limit `max_documents_opened` caps how many documents `show` opens in one session.

### `history`

Lists the versions of one document, newest first.

Returns `entries` (`version`, `time`, `author`, `verb`, `subject`) and `next_before`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| `limit` | integer | How many versions. Default 10, at most 50. |
| `before` | string | A version from an earlier result. Shows older versions. |

### `stats`

Counts words, tokens, characters, or documents, in total or by folder or author.

Returns `metric`, `total`, `documents`, and the groups.

| Parameter | Type | Description |
|---|---|---|
| `metric` | `words`, `tokens`, `chars`, `documents` | What to count. Default `words`. |
| `by` | `none`, `folder`, `author` | How to group the count. Default `none`. |
| `folders` | array | Only these folders. Default: all. |

### `folder_tree`

Shows the folders as a tree with the number of documents in each.

Returns `folders` (`path`, `documents`, `total`, `has_more`) and `truncated`.

| Parameter | Type | Description |
|---|---|---|
| `folder` | string | Start below this folder. Default: the root. |
| `depth` | integer | Levels below the start, 1 to 10. Default 2. |

### `list_tags`

Lists the tags with the number of documents for each.

Returns `tags` (`tag`, `documents`), `total_tags`, `truncated`, and `next_offset`.

| Parameter | Type | Description |
|---|---|---|
| `status` | `active`, `deprecated`, `any` | Which documents to count. Default `active`. |
| `folders` | array | Only these folders. Default: all. |
| `limit` | integer | How many tags. Default 50, at most 200. |
| `offset` | integer | How many results to skip. Default 0. |

### `updates`

Lists the documents that others added, changed, moved, deprecated, disputed, or deleted since a time. Without `since`, it uses the time of your last call. It changes nothing except that time.

Returns `since`, `until` (pass it as `since` next time), `changes` (`id`, `title`, `kind`, `by`, `date`), `total`, `truncated`, `next_offset`, and `marked`.

| Parameter | Type | Description |
|---|---|---|
| `since` | string | A date or an RFC 3339 time. Default: since your last call, or 7 days. |
| `folders` | array | Only these folders. Default: all. |
| `offset` | integer | How many changes to skip, to read the rest of a long window. Default 0. Needs `since`. |

### `lint`

Checks documents for format, size, links, secrets, personal data, duplicate titles, and authors.

Returns `checked`, `errors`, `warnings`, and `findings`.

| Parameter | Type | Description |
|---|---|---|
| `folders` | array | Only these folders. Default: all. |

### `activity`

Lists your recent writes, newest first, with the commit to give to `undo`.

Returns `entries` (`commit`, `time`, `verb`, `title`, `ids`) and `next_before`.

| Parameter | Type | Description |
|---|---|---|
| `since` | string | A date or an RFC 3339 time. Only writes from then on. |
| `limit` | integer | How many entries. Default 10, at most 50. |
| `before` | string | A commit from an earlier result. Shows older entries. |

## Write tools

Every write tool checks the result, commits, and pushes. Every write tool takes `preview` (see [Common rules](#common-rules)). It is not repeated in the tables.

### `add_document`

Creates a document. Sets the dates and the author, uses `name` or makes a file name from the title. It creates the folder if it does not exist.

Returns `id`, `title`, `deprecated`, `commit`, and `version`.

| Parameter | Type | Description |
|---|---|---|
| **`folder`** | string | Folder of the new document. Empty means the root. |
| **`title`** | string | One sentence. At most 15 words by default. |
| **`description`** | string | What the document contains and when to read it. At most 50 words by default. |
| **`body`** | string | Markdown text. May be empty. |
| `name` | string | File name without `.md`, such as `readme`: lowercase letters, digits, and `-`. Default: made from the title. |
| `tags` | array | Tags without `#` and without spaces. |
| `related` | array | IDs of documents to read too. |
| `sources` | array | Ticket IDs or URLs. |
| `anchors` | array | Pointers to what the document describes, as `git:<repo>/<path>@<commit>` or `url:<address>`. |
| `supersedes` | string | ID of the document that this one replaces. That document becomes deprecated. |

### `update_document`

Changes fields or the body of a document. Fields that are not sent stay as they are.

Returns `id`, `title`, `changed`, `commit`, and `version`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. Fails if the document changed. |
| `title` | string | The new title. |
| `description` | string | The new description. |
| `body` | string | The new Markdown text. It replaces the whole body. |
| `replace` | object | Changes one passage of the body: `old` (a passage of the current body that occurs exactly once) and `new` (the text instead, may be empty). |
| `append` | string | Text added at the end of the body. |
| `tags` | array | The new list of tags. It replaces the list. |
| `add_tags` | array | Tags to add. |
| `remove_tags` | array | Tags to remove. |
| `related` | array | The new list of related document IDs. It replaces the list. |
| `sources` | array | The new list of sources. It replaces the list. |
| `anchors` | array | The new list of anchors. It replaces the list. |
| `supersedes` | string | ID of a document that this one replaces. It is added to the list, and that document becomes deprecated. |

Send only one of `body`, `replace`, and `append`. Do not send `tags` together with `add_tags` or `remove_tags`.

### `dispute_document`

Marks a document as disputed with a reason. With `clear: true` it ends the dispute. A disputed document shows its reason in every result, so that readers know.

Returns `id`, `title`, `disputed`, `commit`, and `version`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. |
| `reason` | string | Why the document is wrong. One line. Required unless `clear` is `true`. |
| `clear` | boolean | `true` ends the dispute. |

### `deprecate_document`

Retires a document: sets its status to `deprecated` and keeps the file. The reason goes into the commit message. `search` skips deprecated documents by default.

Returns `id`, `title`, `status`, `commit`, and `version`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. |
| **`reason`** | string | Why the document is retired. One line. |

### `restore_document`

Brings a deprecated document back: sets its status to `active`.

Returns `id`, `title`, `status`, `commit`, and `version`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. |
| `reason` | string | Why the document is back. One line. |

### `move`

Moves or renames a document, or a whole folder (a path with a trailing slash). Rewrites every link to it and keeps the Git history. It makes one commit for the move and one for the link fixes.

Returns `from`, `to`, `moved`, and `links_updated`.

| Parameter | Type | Description |
|---|---|---|
| **`from`** | string | A document ID, or a folder path with a trailing slash. |
| **`to`** | string | The new ID, or the new folder path with a trailing slash. |

### `rename_tag`

Renames a tag in every document that has it, in one commit. When a document has the new tag already, the two tags merge.

Returns `from`, `to`, `documents`, `ids`, and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`from`** | string | The tag to change. |
| **`to`** | string | The new tag. No `#` and no spaces. |

### `create_folder`

Creates an empty folder. Use it only for a folder that has no document yet: `add_document` creates the folder of a new document by itself.

Returns `folder` and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`folder`** | string | The folder below the root, for example `ops/runbooks`. Lowercase letters, digits, and `-`. |

### `undo`

Reverts one of your writes with a new commit. The history stays. It refuses if later changes conflict.

Returns `reverted`, `title`, `changes`, and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`commit`** | string | A commit from `activity`. |

### `sync`

Publishes manual edits: documents that a person changed by hand, outside waymark. Each edit gets the checks of the matching write tool (`add_document`, `update_document`, `delete_document`, or `move`) and its own commit. Then it pulls and pushes.

With read-only access or paused writes, it only pulls. For this reason `sync` has the kind hint read-only, so that it stays listed. Note that an agent tool may then run it without a prompt.

Returns `committed`, `blocked`, `skipped`, `pulled`, `pushed`, and `conflicts`.

No parameters except `preview`.

### `init`

Sets up an empty repository as a memory repository. It also adds the settings file to a folder that has the older `.waymark` marker but no settings file.

Returns `id`, `name`, `slug`, `audience`, `files`, `commit`, and `adopted`.

| Parameter | Type | Description |
|---|---|---|
| **`name`** | string | Name of the memory repository. 1 to 80 characters. |
| `slug` | string | Short name: lowercase letters, digits, and `-`. Default: from the name. |
| `audience` | `personal`, `team` | Default `team`. |
| `folders` | array | Folders to create, such as `ops/runbooks`. At most 50. |
| `email_domains` | array | Domains whose email addresses may write, for example `example.org`. |
| `limits` | object | Shared limits for the settings file. See below. |
| `approval` | object | Approval rules for the settings file. See below. |

All keys of `limits` are optional:

| Key | Type | Description |
|---|---|---|
| `access` | `read-only`, `read-write` | `read-only` lets agents only read. |
| `paused` | boolean | `true` pauses all writes. |
| `folders` | array | Folders that agents may use. Default: all. |
| `tools_denied` | array | Tool names to switch off. |
| `max_results` | integer | Most documents that `search` returns. |
| `max_tokens` | integer | Most tokens that `search` returns. |
| `max_documents_opened` | integer | Most documents that `show` opens in one session. |

All keys of `approval` are optional:

| Key | Type | Description |
|---|---|---|
| `required_for` | array | Tool names, or `write`, that need approval before they run. |
| `via` | `elicitation`, `console` | How approval is asked. Default `elicitation`. |

## Destructive tools

These tools remove files. The files stay in the Git history. The person who deleted them can bring them back with `undo`.

### `delete_document`

Deletes a document. It refuses while another document links to it, and lists the linking documents.

Returns `id`, `title`, and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. |

### `delete_folder`

Deletes a folder with all documents in it, in one commit. It refuses while a document outside the folder links into it.

Returns `folder`, `documents`, `folders`, and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`folder`** | string | The folder below the root. |
| **`expect_documents`** | integer | How many documents the folder holds, also in the folders below it. 0 for an empty folder. A preview shows the number. |

## Verification tools

These tools exist only when the memory repository uses verification: `verification = true` in `.waymark/settings.toml`. With verification on, a new document is `unverified` until an owner verifies it.

### `review`

Lists active documents that are unverified or disputed, disputed first, oldest first. Changes nothing.

Returns `entries` (`id`, `title`, `trust`, `disputed`, `updated`, `version`) and `total`.

| Parameter | Type | Description |
|---|---|---|
| `folder` | string | Only documents in this folder. Default: all. |
| `limit` | integer | How many entries. Default 10, at most 50. |

### `verify_document`

Marks an active document as verified by you, with today's date.

Returns `id`, `title`, `trust`, `verified_by`, `verified`, and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. |

### `reject_document`

Rejects a document: sets its status to `deprecated`, clears its trust, and keeps the file. The reason goes into the commit message.

Returns `id`, `title`, `status`, and `commit`.

| Parameter | Type | Description |
|---|---|---|
| **`id`** | string | Document ID. |
| **`base`** | string | Version from `show`. |
| **`reason`** | string | Why the document is rejected. One line. |

## Results and errors

Every result has these fields, next to the data of the tool:

| Field | When | Content |
|---|---|---|
| `message` | always | A short sentence in plain English. |
| `notices` | when needed | Things the person should know, also unrelated to the call. Example: a document of the person was disputed. |
| `settings` | always | The limits for this call: access, pause, tools switched off, folders, caps, and approval rules. |

A failed call has `code`, `message`, and often `next`: one sentence on what to do.

| Code | Meaning |
|---|---|
| `validation_failed` | A format, size, link, duplicate, or secret check failed. |
| `sync_failed` | No network, no Git login, or the push was rejected. |
| `setup_problem` | The memory repository is not set up or not reachable. Call `status`. |
| `access_denied` | A limit blocks the call: access, folder, or a tool switched off. |
| `stale_base` | The document changed since the agent read it. Call `show` again. |
| `approval_denied` | The person said no, or no answer came in time. |
| `approval_unavailable` | Approval is required but cannot be asked. |
| `not_found` | The document or item does not exist. |
| `invalid_arguments` | An argument is missing or wrong. The `next` sentence names it. |
| `paused` | Writes are paused. |
