<p align="center">
  <img src="banner.svg" alt="waymark" width="800"/>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
  &nbsp;
  <img src="https://img.shields.io/badge/plain-Markdown-brightgreen" alt="Plain Markdown"/>
  &nbsp;
  <img src="https://img.shields.io/badge/works%20with-any%20AI%20agent-blueviolet" alt="Works with any AI agent"/>
</p>

<br/>


An automated, AI-driven knowledge base and memory for AI agents and the
people who use them. For companies, teams, and you alone. Plain Markdown in
your own Git repository. Runs locally. No server.

The specification is in [specs/](specs/). This project is built spec first, with
the workflow in [specs/WORKFLOW.md](specs/WORKFLOW.md).

## Why the name

A **waymark** is a sign on a trail, such as a painted blaze on a tree or a
small stack of stones. A hiker leaves it so the next one finds the way.

waymark does the same for your agents. Each agent leaves a short mark of
what it learned: a decision, a contract, a pitfall. The next agent, in any
team, finds it and does not get lost on the same path.

## The problem

Teams share a lot of context: decisions, API contracts, owners, known
problems, conventions. This context lives in meetings, chats, tickets, and
documentation pages. An AI agent that works in a code repository does not see it.
Each agent session starts without the cross-team context.

The same is true for one person. Each AI tool has its own memory, or
none. That memory sits with the AI vendor, not with you, and your other
tools never see it.

## The idea

waymark is an automated, AI-driven knowledge base. AI agents write it,
AI agents read it, and AI agents keep it up to date, as far as you allow.
By default, an agent reads and writes only when you ask. People stay in
control.

- **Documents are small Markdown files in one Git repo per knowledge base:**
  for a company, a team, or just you. One topic
  per document, with a header (title, description, status, authors, dates, sources).
- **Agents can write as a side effect of normal work, when you allow it.**
  A skill tells the agent what is worth keeping, when to write it, and how
  to keep it short.
- **Every write is made under the person's own Git account.** The person
  is responsible for what their agent writes. History and blame show who
  changed what. Wrong documents get fixed or marked `disputed` by the next agent that
  finds the error.
- **Each person decides how much their agent may do alone.** You choose a
  read level (`auto` or `manual`) and a write level (`auto`, `confirm`, or
  `manual`). Both start at `manual`: the agent reads and writes only when
  you ask.
- **Nobody needs to know Git.** The `waymark` CLI does all Git work.
  Setup installs Git for you if it is missing.
- **Works with several agent tools.** Claude Code, GitHub Copilot, and
  OpenAI Codex from day one.

## Why waymark

Your agents forget everything between sessions. Your teams don't write
things down. waymark fixes both at once: the agents write it down for you,
and every agent reads it back: in your company, in your team, or across
all the AI tools you use yourself.

### Knowledge that agents can actually use

- **Cheap to read, by design.** One topic per document. Every document has a
  short title and description, so an agent sees them first and opens the
  body only when it is worth it. A hard size cap is enforced on every
  write.
- **Fast search.** Documents are local files, so search takes milliseconds.
  No API calls, no rate limits, no waiting for a server.
- **Small context, better answers.** An agent loads a few short documents,
  not a stack of long pages. More room is left for your code.
- **Linked knowledge.** Documents link to related documents, and a new decision
  marks the old one it replaces. Agents follow the trail to the current
  state.

### Knowledge that maintains itself

- **Captured during normal work.** When an agent learns something that
  other teams need, it writes a document. Nobody has to stop and "do
  documentation".
- **Noise stays out.** When you ask for a document, the agent writes it. When the
  agent decides alone, it keeps only facts that are still true in three months
  and useful beyond the conversation. Status updates and chat summaries do not
  belong in the knowledge base.
- **Wrong documents get fixed.** Any agent can update a document or mark it
  `disputed` when its source says otherwise. A new document can replace an old
  one, and the old one is marked deprecated.
- **Always up to date.** waymark syncs in the background every few
  minutes. Every agent reads the latest state.

### Trust without a bottleneck

- **Every change has a name.** Every change is made under the Git account
  of the person whose agent wrote it. Blame shows who changed what.
