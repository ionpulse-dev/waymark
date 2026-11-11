---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - agent-tool-setup
---

# Maintenance

## User Story

> As a person, I want to check, update, and remove waymark easily, so that I always know it works and I can leave without leftovers.

## Personas

- **Every person** — checks health, updates, removes.
- **IT or an operator** — runs the same checks in scripts and monitoring.

## Behaviour

waymark has no commands for people. Health checks are a tool, and update and removal use the tools that installed waymark.

### The `status` tool

`status` is read-only and runs in every limit state. It returns `checks`, each with `name`, `ok`, and `fix`, plus the facts of `session-start`:

- the clone is present, clean, and its remote is reachable,
- the Git login works (the server runs a read-only call to the remote),
- `git` works (`git --version`, because `/usr/bin/git` on macOS can be a stub),
- the repository matches the ID of `.mkbase/waymark.toml`,
- the effective limits and which layer set each (see `trust-model`),
- the last good pull and the last error,
- the version of the binary.

A failed check carries a plain-words `fix`, for example "Git cannot sign in. Sign in with Git first, for example with `gh auth login`, or set up an SSH key."

In server mode, `/health` returns the same facts for operators, without secrets.

### Update

- **Local mode.** The person updates the binary with the package manager or a new download. The tool descriptions come with the binary.
- **Server mode.** The operator deploys a new image tag.

### Removal

The README lists the steps. waymark does not remove itself:

1. Remove the MCP entry from the agent tool.
2. Check the clone for changes that are not pushed (`git status`), then delete the folder if no longer needed. The remote repository stays.
3. Delete `~/.config/waymark/`.
4. Remove the binary with the package manager.

waymark never writes into agent tools, so nothing else is left behind.

## Interfaces & Contracts

`status` returns `checks`: a list of `{ name, ok, fix }`, and the fields `message`, `notices`, and `settings`.

## Acceptance Criteria

- **Given** a broken Git login, **When** `status` runs, **Then** it reports the problem and the fix.
- **Given** a healthy setup, **When** `status` runs, **Then** all checks are ok.
- **Given** `access = "read-only"` or `paused = true`, **When** `status` runs, **Then** it still works and names the state.
- **Given** a repository with another ID than expected, **When** `status` runs, **Then** it reports it.
- **Given** the steps of the README, **When** a person follows them, **Then** no waymark file or entry remains, except the repository they chose to keep.
- **Given** a running server, **When** `/health` is called, **Then** it returns the state without a secret.

## Constraints & Edge Cases

- `status` runs a network call. It has a short time-out and reports "not reachable" instead of waiting.

## Open Questions

- Is a small `waymark doctor` command worth adding later for people without an agent?
