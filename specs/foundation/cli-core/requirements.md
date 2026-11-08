---
type: foundation
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - knowledge-base-repo
  - trust-model
---

# CLI core

## Technical Goal

The `waymark` command is the only tool that touches a content repository for agents. It does all Git work and enforces the rules for format, size, links, and secrets. This item defines what every command shares: principles, output, exit codes, technology, and tests.

## Dependents

- Every feature that adds a command.
- `app-link`, `mac-app`, `agent-skill`, and `distribution`.

## Deliverables

- A portable Rust CLI, one static binary per platform.
- The output contract and the exit codes.
- The compiled-in skill and templates.
- A conformance test suite.

## Scope

**In scope:**
- Shared behavior of all commands, the command list, technology, tests.

**Out of scope:**
- The behavior of single commands (see the feature specs).
- The app (see `mac-app`).

## Constraints

- **Agent first.** Every command supports `--json`. Output is stable and short.
- **Non-interactive for agents.** Only `init`, `setup`, `login`, and `doctor` ask questions, and only when a human runs them in a terminal. Every question also has a flag. With all flags set, or without a terminal, no prompt appears.
- **Idempotent.** Running a command twice gives the same result.
- **Content stays plain Markdown.** The CLI is a convenience and a guard.
- **No server.** A command runs, does its work, and exits.
- **Portable.** macOS first. Linux and Windows are possible later. Nothing in the CLI is macOS-specific except what it asks the app for.
- The skill text and the plain `message` fields are English only.

### Output contract

Every output has machine fields and a plain-words `message`. No Git words such as rebase, branch, or conflict.

| Field | When | Content |
|---|---|---|
| `message` | always | A short sentence in plain English. |
| `notices` | when needed | A list of things the person should know, also unrelated to the command. The skill tells the agent to pass them on at once. |
| `settings` | always | The current `read_level`, `write_level`, `confirm_in`, `paused`, and `mode`. The agent follows the latest value, so a change in the app applies at once. |

The CLI has no window and no web page. The agent is the user interface:

```text
waymark → output (JSON + message) → agent → person in the chat
```

### Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error |
| 2 | Validation failed (lint, schema, secrets) |
| 3 | Sync failed (no network, login, or push rejected) |
| 4 | Setup problem (run `waymark doctor`) |
| 5 | Document changed since the agent read it |
| 6 | Write canceled by the person, or no answer in time |
| 7 | Approval needed, but the app is not running |
| 8 | Writes are paused |

### Commands

| Group | Commands | Specified in |
|---|---|---|
| Setup | `init`, `setup`, `login`, `credential` | `create-knowledge-base`, `join-knowledge-base`, `login-and-identity` |
| Maintenance | `doctor`, `status`, `update`, `uninstall` | `maintenance` |
| Binding | `link`, `unlink`, `--kb <uuid>` | `project-binding` |
| Session | `session` | `session-start` |
| Read | `search`, `show`, `list`, `stats`, `updates`, `stale` | `search-and-read`, `updates-and-stale` |
| Write | `add`, `update-document`, `dispute`, `deprecate`, `move` | `write-documents`, `move-documents` |
| Sync | `sync` | `git-sync` |
| Quality | `lint`, `index` | `quality-checks` |
| Control | `pause`, `resume` | `pause-and-resume` |
| Review | `verify`, `review`, `reject` | `verification` |
| Later | `mcp` | ideas |

### Technology

| Need | Choice |
|---|---|
| Language | Rust |
| Arguments | `clap` |
| YAML frontmatter | `serde` with a maintained YAML crate |
| Git | the `git` binary |
| Keychain | through the app link, or `security-framework` |
| Embedded assets | skill text and templates compiled in |

Libraries are checked again at build time.

### Conformance tests

A fixture repository, a command, and the expected output fix the behavior of the CLI, so it can be rebuilt or ported. They run in CI.

## Acceptance Criteria

- **Given** any command with `--json`, **When** it runs, **Then** the output has `message` and `settings`, and `notices` when there are any.
- **Given** no terminal, **When** `setup` runs without all flags, **Then** it fails with exit code 4 instead of asking.
- **Given** the same command run twice, **When** the second run ends, **Then** the repository state is the same as after the first run.
- **Given** a write on a paused system, **When** it runs, **Then** it exits with code 8.
- **Given** the fixture suite, **When** it runs on a new build, **Then** all expected outputs match.
- **Given** the release binary, **When** `waymark --version` runs on a machine without Node, Python, or Go, **Then** it works.

## Open Questions

- What is the minimum Rust toolchain version?
- Which platforms get a release of the CLI alone, and when?
