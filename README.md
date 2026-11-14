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
your own Git repository. One program, an MCP server, that runs on your
computer or on a shared server that anyone can host.

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

- **Documents are small Markdown files in one Git repo per knowledge base:**
  for a company, a team, or just you. One topic
  per document, with a header (title, description, status, authors, dates, sources).
- **An open document standard.** The [MKBase Document Standard](standard/mkbase-1.0.md)
  defines one document: a small Markdown file with a short header and no fixed
  folders. It is part of this project, with
  [conformance fixtures](standard/conformance/) that any tool can use to check
  that it reads and writes the same documents. waymark implements it.
- **One interface: MCP.** Every agent tool that speaks MCP can search, read,
  write, move, and sync documents. There is no CLI for agents and no skill to
  install.
- **The server enforces, you advise.** The server enforces hard rules: format,
  size, secrets, broken links, duplicates, result caps, and limits. What your
  agent reads, when it writes, and what deserves a document is up to you. You
  write it in your own `AGENTS.md`. The examples below are suggestions.
- **Every write is made under the person's own Git account.** The person
  is responsible for what their agent writes. History and blame show who
  changed what. Wrong documents get fixed or marked `disputed` by the next agent that
  finds the error.
- **Limits are set by you or by IT, and no tool can loosen them.** Access
  (read-only or read-write), pause, switched-off tools, allowed folders, and
  approval rules. The strictest layer wins.
- **Nobody needs to know Git.** waymark does all Git work. Git keeps its own
  login, and waymark never sees a secret.
- **Works with several agent tools.** Claude Code, GitHub Copilot, and
  OpenAI Codex from day one.

## Why waymark

Your agents forget everything between sessions. Your teams don't write
things down. waymark fixes both: agents write it down, and every agent reads
it back, in your company, in your team, or across all the AI tools you use
yourself.

### Knowledge that agents can actually use

- **Cheap to read, by design.** One topic per document. Every document has a
  short title and description, so an agent sees them first and opens the
  body only when it is worth it. A hard size cap is enforced on every
  write.
- **A read budget, enforced.** The server returns at most a few documents per
  search and limits how many documents a session opens. The agent cannot read
  more than the limit, so context stays small.
- **Fast search.** Documents are local files, so search takes milliseconds.
  No API calls to a wiki, no rate limits.
- **Linked knowledge.** Documents link to related documents, and a new decision
  marks the old one it replaces. Agents follow the trail to the current
  state.

### Knowledge that stays clean

- **Rejections teach.** A bad write fails with a plain reason and the next
  step, for example "A document with this title exists. Update it instead."
- **Noise is blocked by code.** Duplicates, oversize documents, secrets, and
  broken links never reach the repository.
- **Wrong documents get fixed.** Any agent can update a document or mark it
  `disputed` when its source says otherwise. A new document can replace an old
  one, and the old one is marked deprecated.
- **Facts come with the answer.** A result shows the age of a document and its
  dispute reason, so the agent can pass them on.

### Trust without a bottleneck

- **Every change has a name.** Every change is made under the Git account
  of the person whose agent wrote it. Blame shows who changed what.
- **Limits you can rely on.** In local mode, a personal file and the MCP entry
  guard against mistakes. A system file that IT owns, or a server, gives limits that a
  user cannot change.
- **Approval where you need it.** Choose which tools need a yes before they run.
  Your agent tool can ask you, and on a server, a web console gives an approval
  that the agent cannot answer.
- **Undo.** List what your agent wrote and revert any write. History stays.
- **One switch to pause.** Stop all writes at once.
- **Notices.** Results carry short notices, for example when someone disputes your document.

### Local or shared

- **Local mode.** The agent tool starts waymark for one session. No server,
  no database, no app. Add one entry to your MCP config.
- **Server mode.** The same binary runs as a shared MCP server over HTTPS, with
  single sign-on, roles, and an audit log, for one organization. Chat apps such
  as Claude and ChatGPT are target clients (their login flows are still to be
  verified). Anyone can host it, for example in a container on your own
  infrastructure.
- **Many organizations.** A later stage serves separate organizations from one
  server, with strict isolation.

### Yours, forever

- **Plain Markdown in Git.** Read it on GitHub, in any editor, or with any
  tool. No database, no proprietary format.
- **100% Obsidian compatible.** Your knowledge base is also an Obsidian
  vault. Open it in Obsidian to browse documents, follow links, and explore
  the graph. No plugin, no export.