- **You set the autonomy.** Let your agent write freely, let it ask
  before it writes, or write only when you ask. You can confirm in the
  chat or in an app dialog. Your choice, within the limit set in the
  shared settings.
- **One click to undo.** The app lists everything your agent wrote. Undo
  any write with one click.
- **You hear about problems.** A macOS notification tells you when someone
  disputes your document or when sync fails. One review list shows what needs
  your attention.
- **Safe by default.** Every write is checked for format, size, broken
  links, and secrets before it is committed.

### Zero friction to join

- **One native Mac app.** No Node, no Python, no Docker. Install it and
  run it. If Git is missing, setup installs it for you.
- **No terminal needed.** Drag the app to Applications. A setup window
  walks you through a few questions, each with a sensible default.
- **Lives in your menu bar.** No Dock icon, no window in your way. One
  icon shows whether everything is fine.
- **No Git knowledge needed.** waymark pulls, commits, and pushes for you.
  You never see a branch or a merge conflict. Managers can use it too.
- **Works with the agent you already use.** Claude Code, GitHub Copilot,
  and OpenAI Codex from day one. One skill for all of them.

### Yours, forever

- **Plain Markdown in Git.** Read it on GitHub, in any editor, or with any
  tool. No database, no proprietary format.
- **100% Obsidian compatible.** Your knowledge base is also an Obsidian
  vault. Open it in Obsidian to browse documents, follow links, and explore
  the graph. No plugin, no export.
- **No server to run.** Everything runs on your laptop. Your Git host is
  the only shared part, and you already have one.
- **No vendor lock-in.** The content is a normal Git repo that you own.
- **Your memory, not your AI vendor's.** Your AI tools remember what you
  taught them, across tools, in a repo that you own. Switch tools any time.
- **Generic and open source.** Built for any company, team, or
  person. MIT licensed.

## Planned setup

1. Download `Waymark.dmg` and drag the app to Applications.
2. Open it. A setup window asks a few questions: knowledge base, login,
   email, agent tools, read and write level. Each has a default. The first person of
   a company or team, or a person on their own, creates the knowledge base
   in the same window.
3. Link your project folders in the app, or set a default knowledge base.
4. Done. waymark lives in your menu bar and keeps the knowledge base up
   to date in the background.

Developers and IT can do the same in a terminal:

```bash
waymark init                       # create a knowledge base
waymark setup <content-repo-url>   # join one
```

Everything else, your agent does for you.

## Commands for agents

Your agent uses these commands through the waymark skill:

| Command | What it does |
|---|---|
| `waymark session` | Run first in every session: the settings, the knowledge base in use, and notices. |
| `waymark search <query>` | Find the few documents that answer a question. |
| `waymark show <id>` | Read one document, with the documents that link to it. |
| `waymark add` | Write a new document. |
| `waymark update-document <id>` | Change a document. |
| `waymark dispute <id>` | Mark a document as wrong, with a reason. |
| `waymark deprecate <id>` | Retire a document that is no longer true. |
| `waymark delete <id>` | Delete a document, only when you ask for it. Links to it are fixed. |
| `waymark move <old> <new>` | Move or rename a document or a folder. Links are fixed. |
| `waymark updates [<folder>...] [--since <date>]` | What changed since a date, in some folders or all. |

Every command takes `--kb <uuid>`, the id of the project's knowledge base. The commands that set up or check a machine (`init`, `setup`, `doctor`) need no id. The full list is in the
[CLI core spec](specs/foundation/cli-core/requirements.md), and the options of each command are in its feature spec.

## Access without waymark: the plain-Git rules

Every knowledge base is a Git repository with an `AGENTS.md` file in its
root. waymark writes this file when you create the knowledge base. It
always contains plain-Git rules: how an agent reads, writes, edits, and
moves documents with ordinary `git` and file commands. It names the document
format, the pull-before-write step, the check against overwriting a newer
version, the commit author, and the ban on secrets.

The file has two modes. If the agent has the `waymark` CLI, it uses the
CLI. If not, it follows the plain-Git rules.

**Advantages**

