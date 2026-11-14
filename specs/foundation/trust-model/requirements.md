---
type: foundation
status: implemented
created: 2026-11-08
updated: 2026-11-14
---

# Trust model

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The server config layer, roles, and separate connections are out of scope for now. Parts that need server mode stay in this text as a record, and they are not implemented now (see [project decisions](../../../docs/decisions.md)).

## Technical Goal

Define who is responsible for what agents write, which limits the server enforces, who may set them, and how readers judge a document. waymark works like a company wiki that AI agents execute: the person is responsible, and the crowd fixes mistakes. The server enforces limits. It does not decide what an agent should do. That is the choice of the team, written in its own `AGENTS.md`.

## Dependents

- `mcp-server` — every tool call checks the effective limits.
- `write-approval`, `pause-and-resume`, `project-binding`, `web-console` — enforce parts of this model.
- `server-mode`, `multi-tenancy` — add the operator layer and roles.
- `verification` — the optional owner check builds on it.

## Deliverables

- The wiki model and the rules of authorship.
- The list of limits and their meaning.
- The config layers and the rule that the strictest layer wins.
- The rules for disputed documents.
- A list of the honest limits of the model.

## Scope

**In scope:**
- Responsibility, authorship, limits, config layers, disputes, confidentiality.

**Out of scope:**
- Advice on behavior: when an agent reads or writes, and what deserves a document. This is the choice of each team, in its `AGENTS.md`.
- A pull request gate where a second person approves a write. This is an idea for after the first release.
- Hard enforcement against a person or agent who bypasses waymark.
- Users and roles of a shared server (see `server-mode`).
- Separate MCP connections for each permission level. This is an option for server mode only (see `server-mode`).
- Rules for each folder with their own level. A global `access` and the `folders` list are enough for the first release.

## Constraints

- Every person needs an account on the Git host anyway. Commits are authored with the name and email of the person who used the agent. There is no generic bot author and no agent attribution.
- Users need no Git knowledge.
- Every agent action happens in a session that a person starts, or through a person's login on a server. There are no background agents.
- The server holds no behavior rules, only limits.

### Wiki model

- The person whose agent writes a document is responsible for it. History and blame show who changed what.
- Wrong documents are fixed like in a wiki: another agent or person changes them, or marks them `disputed` with a reason.
- Readers treat a document as a wiki page: use it, name the author and date, and check important facts against the code.
- Owner verification is optional. A knowledge base turns it on in `.waymark/settings.toml`. It is off by default.

### Limits

A limit is the most that the server ever allows. A limit is not advice.

**Reading is always allowed.** The limits do not switch reading on or off. They set where an agent may read (`folders`) and how much (`max_results`, `max_tokens`, `max_documents_opened`). Whether an agent reads on its own or only when the person asks is not a server limit. The server cannot tell the two apart. It is the choice of the team (its `AGENTS.md`) and of the agent tool (its permission prompts).

**Writing has three states:** not allowed (`access = "read-only"` or `paused`), allowed after a yes (`approval.required_for`), and allowed (the default). These are the only "levels" of waymark. They are not a separate setting. They follow from the limits below.

| Limit | Values | Built-in default |
|---|---|---|
| `access` | `read-only` or `read-write` | `read-write` |
| `paused` | `true` or `false`. `true` acts like `read-only`, set live. | `false` |
| `tools_denied` | List of tool names that are switched off, for example `["delete", "move"]` | empty |
| `folders` | List of folder globs that tools may touch. Empty means all. | empty |
| `max_results` | Most documents that `search` returns per call | 5 |
| `max_tokens` | Most tokens that `search` returns per call | 800 |
| `max_documents_opened` | Most documents that `show` opens in one session | 20 |
| `allowed_repo_hosts` | List of Git hosts that the server may clone from. Empty means any. | empty |
| `approval.required_for` | Tool names, or `write`, that need approval before they run (see `write-approval`) | empty |
| `approval.via` | `elicitation` or `console` | `elicitation` |

