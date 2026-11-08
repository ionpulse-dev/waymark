---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
  - git-sync
---

# Search and read

## User Story

> As a person with an agent, I want my agent to find the few documents that answer a question quickly and cheaply, so that it has the right context and room left for my code.

## Personas

- **Developer** — the agent looks up decisions and contracts.
- **Every person** — asks "what do we know about X?".

## Behaviour

| Command | Task |
|---|---|
| `waymark search <query> [<folder>...]` | Full-text search over title, description, tags, and body by default. `--in path,title,description,tags,body` limits the parts, and is the way to search the path. `--level 0\|1\|2` sets how much each result shows. Filters: `--tag`, `--status`. Results are ranked. |
| `waymark show <id-or-path>` | Prints one document with frontmatter, author and last editor (name and email from Git), the version hash for `--base`, computed backlinks, and a stale mark if `updated` is older than `stale_after_days`. |
| `waymark list [<folder>...] [--level 0\|1\|2]` | Lists documents by filter, for example `--stale`. |
| `waymark stats [<folder>...] [--metric words\|tokens\|chars\|documents] [--by folder\|author]` | Total size of the documents in some folders, with an optional breakdown. Read-only. Default metric: `words`. Tokens are an estimate; other metrics are exact. |

Rules:

- **No folder means everything.** Every command that takes folders treats "no folder" as the root. Folders include subfolders. There is no list of followed folders.
- **Levels:** `0` shows the document ID (path and file name), `1` adds the title, `2` adds the description. `search` shows level 2 plus a short snippet by default. `list` shows level 1. The read budget applies to listings too.
- **Read budget:** `search` returns at most `search.max_results` documents (default 5) and `search.max_tokens` tokens (default 800), or nothing if nothing matches. `--limit` overrides it. Silence is better than noise.
- **Pull before read:** if the last good sync is older than a few minutes, a read does a fast pull first.
- **No stored index.** Search reads the local files.
- The agent sees a document in steps, each more expensive: path, file name, title, description, body.
- At read level `manual`, the skill tells the agent to read only when asked.

## Interfaces & Contracts

Every command takes `--kb <uuid>` and `--json`. `show` prints a `base` hash that write commands accept as `--base`.

## Acceptance Criteria

- **Given** a query with no match, **When** `search` runs, **Then** it returns nothing and exits 0.
- **Given** more matches than the budget, **When** `search` runs, **Then** it returns at most 5 documents within 800 tokens.
- **Given** `--level 0`, **When** `list` runs, **Then** each line is only the ID.
- **Given** a document that others link to, **When** `show` runs, **Then** it lists the backlinks.
- **Given** a document older than `stale_after_days`, **When** `show` runs, **Then** it is marked stale.
- **Given** a folder, **When** `stats --by author` runs, **Then** it counts per `created_by`.
- **Given** a clone that is a few minutes behind, **When** `search` runs, **Then** it pulls first.

## Constraints & Edge Cases

- Documents are local files, so search takes milliseconds. Up to some thousand files, plain search is enough. No vector database at the start.
- A document that was moved is found by its old ID: `show <old-id>` follows Git history and says where it went.

## Open Questions

- Is a ranking by plain text match good enough, or does a local index become necessary at some size?
