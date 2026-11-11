# Project decisions

Decisions that apply to the whole project. Decisions that belong to one spec are in the `decisions.md` of that spec. Never delete an entry. Add new ones, and mark a replaced one as superseded.

---

## Storage is Markdown in Git, and nothing else — 2026-11-06

**Decision:** Documents are Markdown files in a Git repository. There is no backend interface and no wiki backend such as Confluence.

**Alternatives considered:**
- A second backend for a wiki. A live wiki read has no local copy, and a cache of a wiki gets out of sync fast.

**Reason:** One storage keeps the tool small and lets every feature use Git fully: local search, blame per line, history, and undo. Reasons against a wiki as agent knowledge: pages are long and mix current and old content, so agents load too much; every read is an API call that is slow and rate-limited; there are no format or secret checks before a write; history is per page version, not per line; the page format is not plain Markdown; and it needs a license, while admins often restrict write access through the API.

## Everything runs locally, no servers that we operate — 2026-11-06

**Decision:** The Git remote belongs to the owner. The project operates no servers.

**Reason:** A hard user constraint. The only shared part is the Git host that the owner already has.

**Amended by:** MCP is the only interface for agents. The part "no servers that we operate" stays valid. The part "everything runs locally" is widened by a second mode that anyone can host. See "Two modes" below and [architecture-direction.md](architecture-direction.md).

## No cloud agents — 2026-11-06

**Decision:** Every agent action happens in a local session that a person starts, with the person's own license. There is no background curator.

**Reason:** The person stays in control. Capture must be part of normal work, so the skill tells the agent when to write.

## Users need no Git knowledge — 2026-11-06

**Decision:** waymark does all Git work. Users never see a branch, a commit, or a conflict.

**Reason:** Managers and other non-developers are users too.

## A generic, open source product — 2026-11-06

**Decision:** waymark is a generic knowledge base and AI memory for companies, teams, and private people. One knowledge base is one Git repository, whoever owns it. The project is open source under the MIT license. Nothing company-specific is in this repository.

**Reason:** Private people want their own AI memory under their control, and other companies can use the same tool.

## The name is `waymark` — 2026-11-06

**Decision:** The product and CLI are named `waymark`.

**Reason:** A waymark is a marker on a trail: each traveler leaves a mark for the next one, and agents do the same with knowledge. The name is easy to say and type, and a search found no developer tool or AI project with the same name.

## The specification is public — 2026-11-08

**Decision:** The specification, the design decisions, the roadmap, and the ideas are public and live in this repository. The project follows spec-driven development. Business, company-specific, and internal planning material stays outside.

**Reason:** In spec-driven development the spec and the code share one checkout, and contributors must see the spec to improve it.

## Sources of the design problem — 2026-11-06

**Decision:** The hard problems are write quality, concurrent writes, and staleness, not storage. The design starts with the file layout and the write rules for these three.

**Reason:** Git plus Markdown is a good base: every agent can read files and use Git, history, blame, and review come free, and there is no lock-in. Up to some thousand files, plain search is enough.

## The standalone skill is a separate project — 2026-11-08

**Decision:** The plain-Git rules for agents without the CLI are the `waymark-trail` skills, in their own repository. waymark depends on them, never the other way around.

**Reason:** They are useful without waymark and can spread on their own.

## License and copyright holder — 2026-11-08

**Decision:** MIT license. The copyright holder is IonPulse, the company behind the project's GitHub organization, not a person. The Unlicense and CC0 were rejected: the rights of an author cannot be fully waived in some countries (for example Austria and Germany), and the Unlicense has no fallback license.

**Reason:** MIT is simple and permissive, and the holder should be the organization that owns the repositories.

## MCP is the only interface for agents — 2026-11-11

**Decision:** waymark is one binary that is an MCP server and holds all logic. Agents use it only through MCP. There is no CLI for agents. waymark does not ship or install a skill. Teams write the behavior they want in their own `AGENTS.md`, and the README gives examples.

**Alternatives considered:**
- CLI plus skill (the earlier design). The skill can fail to load, and it needs an installer for each agent tool.
- A CLI and an MCP server as two interfaces. Two interfaces to keep in step.
- MCP only for chat apps. Coding agents also support MCP, and tool descriptions are always in their context.

**Reason:** One interface, one behavior to test, and a standard that the three target agent tools support (to be verified). The plain-Git rules of `waymark-trail` remain the fallback for agents without MCP.

**Supersedes:** "No MCP server first" in `specs/foundation/cli-core/decisions.md` (to be marked there when `cli-core` is revised).

## The server enforces, the team advises — 2026-11-11

**Decision:** Hard rules live in the server as limits: format, size, secrets, duplicates, result caps, a read budget per session, access mode, and which tools exist. Behavior advice is not built in. There are no tools to change limits or credentials.

**Reason:** Text can be ignored, code cannot. A limit makes the wrong action impossible, and a clear rejection message teaches the agent.

## Two modes, one core — 2026-11-11

**Decision:** The core is independent of the transport. Local mode uses MCP over stdio and the Git login of the person. Server mode uses MCP over HTTPS with OAuth and is multi-tenant by design. Build order: local mode, then a self-hosted server for one organization, then multi-tenant.

**Alternatives considered:**
- One server per person. It does not fit a shared company server.
- Remote only. It breaks the quick start with no infrastructure.

**Reason:** Local mode needs no infrastructure. Central limits, single sign-on, one Git credential for an organization, and audit need a shared server. One core serves both.

## No macOS app — 2026-11-11

**Decision:** The macOS app is dropped. Setup uses the MCP config and the Git login of the person. Approval uses client prompts, elicitation, and a web console in server mode.

**Reason:** The app's real value was a channel that the agent cannot reach. A web console with a separate sign-in gives the same, on every OS. The app is a large cost for one OS.

## The permission ceiling is layered — 2026-11-11

**Decision:** Limits come from the user's MCP config, an optional managed config, the shared file in the knowledge base, and the server config. The strictest layer wins. A layer can only tighten. In server mode only the server config is a hard limit.

**Reason:** In local mode a user controls their own machine, so a ceiling guards against mistakes only. In server mode the operator needs limits that the user cannot change.

## Git credentials stay out of waymark and out of the agent — 2026-11-11

**Decision:** In local mode, Git uses its own login (SSH key or credential helper). In server mode, the secret comes from the secret store of the host. The commit author is the person who used the agent. The push credential is separate.

**Reason:** The agent and the MCP config must never hold a secret. Separating the push credential from the author keeps the commit history correct, whatever the credential is.
