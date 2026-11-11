---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Project binding

## User Story

> As a person, I want each project to use exactly one knowledge base, so that content from different knowledge bases never mixes.

## Personas

- **Developer** — links code projects to a company knowledge base.
- **Private person** — sets a default knowledge base that works in every folder.
- **Team** — shares the binding through the project repository.

## Behaviour

A project uses a knowledge base through an **MCP entry** of its agent tool. The entry names the repository with `--repo`. One server process serves exactly one knowledge base, so content cannot mix.

- **Project entry.** The entry is in the project-level MCP config of the agent tool (for example `.mcp.json` of the project). It is committed into the project repository, so the whole team shares it. It holds the repository URL and no secret and no personal path.
- **Personal default.** An entry in the user-level MCP config of the agent tool works in every folder. This is the personal default knowledge base.
- **Both present.** Use the same server name in both entries. Agent tools let the project entry override the user entry of the same name. This precedence must be verified for each of the three tools. A project can opt out of the default by an entry for the same name that is disabled.
- **Identity.** After the clone, the server checks that the `id` in `.mkbase/waymark.toml` is the one the entry expects, if the entry names one (`--kb-id`). A different ID is `setup_problem`.
- **No hidden folder.** There is no `.waymark/` folder in the project and no `link` or `unlink` command. The person edits the MCP config of their agent tool.

### Trust in a project entry

A project repository is untrusted input. A poisoned entry could point to another repository. Protections:

- Agent tools ask the person before they start a project-level MCP server for the first time. This must be verified for each tool.
- The limit `allowed_repo_hosts` (a list of hosts, personal or system layer, see `trust-model`) makes the server refuse to clone from other hosts. The default is no restriction.
- A new clone happens only on first start. The server never clones a second repository in a running session.

### Limits per knowledge base

Limits can be set per knowledge base in the personal config, in a section keyed by the knowledge base ID. They follow the layer rules of `trust-model`.

## Interfaces & Contracts

```json
{ "mcpServers": { "waymark": { "command": "waymark", "args": ["--repo", "git@github.com:example/knowledge.git", "--kb-id", "<uuid>"] } } }
```

## Acceptance Criteria

- **Given** a project entry with `--repo`, **When** a session starts in that project, **Then** only that knowledge base is used.
- **Given** an entry with `--kb-id` and a repository with another ID, **When** the server starts, **Then** it reports `setup_problem` and serves nothing.
- **Given** `allowed_repo_hosts = ["git.example.org"]`, **When** an entry names another host, **Then** the server refuses to clone.
- **Given** a user entry and a project entry with the same name, **When** a session starts in the project, **Then** the agent tool uses the project entry (to verify per tool).
- **Given** the server is running, **When** a tool asks for another repository, **Then** it is refused. The tools take no repository argument.

## Constraints & Edge Cases

- The ID is an identifier, not a secret.
- An agent that edits the MCP config of its tool could add an entry. This is a deliberate act and the same limit as raw Git use.
- How a Git worktree shares an entry is open.

## Open Questions

- Do Claude Code, Copilot, and Codex all let a project entry override a user entry of the same name?
- Do they all ask before they start a project-level MCP server?
- How does the project binding behave in a Git worktree?