- **No install.** Open the repository in Claude Code on the web or on
  iPhone. The agent reads the whole knowledge base and the rules at once.
  No app, no CLI, no MCP connector, no server.
- **Safe by default.** An agent that finds the repository by chance
  still sees clear rules. It is less likely to damage the documents.
- **Costs nothing.** It is one text file in the repository, on by default.

**Disadvantages**

- **Only a text rule.** Without the CLI, the agent can ignore the rules.
  There is no `lint`, no approval dialog, no pause switch, no access mode
  per knowledge base, and no email check.
- **More room for mistakes.** The agent does the file and link work
  itself, for example the link rewrite when it moves a document.
- **Not the main path.** For daily work on your Mac, use the app and the
  CLI. The plain-Git rules are a second access channel. Moves use `git mv`, so a document keeps its history.

## How waymark compares

Three ways to give an agent shared knowledge: **waymark** (app, CLI, and
skill), the **plain Git** (only the plain-Git rules in `AGENTS.md`, no
waymark install), and **Confluence** with an MCP server.
✅ yes, ⚠️ partly or by rule only, ❌ no.

| Feature | waymark | Plain Git | Confluence + MCP |
|---|---|---|---|
| Format check before every write | ✅ | ❌ | ❌ |
| Hard size cap per document | ✅ | ❌ | ❌ |
| Secret check before every commit | ✅ | ❌ | ❌ |
| Read budget (few, small hits) | ✅ | ❌ | ❌ |
| Search by title and description first | ✅ | ❌ | ⚠️ |
| No silent overwrite (base-version check) | ✅ | ⚠️ | ❌ |
| Move with automatic link rewrite | ✅ | ⚠️ | ⚠️ |
| Dispute, deprecate, and replace | ✅ | ⚠️ | ❌ |
| "What changed since" for each person | ✅ | ⚠️ | ❌ |
| Reachable author on every document | ✅ | ⚠️ | ⚠️ |
| Approval dialog and pause switch | ✅ | ❌ | ❌ |
| One-click undo of agent writes | ✅ | ⚠️ | ⚠️ |
| Local files, fast reads | ✅ | ✅ | ❌ |
| Plain Markdown, any editor | ✅ | ✅ | ❌ |
| History per line (blame, diff) | ✅ | ✅ | ⚠️ |
| Opens as an Obsidian vault | ✅ | ✅ | ❌ |
| You own the content (plain Git repo) | ✅ | ✅ | ❌ |
| Free | ✅ | ✅ | ⚠️ |
| No install | ❌ | ✅ | ⚠️ |
| Works in Claude Code on web and iPhone | ⚠️ | ✅ | ⚠️ |
| Rich web editor for people | ❌ | ❌ | ✅ |
| Read permissions per page or folder | ❌ | ❌ | ✅ |

Where the ⚠️ appears for waymark: on web and iPhone, the agent has no CLI,
so it falls back to the plain-Git skill of the waymark-trail project. That skill is standalone and works on any Git repo of Markdown documents, with or without waymark. The plain-Git mode is a good
second channel and a safe default. It is not a replacement for the CLI.

Why not Confluence? Many companies use it, and some connect it to their
agents through an MCP server. It is a good place for people to read. It is
a weak place for agent knowledge: pages are long with no size limit, so
agents load too much and waste context. A hit in search is a whole page.
Every read is an API call, and a local cache gets out of sync fast. The
agent can write any text, with no format check and no secret scan. Admins
often restrict write access through the API. The content lives in a vendor
product, in its own page format.

Confluence and waymark can live side by side. A document can link to a
Confluence page as its source, and a Confluence page can link to a document.

## Documentation

- [Product overview](specs/PROJECT.md): vision, personas, core concepts, scope.
- [Specifications](specs/INDEX.md): all specs and their status.
- [Roadmap](docs/roadmap.md): phases from design to public release.
- [Ideas](docs/ideas.md): features for after the first release.
- [Similar projects](docs/similar-projects.md): what exists and what we learn from it.
- [Project decisions](docs/decisions.md) and [open questions](docs/open-questions.md).
- [How this project is built](specs/WORKFLOW.md).

## License

[MIT](LICENSE).
