---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# Create a knowledge base

## User Story

> As the first person of a company, a team, or just myself, I want to create a knowledge base in a few minutes, so that others and my agents can use it.

## Personas

- **Knowledge base owner** — creates the knowledge base and sets the shared settings.
- **Private person** — creates a private knowledge base for their own AI memory.

## Behaviour

The flow runs in the setup window of the app, or in a terminal with `waymark init`. Every question has a default and a flag. With all flags set, or without a terminal, no prompt appears.

`init` asks for:

| Question | Stored in | Default |
|---|---|---|
| Name of the company or project | `.mkbds/waymark.toml` | none |
| Short name (slug) | `.mkbds/waymark.toml` | derived from the name |
| Git host and repository URL | `.mkbds/waymark.toml` | none |
| First folders | `knowledge/` | no folders |
| Allowed email domains (optional) | `.mkbds/waymark.toml` | none |
| Start values and limits for read level, write level, and confirm in | `.mkbds/waymark.toml` | start `manual`, `manual`, `chat`; no limit |
| Who will use it: only me, or several people | which standalone skill goes into `AGENTS.md` | derived from the allowed email domains, else several people |

Then `init`:

1. Generates the UUID of the knowledge base once.
2. Creates the repository skeleton: `AGENTS.md` (with the plain-Git rules), `.obsidian/app.json`, `.gitignore`, and `knowledge/` with the marker folder `knowledge/.mkbds/`, which holds the file `version` and the shared settings `waymark.toml`.
3. Commits and pushes it.
4. Runs `setup` for this person (see `join-knowledge-base`).

## Interfaces & Contracts

```text
waymark init [--name <name>] [--slug <slug>] [--repo <url>]
             [--email-domains <list>] [--read-default <level>] [--write-default <level>] ...
```

Exit code 4 if a value is missing and no terminal is available.

## Acceptance Criteria

- **Given** an empty remote repository, **When** `init` runs with all flags, **Then** the repository has the skeleton and the first commit is pushed.
- **Given** the result, **When** another person runs `setup` with the URL, **Then** they need not enter any shared value.
- **Given** `init` ran twice with the same input, **When** the second run ends, **Then** nothing changes and no error appears.
- **Given** a remote repository that is not empty, **When** `init` runs, **Then** it stops with a clear message and changes nothing.
- **Given** the new repository, **When** it is opened in Obsidian, **Then** it works as a vault.

## Constraints & Edge Cases

- The remote repository is created by the person on the Git host. waymark does not create repositories.
- The repository should be private. waymark warns if it can see that it is public.
- A private person can create a knowledge base on github.com in the same flow.

## Open Questions

- Should `init` offer to create the remote repository when the Git host API allows it?
