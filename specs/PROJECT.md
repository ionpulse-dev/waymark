# Waymark — Product Overview

## Vision

Waymark is long-term agent memory: memory that outlives the short context
window of an agent. It serves companies, teams, and private people. The memory
is plain Markdown in a Git repository that the owner controls. Unlike the
memory of an AI vendor, it is yours, it is shared between all your AI tools,
and it has a history and an undo. What it holds is up to the owner: a team
memory repository, the personal memory of one person, or what a long-running agent
leaves for its next session.

Waymark is one program: an MCP server. Agent tools use it through MCP, on the
person's own computer or on a shared server that anyone can host.

A **waymark** is a sign on a trail. Each agent leaves a short mark of what it
learned, so the next agent does not get lost on the same path.

## Personas

### Developer

Works in a code repository with a coding agent such as Claude Code, GitHub
Copilot, or OpenAI Codex. Wants the agent to know decisions, contracts, and
known problems that live outside the repository. Success: the agent finds the
right document in seconds and uses little context.

### Team member without Git knowledge

Uses an AI agent but does not know Git. Wants to read and add documents
without ever seeing a branch or a merge conflict. IT prepares the Git login.
Success: adds one MCP entry, and everything else works. On a shared server, the
person signs in and approves writes in a web page.

### Private person

Wants one memory for all their AI tools, in a repository they own. Success:
switching the AI tool does not lose what the agents learned.

### Memory owner

The first person of a company or team. Creates the memory repository and sets the
shared limits. Success: new people join in a few minutes and the memory
stays clean.

### Operator

Runs a shared server for an organization, or for many. Sets the sign-in, the Git
credentials, and the limits that users cannot change.

_A single user may play more than one persona._

## Core Concepts

### Memory repository

One Git repository with Markdown documents. It is the container of the memory.
It is separate from the code repository of this project. It holds shared
settings in a `.waymark/settings.toml` file and an `AGENTS.md` with plain-Git
rules for agents that cannot use the MCP server. Every memory repository is a
valid Obsidian vault. A team can use one as its memory repository, a person as a
personal memory, or an agent as its working memory (see the
[terminology spec](foundation/terminology/requirements.md)).

### Document

One topic in one Markdown file with flat YAML frontmatter, as defined by the
[Waymark Memory Standard](../standard/waymark-1.0.md). It has a title, a
required short description, a status, authors, dates, and links to
other documents. The ID of a document is its path without `.md`. A document is
either active or deprecated. It can be marked as disputed with a reason. It
can be replaced by a newer document or deleted when the person asks.

### Folder

Folders are the teams or areas of an organization. The folder of a document is
its owner. The organization builds its own folder tree. Waymark has no built-in
folders.

### MCP server

The `waymark` binary. It is the only interface for agents. Its tools search,
read, write, move, and sync documents. It does all Git work and enforces the
rules: format, size, links, secrets, and limits. Tool descriptions are the only
built-in guidance.

### Behavior

What an agent reads, when it writes, and what deserves a document is the choice
of each team. Teams write it in their own `AGENTS.md`. The README gives example
snippets. Waymark ships no skill.

### Limits

The most that the server allows: access (read-only or read-write), pause,
switched-off tools, folders, result caps, and approval rules. They come from
layers (system file, server config, shared file, personal file, MCP entry). The
strictest layer wins.

### Local mode and server mode

The same binary runs in two modes. In local mode, the agent tool starts it over
stdio for one session. In server mode, it runs as a shared MCP server over HTTPS
with sign-in, roles, and an audit log, for one organization or many. Anyone can
host it.

### Web console

A web page of server mode. People approve writes, see what their agents wrote,
undo it, review disputes, and pause writes.

### Project binding

A project uses one memory repository through a project-level MCP entry that names
the repository. One server process serves exactly one memory repository.

The long-term goal is a building block for a software factory, where agents do more of the delivery work with the right context.

## Design Principles

- **You own the memory** — plain Markdown in your own Git repository. No
  database, no vendor lock-in.
- **Local first, server optional** — local mode needs no server. Server mode is
  the same code, hosted by anyone.
- **MCP is the only interface** — one interface for every agent tool and chat
  app.
- **The server enforces, the team advises** — hard rules are limits in code.
  Behavior is the choice of each team.
- **The person is responsible** — every write is committed under the Git
  account of the person whose agent wrote it, as in a wiki.
- **Limits belong to the person and the operator** — the server enforces them,
  and no tool can loosen them. The strictest layer wins.
- **Cheap to read** — one topic per document, a short description, a hard size
  cap, and a small read budget.
- **No Git knowledge needed** — Waymark runs all Git commands.
- **Secrets stay with Git** — Waymark never stores or passes a Git secret.
- **Obsidian compatible** — everything Waymark writes works in Obsidian
  without changes.
- **Spec driven** — the specification drives the code, never the other way
  around.

## Scope

**In scope:**
- The memory repository format and its rules: the Waymark Memory Standard in
  `standard/`, with its conformance fixtures.
- The `waymark` binary (Rust): the MCP server, local mode and server mode.
- Sign-in, roles, Git credentials, tenancy, and the audit log of server mode.
- The web console of server mode.
- Documentation: MCP entries for Claude Code, GitHub Copilot, and OpenAI Codex,
  and example `AGENTS.md` snippets.
- Quality checks, sync, and undo.

**Out of scope:**
- A skill or any installer that writes into agent tools.
- A desktop app.
- Background or cloud agents. Every agent action happens in a session that a
  person starts or a person's login on a server.
- Binary data in the memory repository. A document describes it and links to it.
- A wiki backend such as Confluence. Storage is Markdown in Git only.
- Operating a service. The project ships the software and a deploy guide. Who
  runs a server is not decided here.

**Standalone parts:** the Waymark Memory Standard is part of this project
(`standard/`). The plain-Git rules for agents without MCP are a standalone skill.
Its home after `waymark-trail` is retired is open (see the open questions).

## Post-MVP

_Capabilities intentionally deferred. No specs exist yet — this section is a reminder only._

- `connect-git-account` — a console page where an owner connects a Git account, with tokens stored encrypted.
- `memory-browser` — a window to search and read documents without an agent.
- `pull-request-gate` — approval of a write by a second person.
- `anchor-check` — flag documents whose anchored code changed.
- `message-channels` — email or webhook messages from a server.
- `device-login` — OAuth device flow instead of the Git login of the person.
- `company-rollout` — deploy the binary and the system config file through device management.
- `waymark-doctor` — a small command for people without an agent.

## Further reading

- [Architecture direction](../docs/architecture-direction.md): why MCP only, and the two modes.
- [Roadmap](../docs/next-steps/roadmap.md): phases from design to public release.
- [Ideas](../docs/ideas.md): the details of the post-MVP list.
- [Similar projects](../docs/similar-projects.md): what exists and what we learn from it.
- [Project decisions](../docs/decisions.md) and [open questions](../docs/next-steps/open-questions.md).
