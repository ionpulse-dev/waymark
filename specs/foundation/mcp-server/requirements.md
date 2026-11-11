---
type: foundation
status: draft
created: 2026-11-11
updated: 2026-11-11
requires:
  - git-sync
  - trust-model
---

# MCP server

## Technical Goal

One binary, `waymark`, is an MCP server and holds all logic: documents, Git work, checks, and limits. MCP is the only interface for agents. There is no CLI for agents and no skill. This item defines what every tool shares: the transports, the tool list, results, errors, configuration, technology, and tests. It replaces `cli-core`.

## Dependents

- Every feature spec. Each defines the tools it adds.
- `server-mode`, `multi-tenancy`, `web-console`, `write-approval`, and `distribution`.

## Deliverables

- A portable Rust binary, one static binary per platform.
- The MCP server over stdio (local mode). `server-mode` adds the HTTPS transport.
- The tool list and the rules for tool descriptions.
- The result format and the error codes.
- The configuration: arguments, environment variables, and the config layers (see `trust-model`).
- A conformance test suite.

## Scope

**In scope:**
- The behavior that all tools share, the technology, and the tests.

**Out of scope:**
- The behavior of single tools (see the feature specs).
- The HTTPS transport, login, and tenancy (see `server-mode` and `multi-tenancy`).
- A skill, an app, or any installer that writes into agent tools. People add the MCP entry to their agent tool by hand (see `agent-tool-setup`).

## Constraints

- **MCP is the only interface.** Every function for agents is a tool. The binary has no agent commands. It may answer `--version` and `--help`, and nothing more.
- **One core, two transports.** The core knows nothing about stdio or HTTPS. Tool handlers are the same in both modes.
- **The server enforces, the team advises.** Hard rules live in the server (see `trust-model`). waymark gives no advice on behavior. Teams write their own rules in `AGENTS.md`. The README only shows examples.
- **Tool descriptions are the only built-in guidance.** They are short, plain, and exact. They say what the tool does and what it returns. They do not give behavior advice.
- **Small tool set.** The tool schemas use context in every session. A tool is added only when a feature needs it.
- **No control tools.** There is no tool that changes limits, access, pause, credentials, or approval settings. An agent can never loosen its own limits.
- **Credentials never pass through the agent.** No tool receives or returns a secret.
- **Stateless between sessions.** The server keeps state only for one session (the read budget). Everything else is in Git, the config files, or the server's data folder.
- **Idempotent.** The same call twice gives the same repository state.
- **English only.** The `message` fields and the tool descriptions are English.

### Transports

| Mode | Transport | Started by |
|---|---|---|
| Local | MCP over stdio | The agent tool, with an entry in its MCP config. One process per session. |
| Server | MCP over HTTPS | The operator (see `server-mode`) |

### Tools

| Tool | Task | Kind | Specified in |
|---|---|---|---|
| `status` | State of the setup, the knowledge base in use, the effective limits, open notices | read | `maintenance`, `session-start` |
| `search`, `show`, `list`, `stats` | Find and read documents | read | `search-and-read` |
| `updates` | What changed since a date | read | `updates` |
| `lint` | Check documents | read | `quality-checks` |
| `activity` | Recent writes of the person | read | `activity-and-undo` |
| `add`, `update_document`, `dispute`, `deprecate` | Create and change documents | write | `write-documents` |
| `delete` | Delete a document | destructive | `write-documents` |
| `move` | Move or rename documents | write | `move-documents` |
| `undo` | Revert one write | write | `activity-and-undo` |
| `sync` | Commit pending manual edits, pull, push | write | `git-sync`, `manual-edits` |
| `index` | Generate index files | write | `quality-checks` |
| `init` | Create the structure of an empty knowledge base repository | write | `create-knowledge-base` |
| `verify`, `review`, `reject` | Owner verification. Present only when verification is on. | write | `verification` |

Every tool has a JSON input schema and a kind hint (read-only, write, or destructive) so that clients can ask the person per tool. A tool that the limits switch off is not listed (see `trust-model`).

Not tools: login, credentials, limits, approval settings, pause, setup of agent tools.

### Result format

Every result carries a plain-words `message` and a structured copy:

| Field | When | Content |
|---|---|---|
| `message` | always | A short sentence in plain English. No Git words such as rebase, branch, or conflict. |
| `notices` | when needed | Things the person should know, also unrelated to the call. Example: a document of the person was disputed. |
| `settings` | always | The effective limits for this call: `access`, `paused`, tools switched off, folders, caps, and approval rules. |

Read tools return the data and the three fields. A document in a result carries its age (`updated`) and its dispute reason, so that the agent can pass them on.

### Errors

A failure sets the MCP error flag. The structured result has `code`, `message`, and, where it helps, `next`: one plain sentence on what to do. A rejection teaches. Example: "A document with this title exists: `ops/deploy`. Update it instead."

| Code | Meaning |
|---|---|
| `validation_failed` | Format, size, link, duplicate, or secret check failed |
| `sync_failed` | No network, Git login, or push rejected |
| `setup_problem` | The knowledge base is not set up or not reachable. Run `status`. |
| `access_denied` | A limit blocks the call (access, folder, tool switched off) |
| `stale_base` | The document changed since the agent read it |
| `approval_denied` | The person said no, or no answer came in time |
| `approval_unavailable` | Approval is required but cannot be asked (client lacks the feature, or the console is not connected) |
| `paused` | Writes are paused |

### Configuration

The agent tool starts the server from an entry in its MCP config. The entry passes arguments and environment variables:

```json
{
  "mcpServers": {
    "waymark": {
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/knowledge.git", "--access", "read-write"],
      "env": { "WAYMARK_AUTHOR_NAME": "Jane Doe", "WAYMARK_AUTHOR_EMAIL": "jane@example.org" }
    }
  }
}
```

| Argument or variable | Meaning |
|---|---|
| `--repo` | Git URL of the knowledge base. The server clones it on first start. |
| `--kb-id` | Optional. The ID that `.mkbase/waymark.toml` must have. A different ID is `setup_problem` (see `project-binding`). |
| `--path` | Local folder of the clone. Default `~/Projects/<slug>-knowledge-base`. |
| `--author-name`, `--author-email` (or `WAYMARK_AUTHOR_NAME`, `WAYMARK_AUTHOR_EMAIL`) | Commit author. Default: the Git settings of the person. |
| Any limit of `trust-model` | For example `--access`, `--no-delete`, `--folders`. |

A secret is never an argument. In local mode, Git uses its own login. In server mode, secrets come from the host (see `server-mode`).

### Technology

| Need | Choice |
|---|---|
| Language | Rust |
| MCP | an MCP library for Rust, checked at build time (for example `rmcp`) |
| YAML frontmatter | `serde` with a maintained YAML crate |
| Git | the `git` binary (see `git-sync`) |

### Conformance tests

A fixture repository, a tool call, and the expected result fix the behavior, so the server can be rebuilt or ported. The tests call the tools through an in-memory MCP client, in both transports. They run in CI.

## Acceptance Criteria

- **Given** a started server, **When** the client lists tools, **Then** the list holds the tools above that the limits allow, and no control tool.
- **Given** any tool call, **When** it ends, **Then** the result has `message` and `settings`, and `notices` when there are any.
- **Given** a client that asks for a tool that changes limits, **When** it calls it, **Then** the server reports an unknown tool.
- **Given** the same call twice, **When** the second ends, **Then** the repository state is the same as after the first.
- **Given** the fixture suite, **When** it runs on a new build, **Then** all expected results match, in stdio and in HTTPS.
- **Given** the release binary, **When** it runs on a machine without Node, Python, or Go, **Then** it works. `git` is the only requirement.
- **Given** a rejected call, **When** the result returns, **Then** it has a `code` and, where a fix exists, a `next` sentence.
- **Given** an entry with `--repo` and a new URL, **When** the server starts, **Then** it clones the repository once and reuses the clone later.

## Open Questions

- How does the server find the knowledge base when the client starts it in another folder? The entry carries `--repo`, so the working folder may be unnecessary.
- Are documents also offered as MCP resources, besides the tools?
- How small can the tool set be, to keep the context cost low? Can `list` and `stats` merge into `search`?
- Which Rust toolchain version and which MCP library?
- Which platforms get a release, and when?
- Can an embedded Git library remove the dependency on the `git` binary?
