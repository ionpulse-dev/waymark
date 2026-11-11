---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Create a knowledge base

## User Story

> As the first person of a company, a team, or just myself, I want to create a knowledge base in a few minutes, so that others and my agents can use it.

## Personas

- **Knowledge base owner** — creates the knowledge base and sets the shared settings.
- **Private person** — creates a private knowledge base for their own AI memory.

## Behaviour

1. The person creates an empty, private repository on the Git host. waymark does not create repositories.
2. The person adds an MCP entry with `--repo <url>` (see `join-knowledge-base`) and starts a session. The server clones the empty repository.
3. `status` reports that the repository is empty and that the `init` tool can set it up.
4. The `init` tool creates the structure, commits, and pushes.

`init` takes these inputs. Every input has a default or is optional:

| Input | Stored in | Default |
|---|---|---|
| `name` of the company or project | `.mkbase/waymark.toml` | none, required |
| `slug` | `.mkbase/waymark.toml` | derived from the name |
| `email_domains` (optional) | `.mkbase/waymark.toml` | none |
| `limits` and `approval` (optional shared limits, see `trust-model`) | `.mkbase/waymark.toml` | none |
| `folders` (first folders) | `knowledge/` | no folders |
| `audience`: `personal` or `team` | which standalone rules go into `AGENTS.md` | derived from the email domains, else `team` |

Then `init`:

1. Generates the UUID of the knowledge base once.
2. Creates the repository skeleton: `AGENTS.md` (with the plain-Git rules, see `plain-git-access`), `.obsidian/app.json` (with `"useMarkdownLinks": true` and `"newLinkFormat": "relative"`, as MKBase 1.0 section 11 recommends), `.gitignore` (with `.obsidian/*` and `!.obsidian/app.json`), and `knowledge/` with the marker folder `knowledge/.mkbase/`, which holds the file `version` and the shared settings `waymark.toml`.
3. Commits and pushes it.

`init` runs only on a repository that has no `.mkbase` folder and no `knowledge/` folder. Anything else fails with `validation_failed` and no change. It is a write tool, so the limits apply: a layer can switch it off with `tools_denied = ["init"]`, and `access = "read-only"` blocks it.

## Interfaces & Contracts

```json
{ "tool": "init", "arguments": { "name": "Example", "slug": "example", "email_domains": ["example.org"], "audience": "team" } }
```

## Acceptance Criteria

- **Given** an empty remote repository and a running server, **When** `init` runs with a name, **Then** the repository has the skeleton and the first commit is pushed.
- **Given** the result, **When** another person adds the entry with the URL, **Then** they need not enter any shared value.
- **Given** `init` ran, **When** it runs again, **Then** it fails with `validation_failed` and changes nothing.
- **Given** a repository that is not empty, **When** `init` runs, **Then** it fails and changes nothing.
- **Given** the new repository, **When** it is opened in Obsidian, **Then** it works as a vault.
- **Given** `access = "read-only"`, **When** `init` runs, **Then** it fails with `access_denied`.

## Constraints & Edge Cases

- The repository should be private. waymark warns in `status` if it can see that it is public.
- A private person can create a knowledge base on github.com in the same flow.
- There is no terminal flow and no prompt. The agent or a client form collects the inputs.

## Open Questions

- Should `init` offer to create the remote repository when the Git host API allows it?
- Which inputs may a person leave to the defaults without losing the shared limits?