The document limits (`documents.max_bytes`, `documents.title_max_words`, `documents.description_max_words`) and the secret patterns are shared settings of the knowledge base (see `memory-repository`).

### Config layers

Limits come from layers. All layers use the same keys.

| Layer | Set by | Read | Strength |
|---|---|---|---|
| System config file (`/Library/Application Support/waymark/config.toml` on macOS, `/etc/waymark/config.toml` on Linux). Optional. | IT. The file belongs to an administrator. | at every call | **Hard in local mode**, where a company needs it. A normal user and their agent cannot change it. Most people do not use it. |
| Server config | The operator of a server (see `server-mode`) | at every call | **Hard in server mode.** Users cannot touch it. |
| Shared settings in the knowledge base (`.waymark/settings.toml`, section `[limits]`) | The team owner | at every sync | The server refuses to change this file through its tools. Two exceptions come from code in the binary and never from input: the automatic repairs and, later, `migrate` (see `memory-repository`, section "Versions and updates"). A person with Git access can still change it. |
| Personal config (`~/.config/waymark/config.toml`) | The person | at every call | A safety default. An agent with file access could edit it. |
| MCP entry (arguments and environment variables) | The person | at start | A safety default. |

**The strictest layer wins.** A layer can tighten a limit and never loosen it. A layer that sets nothing has no say. Per limit: `access` takes `read-only` if any layer says so. `paused` is true if any layer says so. `tools_denied` and `approval.required_for` are the union. `approval.via` takes `console` if any layer says so. `allowed_repo_hosts` is the intersection of the layers that set it. `folders` is the intersection of the layers that set it. The numeric caps take the smallest value.

In server mode, the client config is not a layer, because the user controls it. Limits per user or role come from the server (see `server-mode`).

Only the person or the operator changes limits. No tool changes them.

**Local mode is a safety default.** In local mode, the simple config is the gate: the MCP entry and the shared file. It guards against mistakes. A person, or an agent that can edit files, can change it. A hard limit needs a file that someone else owns (the system file) or a sign-in (server mode). Hard permission levels are a server-mode topic. See `server-mode` for the sign-in grants and for the option of separate connections for read and write.

### Disputed documents

- A disputed document is not to be relied on. Readers mention the dispute and its reason.
- A document has no stale mark. If `updated` is old, a reader checks the facts that matter before relying on them and says how old the document is.
- The folder shows which team to ask.

### Limits of the model (honest)

- The server cannot prove that a human, not an agent, said yes. Only approval through the console is enforced. Approval through elicitation depends on the client.
- A layer under the person's control (personal config, MCP entry) guards against mistakes, not against a determined user or agent that can edit files.
- Nothing stops an agent that has a shell from running raw Git commands in the content repository. Only a rule in the team's `AGENTS.md` can forbid it.
- An approval can be given without reading. A person can click through.
- Direct writes to `main` need a branch protection setting that allows them.
- Once a document is pushed, assume every reader has it. Prevention is the only real safeguard.

## Acceptance Criteria

- **Given** an agent write, **When** it is committed, **Then** the commit author is the name and email of the person who used the agent.
- **Given** the shared setting `access = "read-only"` and the personal setting `read-write`, **When** a write tool runs, **Then** it fails with `access_denied`.
- **Given** two layers with `max_results` 5 and 3, **When** `search` runs, **Then** it returns at most 3 documents.
- **Given** an agent, **When** it lists tools, **Then** no tool changes a limit.
- **Given** `tools_denied = ["delete"]`, **When** the client lists tools, **Then** `delete` is not listed and a call fails with `access_denied`.
- **Given** a system config file with `access = "read-only"`, **When** the person's config says `read-write`, **Then** writes fail.
- **Given** `paused = true` in the personal config, **When** the next write tool runs, **Then** it fails with `paused`.
- **Given** no layer sets anything, **When** a tool runs, **Then** the built-in defaults apply.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`): the built-in default of `access` stays `read-write`. Whether agent tools support a managed MCP config is a research item in `todo.md`.
