# Architecture direction

Status: agreed in a design discussion on 2026-11-11. All specs were revised to match it on the same day. They are all `draft`, and each needs review and approval before implementation starts. The section "Impact on the specs" records what changed.

## Summary

waymark is **one binary**. It is an MCP server and it holds all logic: documents, Git work, checks, and limits. MCP is the **only interface for agents**. There is no CLI for agents, and waymark does not ship or install a skill. Teams describe the behavior they want in their own `AGENTS.md`. The README only gives examples.

```text
agent tool or chat app ⇄ MCP ⇄ waymark (core: documents, checks, Git) ⇄ Git repository of the owner
```

## Why MCP and not a CLI with a skill

- All three target agent tools (Claude Code, GitHub Copilot, OpenAI Codex) support MCP. This is from memory and must be verified (see open questions).
- Tool descriptions are always in the context of the agent. A skill can fail to load.
- Tool input is typed. There are no shell quoting problems.
- Many clients ask the person before a tool runs, per tool.
- One interface means one behavior to build and test.
- Chat apps and phones can only use MCP, and only a remote one.

Costs: the tool schemas use context in every session, so the tool set must stay small. Agents without MCP support cannot use waymark. The standalone plain-Git skill remains the fallback for them.

## Principles

1. **The server enforces, the team advises.** Hard rules live in the server: format, size, secrets, links, duplicates, result caps, a read budget per session, access mode, and which tools exist. Behavior advice ("read at most 5 documents") becomes a limit in the server. What an agent does and when, is the choice of the team, written in its `AGENTS.md`.
2. **Rejections teach.** An error message tells the agent what to do next ("update `ops/deploy` instead").
3. **Results carry the facts.** A result includes the age of a document and its dispute reason, so the agent can pass them on.
4. **No control tools.** There is no tool to change limits, the access mode, or credentials. An agent can never loosen its own limits.
5. **Git credentials never pass through the agent.** See "Credentials".
6. **Git stays the source of truth.** A deployment never owns the data. It holds a cache clone of the Git repository.

## One core, two modes

The core has no knowledge of the transport. The same tools and the same logic run in two modes.

| Mode | Transport | For | Notes |
|---|---|---|---|
| **Local** | MCP over stdio. The agent tool starts the binary for one session. | One person or a small team on desktops | No infrastructure. Uses the Git login of the person. Quick start. |
| **Server** | MCP over HTTPS, with OAuth 2.1 | A company or provider that runs a shared server. Also chat apps and phones. | Needs identity, tenancy, and credential handling. |

Anyone can host the server mode: a company on its own infrastructure, or a provider for others. The same code serves both. The question whether the project itself operates a hosted service is not decided here. The decision "no servers that we operate" stays valid until it is changed on purpose.

### Build order

1. Core and local mode (stdio). This is the first usable product and the quick start.
2. Server mode for one organization, self-hosted: single sign-on, GitHub App, web console.
3. Multi-tenant server mode on top of step 2.

Tenancy goes into the data model from step 1, so it does not have to be added later.

### Server mode components

```text
client ⇄ MCP over HTTPS ⇄ [ identity | tenancy | policy | credentials | audit ] ⇄ core ⇄ Git repository
```

- **Identity.** Single sign-on (OIDC or SAML) for organizations. Email or Git host login for private people.
- **Tenancy.** Each organization has its own memory repositories, settings, and limits. Nothing is visible across tenants.
- **Git credentials.** A GitHub App installed on the organization is the preferred model: short-lived, scoped to chosen repositories, and no pasted tokens. A deploy key or a fine-grained token also works for a small setup.
- **Commit author.** The commit author is the person who used the agent, taken from the login. The push credential only proves that the server may write. The two are separate.
- **Authorization.** Who may read or write which memory repository.
- **Workspace.** One cache clone per memory repository, with a write lock.
- **Audit log.** Who did what, and when.
- **Web console.** A web page for approval of writes, the activity list, and undo. The person signs in separately, so the agent cannot answer it. It replaces the macOS app.

