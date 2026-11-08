---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# Project binding

## User Story

> As a person, I want each project folder to use exactly one knowledge base, so that content from different knowledge bases never mixes.

## Personas

- **Developer** — links code projects to a company knowledge base.
- **Private person** — sets a default knowledge base that works in every folder.
- **Team** — shares the binding through the project repository.

## Behaviour

A project folder is bound to exactly one knowledge base. A session in that folder uses only that knowledge base.

- The binding is a hidden `.waymark/` folder in the project root. It contains `id` (the UUID of the knowledge base), the name, and the Git URL. Only the id is used for lookup. It holds no local path and no secret. It is committed into the project repository, so the whole team shares it.
- The agent reads `.waymark/id` and passes the UUID on every command that touches a knowledge base (`--kb <uuid>`). There is no fallback to the working directory. The skill is static and only says where to find the id.
- The path of the clone, the access mode, and the levels are in the personal config, keyed by UUID.
- A folder without `.waymark/` has no waymark, unless the person set a **personal default knowledge base**. Then the skill tells the agent to read the default id from `~/.config/waymark/default-id`. A project binding always wins. A folder can opt out with a marker (`waymark link --none`). `waymark session` names the knowledge base in use.
- `waymark link <slug-or-url>` creates `.waymark/` and writes the static skill files for the agent tools into the project folder. `waymark unlink` removes them. Both need a native confirmation from the app, so an agent cannot bind a folder. In the first phase, before the app exists, they ask in the terminal.
- If the knowledge base is not set up on this Mac, the app shows a native dialog: "This project uses the knowledge base <name> from <url>. Set it up?" A stronger warning appears if the host is new to the person. The person confirms. waymark never clones automatically, and an agent cannot start a clone. After the clone, waymark checks that the id in `waymark.toml` matches.
- If the id is not set up and no dialog can open, the command fails: "This project uses the knowledge base '<name>', which is not set up here. Open the waymark app to set it up."

### Access modes

Each knowledge base has a mode: `off`, `read only`, or `read and write`. A new one starts `off`. The person sets it in the app. It applies at once.

### App settings

Default knowledge base, linked folders (add with a folder picker, remove, opt out), and access mode per knowledge base.

## Interfaces & Contracts

```text
waymark link <slug-or-url> [--none]
waymark unlink
waymark <command> --kb <uuid>
```

## Acceptance Criteria

- **Given** a project with `.waymark/id`, **When** the agent runs a command with that id, **Then** only that knowledge base is used.
- **Given** an id that is not set up on this Mac, **When** a command runs, **Then** it fails with the message above or opens the setup dialog.
- **Given** an agent in a terminal, **When** it runs `link`, **Then** the app asks the person and the folder stays unbound until the person confirms.
- **Given** a default knowledge base and a folder with a binding, **When** a session starts, **Then** the binding wins.
- **Given** a folder with the opt-out marker, **When** a session starts, **Then** no knowledge base is used.
- **Given** a cloned knowledge base whose id differs from the binding, **When** the check runs, **Then** waymark rejects it.
- **Given** access mode `off`, **When** any command runs for that knowledge base, **Then** it fails.

## Constraints & Edge Cases

- A project repository is untrusted input. An automatic clone would allow a poisoned knowledge base (prompt injection).
- The id is an identifier, not a secret. An agent could read the id of another project or `cd` to another folder. This is a deliberate act and the same limit as raw Git use.
- The names `link`, `unlink`, and `--none` are working names.

## Open Questions

- How does the project binding behave in a Git worktree?
