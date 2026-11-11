---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Search and read

## User Story

> As a person with an agent, I want my agent to find the few documents that answer a question quickly and cheaply, so that it has the right context and room left for my code.

## Personas

- **Developer** — the agent looks up decisions and contracts.
- **Every person** — asks "what do we know about X?".

## Behaviour

| Tool | Task |
|---|---|
| `search` | Full-text search over title, description, tags, and body by default. `in` limits the parts (`path`, `title`, `description`, `tags`, `body`). `level` 0, 1, or 2 sets how much each result shows. Filters: `folders`, `tag`, `status`. Results are ranked. |
| `show` | Returns one document with frontmatter, the author and last editor (name and email from Git), its age, the `base` version for `update_document`, and computed backlinks. A disputed document shows its `dispute_reason`. A deprecated document names the document that replaces it (the one whose `supersedes` names it). |
| `list` | Lists documents by filter, for example `status: deprecated` or `disputed: true`. |
| `stats` | The total size of the documents in some folders, with an optional breakdown by folder or author. Read-only. Metrics: `words` (default), `tokens` (an estimate), `chars`, `documents`. |

### What the server enforces

- **Result caps.** `search` returns at most `max_results` documents (default 5) and `max_tokens` tokens (default 800), or nothing if nothing matches. A call may ask for fewer, never for more than the cap.
- **Read budget.** `show` opens at most `max_documents_opened` documents per session (default 20). After that, `show` fails with `access_denied` and says that the budget is used up.
- **Folders.** The `folders` limit narrows every read. A document outside it is not found.
- **No folder means everything.** A tool that takes folders treats "no folder" as the root. Folders include subfolders.
- **Levels.** `0` shows the document ID (path and file name), `1` adds the title, `2` adds the description. `search` shows level 2 plus a short snippet by default. `list` shows level 1. The caps apply to listings too.
- **Pull before read.** If the last good pull is older than a few minutes, a read does a fast pull first.
- **No stored index.** Search reads the local files.
- **Facts in the result.** Every document shows `updated`, and `disputed` with its reason, so that the agent can pass them on.

The server does not decide when an agent reads. That is the choice of the team, in its `AGENTS.md`.

## Interfaces & Contracts

`show` returns a `base` hash that `update_document` accepts. Every result carries `message`, `settings`, and `notices` (see `mcp-server`).

## Acceptance Criteria

- **Given** a query with no match, **When** `search` runs, **Then** it returns nothing and no error.
- **Given** more matches than the cap, **When** `search` runs, **Then** it returns at most 5 documents within 800 tokens.
- **Given** a call that asks for 50 results, **When** `search` runs, **Then** it returns at most the cap.
- **Given** the read budget is used up, **When** `show` runs, **Then** it fails with `access_denied` and says why.
- **Given** `level` 0, **When** `list` runs, **Then** each line is only the ID.
- **Given** a document that others link to, **When** `show` runs, **Then** it lists the backlinks.
- **Given** a disputed document, **When** `show` runs, **Then** it shows the `dispute_reason`.
- **Given** a deprecated document that another document supersedes, **When** `show` runs, **Then** it names the replacement.
- **Given** the limit `folders = ["engineering/**"]`, **When** `search` runs, **Then** no document outside it appears.
- **Given** a folder, **When** `stats` runs by author, **Then** it counts per `created_by`.
- **Given** a clone that is a few minutes behind, **When** `search` runs, **Then** it pulls first.

## Constraints & Edge Cases

- Documents are local files, so search takes milliseconds. Up to some thousand files, plain search is enough. No vector database at the start.
- A document that was moved is found by its old ID: `show` with the old ID follows Git history and says where it went.
- The read budget is state of one session. It resets when the session ends.

## Open Questions

- Is ranking by plain text match good enough, or does a local index become necessary at some size?
- Is 20 opened documents per session a good default?
- How does the budget work on a shared server, where a session is an HTTP session?