## Configuration and limits

The MCP config entry of the agent tool starts the server and passes arguments and environment variables.

```json
{
  "mcpServers": {
    "waymark": {
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git",
               "--access", "read-write", "--no-delete", "--folders", "engineering/**"]
    }
  }
}
```

The server clones the repository on first start. No separate init or setup command is needed.

### Permission ceiling

A ceiling is the most that the server ever allows. Layers, from loosest to strictest source:

| Layer | Set by | Strength |
|---|---|---|
| MCP config of the user | The person | A safety default. An agent with file access could edit the file for the next session. |
| Managed MCP config | IT, pushed to desktops | Stronger, if the agent tool supports managed config (to be verified) |
| Shared file in the memory repository (`.waymark/settings.toml`) | The team owner | The server refuses to change this file through its tools |
| Server config (server mode) | The operator | Hard. The user cannot touch it. |

**The strictest layer wins.** A layer can tighten a limit and never loosen it. In server mode the client config can never be the ceiling, because the user controls it. The server enforces limits by user or role.

Possible ceilings: access mode (read-only or read-write), allowed tools (no delete, no move), allowed folders, result caps, size caps, and a read budget per session.

### Credentials

| Where | How |
|---|---|
| Local mode | Git's own login: an SSH key or a credential helper. waymark never sees the secret. |
| Server mode | Stored in the secret store of the host, passed as an environment variable or a mounted file. Never in the image, the repository, or the MCP config. |
| Server mode, later | A web page where the person connects a Git account with "Sign in with GitHub". The server then stores tokens encrypted at rest. |

## What MCP can and cannot do

**Can:** give every MCP client the same tools; run remotely for chat apps and phones; carry usage instructions to the model; ask the person through the client (elicitation, where supported).

**Cannot:** force the agent to call a tool; prove that a human answered; run work without an open session; show windows or notifications of its own; stop an agent that has a shell from using raw Git in the repository.

## Impact on the specs

Done on 2026-11-11. All specs are `draft`.

**New**
- `foundation/mcp-server` — the core and the MCP server in one binary. Replaces `cli-core`.
- `foundation/server-mode` — the HTTPS transport, sign-in, roles, Git credentials, workspace, and audit log for one organization.
- `foundation/multi-tenancy` — many organizations on one server.
- `features/write-approval` — approval through MCP elicitation or the console.
- `features/web-console` — approval, activity, undo, review, and pause on a server.

**Superseded** (the folders stay as a record, see `specs/INDEX.md`)
- `foundation/cli-core` by `mcp-server`.
- `foundation/agent-skill` by `agent-tool-setup` and `mcp-server`.
- `foundation/mac-app` by `web-console` and `distribution`.
- `foundation/app-link` by `write-approval` and `web-console`.
- `features/approval-dialog` by `write-approval` and `web-console`.

**Rewritten or revised**
- `foundation/trust-model`: limits and config layers replace the read level, write level, and the app.
- `foundation/memory-repository`: the `settings.toml` schema has `[limits]` and `[approval]`. No `.waymark/` folder.
- `foundation/git-sync`, `foundation/distribution`, `foundation/document-format`.
- All feature specs that remain: the commands became tools, and behavior advice left the specs.
- `specs/PROJECT.md`, `specs/INDEX.md`, `specs/WORKFLOW.md` (new status `superseded`), and the README.

## Open questions

- How does the server find the memory repository: the working folder, the MCP roots, or a flag?
- A read budget per session needs state in the server. How does this fit the earlier "no server state" rule?
- Do Claude Code, GitHub Copilot, and OpenAI Codex all support remote MCP servers, managed MCP config, and per-tool permission prompts? Verify against current documentation.
- Are documents also offered as MCP resources, besides the tools?
- Which Rust MCP library fits?
- Which embedded Git approach avoids a dependency on the `git` binary, or is the dependency acceptable?
- How big should the first tool set be, to keep the context cost low?