- **No vendor lock-in.** The content is a normal Git repo that you own. A server
  holds only a cache of it.
- **Your memory, not your AI vendor's.** Your AI tools remember what you
  taught them, across tools, in a repo that you own. Switch tools any time.
- **Generic and open source.** Built for any company, team, or
  person. MIT licensed.

## Planned setup

1. Install the `waymark` binary. `git` must work and be signed in to your
   Git host (SSH key or `gh auth login`). IT can prepare this.
2. Add one entry to the MCP config of your agent tool.
3. Start a session. waymark clones the knowledge base on first start.
4. A team adds the same entry to the project config, so everyone uses the same
   knowledge base.

```json
{
  "mcpServers": {
    "waymark": {
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/knowledge.git"]
    }
  }
}
```

Entries for Claude Code, GitHub Copilot, and OpenAI Codex are in the
[agent tool setup spec](specs/features/agent-tool-setup/requirements.md). They
are verified against the tool versions before the first release.

The first person of a company or team, or a person on their own, creates an
empty private repository and calls the `init` tool once.

## Tools for agents

| Tool | What it does |
|---|---|
| `status` | The knowledge base in use, the effective limits, health checks, and notices. |
| `search` | Find the few documents that answer a question. |
| `show` | Read one document, with the documents that link to it. |
| `list`, `stats` | List documents by filter, and count their size. |
| `updates` | What changed since a date, in some folders or all. |
| `add` | Write a new document. |
| `update_document` | Change a document. |
| `dispute` | Mark a document as wrong, with a reason, or clear the mark. |
| `deprecate` | Retire a document that is no longer true. |
| `delete` | Delete a document. A team can switch it off. Links to it are fixed. |
| `move` | Move or rename a document or a folder. Links are fixed. |
| `activity`, `undo` | See what your agent wrote and revert a write. |
| `sync` | Commit manual edits, pull, and push. |
| `lint`, `index` | Check documents, and generate index files. |
| `init` | Create the structure of an empty knowledge base repository. |

A limit can switch a tool off, and then it is not listed. There is no tool that
changes limits, pause, approval, or credentials. The full list is in the
[MCP server spec](specs/foundation/mcp-server/requirements.md).

## Project and default entry

One waymark process serves one knowledge base, and no tool can change it. A
project shares its knowledge base through the MCP entry in the project (for
example `.mcp.json`), which holds the repository URL and no secret:

```json
{ "mcpServers": { "waymark": { "command": "waymark", "args": ["--repo", "git@github.com:example/knowledge.git", "--kb-id", "<id from .mkbase/waymark.toml>"] } } }
```

A personal default in the user-level config of your agent tool needs no
arguments if the knowledge base is in `~/Projects/<name>-knowledge-base`:

```json
{ "mcpServers": { "waymark": { "command": "waymark" } } }
```

Use the same server name in both. In the agent tools that were checked, the
project entry wins in the project. waymark finds the folder in this order:
`--path`; `--repo` (cloned once to `~/Projects/<name>-knowledge-base`); the
working folder and the folders above it; the one knowledge base in
`~/Projects`. `--kb-id` makes the server refuse a repository with another ID.
`allowed_repo_hosts` in your personal config limits the Git hosts waymark may
use. A large first clone can exceed the start time-out of your tool: clone it by
hand to the default folder first.

## Pause writes

To stop all writes at once, add this to `~/.config/waymark/config.toml`:

```toml
[limits]
paused = true
```

The agent can still read. Write tools disappear from the tool list, and a direct
write call fails with `paused`. The change works at the next call, with no
restart. There is no timer. To resume, set `paused = false` (or remove the
line) in the file that set it. A lower file cannot undo a pause from a higher
one. You can also start the server with `--paused`. An agent cannot pause or
resume: no tool does it. `access = "read-only"` wins over a pause.

## Approval

An owner or a person can ask for a human yes before some writes run. In any
config file:

```toml
[approval]
required_for = ["delete", "update_document"]   # or ["write"] for every write tool
```

waymark then asks you in the agent tool (MCP elicitation) after all checks
passed and before the commit. The question shows the tool, the author, and the
full text. You tick "Approve this write" and accept. Decline, cancel, a box that
is not ticked, or no answer in 2 minutes means nothing is written
(`approval_denied`). A client that cannot show the question gets
`approval_unavailable`, and nothing is written. A `preview` never asks.

This guards against mistakes. A client hook can answer the question
automatically, so it does not stop a deliberate bypass. Which clients show the
question is checked by the manual tests of the spec. The other way to get a yes
is the permission prompt of your agent tool: set it to ask for the write tools
(for example `mcp__waymark__add` in Claude Code).

