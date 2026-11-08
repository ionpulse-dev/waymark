# Waymark — Product Overview

## Vision

Waymark is an automated, AI-driven knowledge base and memory for AI agents
and the people who use them. It serves companies, teams, and private
people. The knowledge is plain Markdown in a Git repository that the owner
controls. It runs locally and needs no server. Unlike the memory of an AI
vendor, the memory is yours, it is shared between all your AI tools, and it
has a history and an undo.

A **waymark** is a sign on a trail. Each agent leaves a short mark of what it
learned, so the next agent does not get lost on the same path.

## Personas

### Developer

Works in a code repository with a coding agent such as Claude Code, GitHub
Copilot, or OpenAI Codex. Wants the agent to know decisions, contracts, and
known problems that live outside the repository. Success: the agent finds the
right document in seconds and uses little context.

### Manager or other non-developer

Uses an AI agent but does not know Git. Wants to read and add knowledge
without ever seeing a branch or a merge conflict. Success: installs the app,
answers a few questions, and everything else works.

### Private person

Wants one memory for all their AI tools, in a repository they own. Success:
switching the AI tool does not lose what the agents learned.

### Knowledge base owner

The first person of a company or team. Creates the knowledge base and sets the
shared settings, such as the loosest level of autonomy that is allowed.
Success: new people join in a few minutes and the knowledge base stays clean.

_A single user may play more than one persona._

## Core Concepts

### Knowledge base

One Git repository with Markdown documents. It has its own repository (the
content repository), separate from this project. It holds shared settings in a
`waymark.toml` file and an `AGENTS.md` with rules for agents. Every
knowledge base is a valid Obsidian vault.

### Document

One topic in one Markdown file with flat YAML frontmatter. It has a title, a
required short description, a type, a status, authors, dates, and links to
other documents. The ID of a document is its path without `.md`. A document is
either active or deprecated. It can be marked as disputed with a reason. It
becomes stale when nobody changes it for a configured number of days.

### Folder

Folders are the teams or areas of an organization. The folder of a document is
its owner. The organization builds its own folder tree. Waymark has no built-in
folders.

### Skill

The rules of behavior for an AI agent: when to read, when to write, what is
worth a document, and how to write it. It is compiled into the CLI and
written into each agent tool.

### CLI

The `waymark` command. Agents use it to search, read, write, and sync. It does
all Git work and enforces the rules: format, size, links, and secrets.

### App

A small macOS menu bar app for people: setup, approval dialogs, notifications,
a list of what the agent wrote with undo, and background sync.

### Read level and write level

Settings of a person that control how much the agent does alone. Read level:
`auto` or `manual`. Write level: `auto`, `confirm`, or `manual`. The defaults
are `manual`: the agent reads and writes only when asked.

### Project binding

A project folder is bound to exactly one knowledge base by an id in a hidden
`.waymark/` folder. A session in that folder uses only that knowledge base.

The long-term goal is a building block for a software factory, where agents do more of the delivery work with the right context.

## Design Principles

- **You own the memory** — plain Markdown in your own Git repository. No
  database, no vendor lock-in.
- **Local, no server** — everything runs on the person's computer. The Git host
  is the only shared part.
- **The agent is the interface** — people talk to their agent. The app and the
  CLI support this and never replace it.
- **The person is responsible** — every write is committed under the Git
  account of the person whose agent wrote it, as in a wiki.
- **Control stays with the person** — the defaults are the strictest. People
  loosen them later.
- **Cheap to read** — one topic per document, a short description, a hard size
  cap, and a small read budget.
- **No Git knowledge needed** — Waymark runs all Git commands.
- **Obsidian compatible** — everything Waymark writes works in Obsidian
  without changes.
- **Spec driven** — the specification drives the code, never the other way
  around.

## Scope

**In scope:**
- The knowledge base format and its rules.
- The `waymark` CLI for agents (Rust).
- The macOS menu bar app (Swift).
- The skill and the instruction blocks for Claude Code, GitHub Copilot, and
  OpenAI Codex.
- Setup, sync, quality checks, and undo.

**Out of scope:**
- Servers that the project operates.
- Cloud or background agents. Every agent action happens in a session that a
  person starts.
- Binary data in the knowledge base. A document describes it and links to it.
- A wiki backend such as Confluence. Storage is Markdown in Git only.

**Related project:** the plain-Git rules for agents without the CLI are a
separate, standalone project, `waymark-trail`.

## Post-MVP

_Capabilities intentionally deferred. No specs exist yet — this section is a reminder only._

- `mcp-server` — a thin MCP adapter over the CLI, for Claude and ChatGPT connectors.
- `mobile-companion` — an iPhone app for approval and review only.
- `knowledge-browser` — a window to search and read documents without an agent.
- `pull-request-gate` — approval of a write by a second person.
- `anchor-check` — flag documents whose anchored code changed.
- `device-login` — OAuth device flow instead of a pasted token.
- `company-rollout` — deploy the app with a pre-filled configuration through device management.

## Further reading

- [Roadmap](../docs/roadmap.md): phases from design to public release.
- [Ideas](../docs/ideas.md): the details of the post-MVP list.
- [Similar projects](../docs/similar-projects.md): what exists and what we learn from it.
- [Project decisions](../docs/decisions.md) and [open questions](../docs/open-questions.md).
