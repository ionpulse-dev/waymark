# Similar projects

Research from 2026-11-06. Star counts are a snapshot.

**Result:** no mature project does what waymark plans. The close ones are
small and new. The big ones are single-user memory tools.

## Projects

| Project | What it is | Storage | Agent access | Trust and review | Maturity |
|---|---|---|---|---|---|
| [teamlore](https://github.com/lak7/teamlore) | Shared team memory for Claude Code | `.lore/` folder in each code repo, one note per file, max 120 words | Skill and hooks, Claude Code only | Agent only stages. A human merges a PR. | 74 stars, MIT, experimental |
| [project-cairn](https://github.com/iBlinkQ/project-cairn) | Skill for lessons, decisions and a log per project | Markdown in the project | Skill for Claude Code and Codex | User confirms before an item moves to long-term knowledge | 252 stars, MIT, active |
| [basic-memory](https://github.com/basicmachines-co/basic-memory) | Local knowledge graph over Markdown | Markdown plus a local index database | MCP server (Python) | None. Agents write directly. | 4.5k stars, AGPL-3.0, active |
| [repertory](https://github.com/rambaarde/repertory) | Company-wide memory in a private Git repo. Closest in scope. | Markdown in Git | npm CLI with slash commands, web portal | Not verified | 2 stars, Apache-2.0, very early |
| [waypost](https://github.com/Fryva/waypost) | Project memory (ADRs, specs) for 20+ agent tools | Markdown in Git | Node CLI, `doctor` and `brief` commands | Not verified | 1 star, very early. Name is close to waymark. |
| [teamctx](https://github.com/teamctx/teamctx) | Reports live state (PRs, CI, Jira) before an agent edits. Stores no notes. | None | CLI | Source-backed output, no LLM in the content path | 0 stars. Not sure that this is the project meant earlier. |
| Cline Memory Bank and clones (for example [memory-bank-mcp](https://github.com/alioshr/memory-bank-mcp)) | Fixed set of Markdown files that the agent reads each session | Markdown | Instructions or MCP | None, single user | Pattern, clones mostly stale |
| [MADR](https://github.com/adr/madr), [log4brains](https://github.com/thomvaill/log4brains) | Decision record format and site | Markdown | Not agent-aware | Normal PR review | MADR active, log4brains dormant |

## Lessons for waymark

### Copy

- **One topic per file and a hard size limit.** teamlore uses 120 words. waymark has one topic per file and a hard size cap, checked by `waymark lint`.
- **Soft expiry and a commit anchor.** teamlore stores `verify_by` and the commit of the code that the note is about. If that code changes, the note is probably stale. waymark has `anchors` now and the check comes later.
- **Small read budget.** teamlore gives the agent at most 5 matching notes, about 800 tokens. If nothing matches, it gives nothing. Silence is better than noise. `waymark search` has a read budget too.
- **Start-of-session summary.** waypost has one. `waymark updates` lists new and disputed documents in the same way.
- **Honest output.** teamctx says plainly what it could not check. waymark output says "unverified" and "not checked" in plain words.
- **MADR as the template** for documents that record a decision.

### Avoid

- Anonymous writes. basic-memory lets agents write directly with no trace of the person. A wrong or poisoned note then spreads. waymark writes every commit under the person's Git account, so blame shows who is responsible.
- A runtime such as Node or Python. A native app with no runtime is an advantage.
- Support for only one agent tool.
- A cloud or database dependency.
- A full dump of the memory into the agent context.

### Different on purpose

- One memory repository per organization, not notes inside each code repository. Only repertory does the same.
- Users need no Git knowledge. Other tools need pull request skills.
- Wiki model with personal responsibility. Owner verification is optional. No other project has this.
- Autonomy is a personal setting. teamlore always needs a human pull request merge, which is strong but slow.
- Three agent tools from day one.

## Not verified

- Which `teamctx` repository is meant by the early research.
- Trust and staleness behavior of repertory and waypost.
- Whether Cline Memory Bank has a review step.

## Personal AI memory with a self-hosted MCP server

Research from 2026-11-06, after the decision to serve private people too. Question: does a tool give a person
their own AI memory, in a Git repo they own, reachable from Claude and
ChatGPT through a self-hosted remote MCP server?

**Result:** no mature tool does exactly this. The pieces exist, but not
together.

### Connectors of the AI apps

- **Claude:** custom remote MCP connectors on all plans (Free: one
  connector). Read and write tools allowed. Connections come from
  Anthropic's cloud, so the server needs a public HTTPS URL (or a tunnel)
  and OAuth. Source:
  [Claude help center](https://support.claude.com/en/articles/11175166-get-started-with-custom-connectors-using-remote-MCP).
- **ChatGPT:** "developer mode" with MCP connectors on paid plans. Write
  actions may be limited to business plans. Not verified from OpenAI
  directly (the help page refused automated access).

### Closest tools

| Tool | Storage | Git and Markdown | Remote MCP for Claude and ChatGPT | Maturity |
|---|---|---|---|---|
| [mcp-memory-service](https://github.com/doobidoo/mcp-memory-service) | SQLite, Cloudflare, or vector DB | no | yes, OAuth, both apps (per README, not tested) | 2.1k stars, Apache-2.0, active |
| [basic-memory](https://github.com/basicmachines-co/basic-memory) | Markdown plus local index | Markdown yes, Git only by hand | local MCP. Remote through their paid cloud (a third party holds the data) | 4.5k stars, AGPL-3.0, active |
| [obsidian-mcp-server](https://github.com/cyanheads/obsidian-mcp-server) | Obsidian vault | Markdown yes, Git only if the vault is a repo | HTTP and OAuth options. Needs the Obsidian app running. Apps not verified. | 700 stars, Apache-2.0 |
| [engram](https://github.com/rwnalds/engram) | Markdown vault in Git | yes | self-hosted MCP server, aimed at coding agents. Consumer apps not mentioned. | 20 stars, MIT, very early |
| [jmac-memory-os](https://github.com/JMacCyber/jmac-memory-os) | Markdown facts in your Git repo | yes | "no server". Remote not verified. | 7 stars, very early |
| [mem0](https://github.com/mem0ai/mem0), [Letta](https://github.com/letta-ai/letta) | vector store or database | no | self-hosted MCP not verified | large, Apache-2.0 |

Also Git and Markdown, aimed at coding agents, remote not verified:
`danielmarbach/mnemonic`, `okf-memory/okf-agent-memory`, `krimto-labs/krimto`.

### Gap that waymark could fill

- Git-first documents (one topic per file) with a remote MCP front end that
  works with the OAuth flow of Claude and ChatGPT.
- One small deploy: a container or a home server plus tunnel, with docs
  for both apps.
- Write safety: commit per write, approval, Git history as audit trail.
  Some app modes call tools without asking the person.
- Small tool set (search, read, add, update) and a read-only mode.
- The same file format for the local CLI and the MCP server, so local
  agents and the chat apps share one memory.

## Obsidian and Obsidian Sync

Compared on 2026-11-06.

| | Obsidian Sync | waymark |
|---|---|---|
| Purpose | Sync a personal Obsidian vault across devices | Long-term agent memory, with write rules |
| Who writes | The person, by hand | Mainly agents, as a side effect of work |
| Storage | Markdown on the devices, synced end-to-end encrypted | Markdown in the person's own Git repo (the Git host can read it) |
| History | Version history per file, limited time | Full Git history, blame, undo |
| AI agents | Only through community plugins | Core purpose: skill and CLI for three agent tools |
| Rules and checks | None | One topic per document, size limits, links, secrets, email |
| Teams | Shared vaults, no ownership or review | Team folders, authors, disputes, optional review |
| Price | $4–5 per user per month, app free | Free, open source |

**Lesson:** a waymark memory repository is a valid Obsidian vault (Markdown,
YAML frontmatter, relative links). People can read and browse the
memory repository in Obsidian today, including the link graph. Edits in
Obsidian bypass the waymark checks. `waymark lint` reports problems later.
