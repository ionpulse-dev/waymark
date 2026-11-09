---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - login-and-identity
  - agent-tool-setup
  - git-sync
---

# Join a knowledge base

## User Story

> As a new person, I want to install the app and answer a few questions, so that my agents can use the knowledge base without any Git knowledge.

## Personas

- **Manager or other non-developer** — installs the app and answers questions with defaults.
- **Developer** — joins from a terminal with flags.

## Behaviour

Steps for a new person: download `Waymark.dmg`, drag the app to Applications, open it. The setup window asks the personal questions. Developers and IT use `install.sh` and then `waymark setup <content-repo-url>` with flags.

`setup`:

1. Checks that `git` works. If not, it starts Apple's installer for the command line tools (`xcode-select --install`).
2. Logs in to the Git host if needed (see `login-and-identity`).
3. Clones the content repository.
4. Reads `.mkbase/waymark.toml`.
5. Asks only the personal questions.
6. Writes the instruction blocks and skill files for the chosen agent tools (see `agent-tool-setup`).

Personal questions:

| Question | Default |
|---|---|
| Content repository URL | from the argument |
| Local folder | `~/Projects/<slug>-knowledge-base` |
| Agent tools to configure | tools found on the machine |
| Contact email address | from the Git host account, confirmed or corrected by the person |
| Personal read level, write level, confirm in | the shared start values, within the shared limits |
| Background sync on, and its interval | on, every 5 minutes |
| Access mode of this knowledge base | `read and write` when the person joins on purpose. A knowledge base added any other way starts `off`. |

Two places hold settings:

- **Shared** settings are in `.mkbase/waymark.toml` in the repository. People who join never type them again.
- **Personal** settings are in `~/.config/waymark/config.toml`, one entry per knowledge base, keyed by its UUID. A person can join several knowledge bases.

### Personal settings (`~/.config/waymark/config.toml`)

One entry per knowledge base, keyed by its UUID. Only the app writes the levels and the access mode (non-interactive `setup` may set initial values for rollout scripts).

```toml
[kb."<uuid>"]
slug = "example"                 # label
path = "~/Projects/example-knowledge-base"
mode = "read and write"          # off | read only | read and write
tools = ["claude-code", "codex"]
email = "jane@example.org"
sync = true
sync_interval_minutes = 5

[kb."<uuid>".read]
level = "manual"                 # auto | manual

[kb."<uuid>".write]
level = "manual"                 # auto | confirm | manual
confirm_in = "chat"              # chat | app
```

Other files in `~/.config/waymark/`: `default-id` (the personal default knowledge base), `installed.json` (the manifest), a status file written by the sync run, a hash of the settings (the app uses it to detect outside changes), and the time of the last `updates` call per knowledge base.

Without the app (first phase), the install uses the channel of `distribution` that exists then, for example `install.sh`.

## Interfaces & Contracts

```text
waymark setup [<content-repo-url>] [--path <folder>] [--tools <list>] [--email <address>]
              [--read <level>] [--write <level>] [--confirm-in <where>] ...
```

## Acceptance Criteria

- **Given** a Mac without `git`, **When** setup runs, **Then** it starts the installer once and continues when `git` works.
- **Given** a valid URL and login, **When** setup ends, **Then** the repository is cloned and the chosen tools are configured.
- **Given** a personal level looser than the shared limit, **When** setup stores it, **Then** the stricter value is stored.
- **Given** a person who joins a second knowledge base, **When** setup ends, **Then** the first one is unchanged.
- **Given** a repository whose `.mkbase/waymark.toml` has no `id`, **When** setup reads it, **Then** it stops with a clear message.
- **Given** setup ran twice, **When** the second run ends, **Then** nothing is duplicated.

## Constraints & Edge Cases

- A new person must reach a working setup in under 5 minutes with two steps.
- waymark never clones a repository without a person confirming it.

## Open Questions

- What does the setup window show when the Git host account has no usable email?
