---
type: foundation
status: implemented
created: 2026-11-11
updated: 2026-11-15
requires:
  - git-sync
  - trust-model
---

# MCP server

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Only the stdio transport is built. Parts that need server mode stay in this text as a record, and they are not implemented now (see [project decisions](../../../docs/decisions.md)).

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
| `status` | State of the setup, the memory repository in use, the effective limits, open notices | read | `maintenance`, `session-start` |
| `search`, `show`, `stats` | Find, list, and read documents (`search` without a query lists) | read | `search-and-read`, `search-filters`, `document-history`, `lean-tools` |
| `history` | The versions of one document | read | `document-history` |
| `folder_tree` | The folders with counts | read | `folders` |
| `list_tags` | The tags with counts | read | `manage-tags` |
| `updates` | What changed since a date | read | `updates` |
| `lint` | Check documents | read | `quality-checks` |
| `activity` | Recent writes of the person | read | `activity-and-undo` |
| `add_document`, `update_document`, `dispute_document`, `deprecate_document` | Create and change documents | write | `write-documents` |
| `restore_document` | Bring a deprecated document back | write | `restore-and-replace` |
| `rename_tag` | Rename or merge a tag in all documents | write | `manage-tags` |
| `create_folder` | Create an empty folder | write | `folders` |
| `delete_document` | Delete a document | destructive | `write-documents` |
| `delete_folder` | Delete a folder with its documents | destructive | `folders` |
| `move` | Move or rename documents | write | `move-documents` |
| `undo` | Revert one write | write | `activity-and-undo` |
| `sync` | Commit pending manual edits, pull, push | write | `git-sync`, `manual-edits` |
| `init` | Create the structure of an empty memory repository | write | `create-memory-repository` |
| `verify_document`, `review`, `reject_document` | Owner verification. Present only when verification is on. | write | `verification` |

The names, descriptions, parameters, paging, and dates of all tools follow [tool-conventions](../tool-conventions/requirements.md).

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
| `setup_problem` | The memory repository is not set up or not reachable. Run `status`. |
| `access_denied` | A limit blocks the call (access, folder, tool switched off) |
| `stale_base` | The document changed since the agent read it |
| `approval_denied` | The person said no, or no answer came in time |
| `not_found` | The document or the memory repository item does not exist |
| `invalid_arguments` | The call has a missing or wrong argument. The `next` sentence names it. |
| `approval_unavailable` | Approval is required but cannot be asked (client lacks the feature, or the console is not connected) |
| `paused` | Writes are paused |

### Configuration

The agent tool starts the server from an entry in its MCP config. The entry passes arguments and environment variables:

```json
{
  "mcpServers": {
    "waymark": {
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git", "--access", "read-write"],
      "env": { "WAYMARK_AUTHOR_NAME": "Jane Doe", "WAYMARK_AUTHOR_EMAIL": "jane@example.org" }
    }
  }
}
```

| Argument or variable | Meaning |
|---|---|
| `--repo` | Git URL of the memory repository. The server clones it on first start, into `--path` or into `~/.waymark/<name>`. One of `--repo` and `--path` is required (see `project-binding`). |
| `--memory-repository-id` | Optional. The ID that `.waymark/settings.toml` must have. A different ID is `setup_problem` (see `project-binding`). |
| `--path` | Local folder of the memory repository. `~` is the home folder. With `--repo`, a missing or empty folder is the clone target. Without `--path`, `--repo` uses `~/.waymark/<name>`. |
| `--author-name`, `--author-email` (or `WAYMARK_AUTHOR_NAME`, `WAYMARK_AUTHOR_EMAIL`) | Commit author. Default: the Git settings of the person. |
| Any limit of `trust-model` | For example `--access`, `--no-delete`, `--folders`. |

A secret is never an argument. In local mode, Git uses its own login. In server mode, secrets come from the host (see `server-mode`).

### Updates of the MCP entry

The entry of the agent tool (the command and its arguments) belongs to the person. waymark never writes into an agent tool, so a change of the entry is always made by the person. This section says how waymark makes sure that the person learns about it. The whole path of updates (binary, settings schema, repairs, migrations) is in `memory-repository`, section "Versions and updates".