## Your rules: `AGENTS.md`

waymark ships no behavior text. You decide how your agent uses the tools, in
your own `AGENTS.md`. Three suggestions to copy and change. They are only
suggestions. waymark does not insert or check them.

**Manual: use waymark only when asked**

```text
Use the waymark tools only when the person asks.
Search first, then open at most 3 documents.
Write only on request. Never write secrets, tokens, or personal data.
Name the author and date when you use a document.
```

**Balanced: read alone, write on request**

```text
At the start of a task, search waymark for the topic and open at most 3 documents.
If a document is old or marked disputed, say so and check the facts.
Write only when the person asks, or after you ask "Save this?" and they say yes.
One topic per document, with a one-line description. Update before you add.
Never write secrets, tokens, or personal data.
```

**Proactive: the agent decides, undo is the safety net**

```text
Search waymark before you start work. Save decisions, pitfalls, and contracts that
the next agent needs. Do not save what the code or the history already shows.
Update a document that exists instead of adding a copy. If a document is wrong,
mark it disputed with a reason. Tell the person in one line what you saved.
Never write secrets, tokens, or personal data.
```

## Limits

The server enforces limits. A limit can only tighten, and the strictest layer
wins.

| Layer | Who | Strength |
|---|---|---|
| System file (`/etc/waymark/config.toml` or the macOS equivalent) | IT | Hard in local mode |
| Server config | The operator | Hard in server mode |
| Shared file in the knowledge base (`.mkbase/waymark.toml`) | The team owner | Tools cannot change it |
| Personal file (`~/.config/waymark/config.toml`) | You | A safety default |
| MCP entry | You | A safety default |

## Access without waymark: the plain-Git rules

Every knowledge base is a Git repository with an `AGENTS.md` file in its
root. The `init` tool writes this file. It contains plain-Git rules: how an
agent reads, writes, edits, and moves documents with ordinary `git` and file
commands. It names the document format, the pull-before-write step, the check
against overwriting a newer version, the commit author, and the ban on secrets.

The file has two modes. If the agent has the waymark MCP tools, it uses them.
If not, it follows the plain-Git rules.

**Advantages**

- **No install.** Open the repository in Claude Code on the web or on
  iPhone. The agent reads the whole knowledge base and the rules at once.
  No binary, no MCP connector, no server.
- **Safe by default.** An agent that finds the repository by chance
  still sees clear rules. It is less likely to damage the documents.
- **Costs nothing.** It is one text file in the repository.

**Disadvantages**

- **Only a text rule.** Without the MCP server, the agent can ignore the rules.
  There is no `lint`, no approval, no pause switch, no limits, and no email check.
- **More room for mistakes.** The agent does the file and link work
  itself, for example the link rewrite when it moves a document.
- **Not the main path.** The plain-Git rules are a second access channel. Moves use `git mv`, so a document keeps its history.

## How waymark compares

Three ways to give an agent shared knowledge: **waymark** (an MCP server),
**plain Git** (only the plain-Git rules in `AGENTS.md`, no waymark install), and
**Confluence** with an MCP server.
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
| Limits, approval, and a pause switch | ✅ | ❌ | ❌ |
| Undo of agent writes | ✅ | ⚠️ | ⚠️ |
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

Where the ⚠️ appears for waymark: on web and iPhone, an agent can use a
shared waymark server over MCP, or, without a server, fall back to the
standalone plain-Git skill. It works on any Git repo of Markdown documents, with
or without waymark. The plain-Git
mode is a good second channel and a safe default.

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
- [MKBase Document Standard](standard/mkbase-1.0.md): the document format, with [conformance fixtures](standard/conformance/).
- [Specifications](specs/INDEX.md): all specs and their status.
- [Roadmap](docs/roadmap.md): phases from design to public release.
- [Ideas](docs/ideas.md): features for after the first release.
- [Similar projects](docs/similar-projects.md): what exists and what we learn from it.
- [Project decisions](docs/decisions.md) and [open questions](docs/open-questions.md).
- [How this project is built](specs/WORKFLOW.md).

## Development

waymark is built on macOS with Apple silicon. On a new Mac, clone the repository and run:

```bash
make setup
make check
```

`make help` lists every command: build, test, lint, run, and release preparation. The work follows the spec-driven process in [specs/WORKFLOW.md](specs/WORKFLOW.md).

## License

[MIT](LICENSE).
