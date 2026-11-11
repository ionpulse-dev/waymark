---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - login-and-identity
  - agent-tool-setup
---

# Join a knowledge base

## User Story

> As a new person, I want to install waymark and add one entry to my agent tool, so that my agents can use the knowledge base without any Git knowledge.

## Personas

- **Developer** — installs the binary and adds the entry.
- **Anyone with a coding agent** — follows the README steps. IT can prepare the machine (Git login, system config).

## Behaviour

Steps for a new person:

1. Install the binary (see `distribution`).
2. Make sure Git can sign in to the repository (see `login-and-identity`).
3. Add the MCP entry for the agent tool, from the README snippet, with the repository URL (see `agent-tool-setup`).
4. Start a session. The server clones the repository on first start and confirms with `status`.

On first start the server:

1. Checks that `git` works.
2. Clones the repository to `--path`, default `~/Projects/<repository-name>`, and asks for nothing.
3. Reads `.mkbase/waymark.toml` and the shared limits.
4. Checks the ID, if the entry names one.
5. Uses the author from the entry, or from the Git settings of the person, and checks the email (see `login-and-identity`).

There are no questions and no setup window. Everything the person must decide is an argument of the entry or a key of the personal config.

### Shared and personal settings

- **Shared** settings are in `.mkbase/waymark.toml` in the repository. People who join never type them again.
- **Personal** settings are in `~/.config/waymark/config.toml`. The file is optional. The server reads it at every call.

```toml
# ~/.config/waymark/config.toml  (all keys optional)
[limits]
access = "read-write"            # read-only | read-write
paused = false
tools_denied = []
allowed_repo_hosts = []          # empty = no restriction

[approval]
required_for = []
via = "elicitation"

[kb."<uuid>".limits]             # tightens further for one knowledge base
access = "read-only"
```

A personal value can only tighten the shared one (see `trust-model`). Other state is a small folder `~/.config/waymark/state/`, for example the time of the last `updates` call.

## Interfaces & Contracts

The MCP entry is the interface (see `mcp-server`). There is no `setup` command.

## Acceptance Criteria

- **Given** a Mac without working `git`, **When** the server starts, **Then** `status` reports it with a fix and no clone starts.
- **Given** a valid URL and a Git login, **When** the first session starts, **Then** the repository is cloned and `status` is ok.
- **Given** a personal value looser than the shared limit, **When** the server reads both, **Then** the stricter value applies.
- **Given** a person who adds a second entry for another knowledge base, **When** a session starts, **Then** the first one is unchanged.
- **Given** a repository whose `.mkbase/waymark.toml` has no `id`, **When** the server reads it, **Then** it reports `setup_problem`.
- **Given** the server started twice, **When** the second start ends, **Then** nothing is cloned twice.

## Constraints & Edge Cases

- A new person must reach a working setup in under 5 minutes with two steps: install the binary and add one entry.
- waymark never clones without an entry that the person (or the project and the person's agent tool) approved.

## Open Questions

- What does `status` show when the Git login gives no usable email?
- Is the default clone folder `~/Projects/<repository-name>` right for people who do not use that folder?