- **The server does not exit on a bad argument.** An argument that this waymark does not know, or a wrong value, never stops the process. The server starts, answers `initialize`, and lists `status`. (`--help` and `--version` work as before.)
- **Deprecated arguments.** The binary has a list of arguments that no longer have an effect (`DEPRECATED_FLAGS` in `src/mcp/args.rs`), each with a sentence. Such an argument is ignored, the tools work, and every result has a notice: "The MCP entry uses `--x`. <sentence>". `status` shows it in the check `entry` (ok). An argument is removed from the program only after it was in this list for at least one minor version.
- **Any other unknown argument blocks the tools.** It may come from a newer waymark, and it may be a limit that this build would ignore. The server then needs an update: see below. The same holds for a wrong value of a known argument (for example `--max-results many`): the server must not guess a memory repository, so it starts with no arguments at all.
- **A new required argument** is handled like an unknown one from the other side: a build that needs an argument and does not find it starts with the same state and says what to add. A release must not add a required argument without this check.
- **"Needs an update" state.** It exists for three causes: an unknown or wrong argument in the entry, a settings file of a higher schema (see `memory-repository`), and a required argument that is missing. In this state:
  - `initialize` sends `instructions` that start with "WAYMARK NEEDS AN UPDATE:", then the message and the fix, and then "Tool "status" returns the fix."
  - `tools/list` lists only `status`. Its description carries the same text ("WAYMARK NEEDS AN UPDATE: ... Call this tool for the fix."), up to 600 characters. A tool description is in the context of the agent from the start, so the agent knows without a call.
  - Every other tool fails with `setup_problem`, the message, and a `next` sentence. For an argument of the entry the `next` sentence names `brew upgrade waymark` (if the entry was made for a newer waymark), and the two commands that replace the entry of Claude Code: `claude mcp remove waymark`, then `claude mcp add --transport stdio --scope user waymark -- waymark <the arguments that remain>`. Then it says to restart the agent tool.
  - `status` works. It has a failing check `entry` with the message and the fix, and `healthy` is false.
  - The error carries `data.update_needed: true`, so that code can tell this state from other setup problems.
- **What cannot be helped.** If the binary is missing or crashes before `initialize`, no message is possible. The agent tool shows its own error then.
- **The three channels.** The text at the start (`instructions`), the description of `status` (always in the context), and the error of every tool call. A client may not show the first, so the other two exist.

### Technology

| Need | Choice |
|---|---|
| Language | Rust |
| MCP | `rmcp` 3.x with `tokio` (checked at build time) |
| YAML frontmatter | `yaml-rust2`, without `serde` (see `document-format`) |
| Settings files | `toml` with `serde` |
| Git | the `git` binary (see `git-sync`) |

### Conformance tests

A fixture repository, a tool call, and the expected result fix the behavior, so the server can be rebuilt or ported. The tests call the tools through an in-memory MCP client, in both transports. They run in CI.

## Acceptance Criteria

- **Given** an argument in the entry that this waymark does not know, **When** the session starts, **Then** the server starts, `instructions` start the problem with "WAYMARK NEEDS AN UPDATE", `tools/list` lists only `status` with the problem in its description, and every other tool fails with `setup_problem` and the fix.
- **Given** a wrong value of a known argument, **When** the session starts, **Then** the same state holds and no memory repository is guessed.
- **Given** a deprecated argument, **When** a tool runs, **Then** it works and the result has a notice.
- **Given** a settings file of a higher schema, **When** the session starts, **Then** the same state holds with the message "needs a newer waymark".
- **Given** a working setup, **When** the session starts, **Then** there is no warning and all tools are listed.

- **Given** a started server, **When** the client lists tools, **Then** the list holds the tools above that the limits allow, and no control tool.
- **Given** any tool call, **When** it ends, **Then** the result has `message` and `settings`, and `notices` when there are any.
- **Given** a client that asks for a tool that changes limits, **When** it calls it, **Then** the server reports an unknown tool.
- **Given** the same call twice, **When** the second ends, **Then** the repository state is the same as after the first.
- **Given** the fixture suite, **When** it runs on a new build, **Then** all expected results match, in stdio and in HTTPS.
- **Given** the release binary, **When** it runs on a machine without Node, Python, or Go, **Then** it works. `git` is the only requirement.
- **Given** a rejected call, **When** the result returns, **Then** it has a `code` and, where a fix exists, a `next` sentence.
- **Given** an entry with `--repo` and a new URL, **When** the server starts, **Then** it clones the repository once and reuses the clone later.

## Open Questions

None. All answered on 2026-11-14 (see `decisions.md`).
