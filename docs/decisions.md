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

**Amended by:** MCP is the only interface for agents. There is no skill. In server mode, a person's login stands for the local session. Every agent action still belongs to a person.

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

**Amended by:** "The MKBase standard lives in this repository" below. The standard moved here. The home of the skills after `waymark-trail` is retired is open (see [open-questions.md](next-steps/open-questions.md)).

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

**Reason:** One interface, one behavior to test, and a standard that the three target agent tools support (to be verified). The standalone plain-Git skill remains the fallback for agents without MCP.

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

## Specs are superseded, never deleted — 2026-11-11

**Decision:** A spec that other specs replace gets the status `superseded` and a `superseded_by` list. Its folder stays. `INDEX.md` lists it in a separate table and does not count it. A short note at the top of the spec and of its `decisions.md` says so.

**Alternatives considered:**
- Delete the old specs. The workflow forbids it, and Git history is not as easy to read.
- Keep them as `draft`. They would still look like part of the plan.

**Reason:** The specification describes the current state, and the history of why it changed stays readable.

## The specification follows the MCP-only direction — 2026-11-11

**Decision:** All specs were rewritten for the direction above on the same day. Five specs are superseded: `cli-core`, `agent-skill`, `mac-app`, `app-link`, and `approval-dialog`. Five are new: `mcp-server`, `server-mode`, `multi-tenancy`, `write-approval`, and `web-console`. Every spec stays `draft`.

**Reason:** The state of the repository should reflect the new direction completely. See [architecture-direction.md](architecture-direction.md).

## The MKBase standard lives in this repository — 2026-11-11

**Decision:** The MKBase Document Standard 1.0, with its conformance fixtures, is part of waymark. It lives in `standard/` and is maintained here. The `waymark-trail` repository is to be retired when this repository goes public. Until then, both hold the same text, and changes are made here only.

**Alternatives considered:**
- A separate repository for the standard. It adds a repository for a file that this project owns.
- Keep it in `waymark-trail`. That repository is to be taken down.

**Reason:** One repository, one spec workflow, and no second copy to keep in step. See `specs/foundation/document-format/decisions.md`.

## Current scope: local mode on macOS, delivered by Homebrew — 2026-11-14

**Decision:** The current task covers only local mode of the `waymark` binary on macOS on Apple silicon, installed with Homebrew from an own tap. Out of scope for now: server mode, multi-tenancy, sign-in, the web console, Linux, and Intel Macs. The specs for these stay `draft` and are not implemented. The affected specs get no new work until the scope changes.

**Autonomous work:** The product owner asked on 2026-11-14 for all in-scope specs to be finished and implemented end to end, using the judgment of the AI, as `specs/WORKFLOW.md` allows under "Approval Gates". Decisions made this way are marked "decided autonomously" in the `decisions.md` of their spec, so they can be reviewed. The AI does not stop for questions. It writes each open question into the spec it belongs to, or into the working plan, and decides with the best default. The AI never performs these outward actions on its own: publishing or releasing anything, creating the Homebrew tap, making this repository public, and creating accounts or domains. It prepares them and leaves them for the product owner.

**Done means implemented:** The product owner runs the `[manual]` tests at the end. A spec stops at `implemented`. Only the product owner moves it to `done`.

**Release:** The Homebrew tap and the release files must be public. This repository is private today. It becomes public at the release, after a check for secrets and business details. The tap is created last, with the approval of the product owner.

**Alternatives considered:** Build local and server mode together (rejected: server mode needs sign-in, tenancy, and a console, which delay a usable binary). Support Linux now (rejected: macOS is the only goal for the first release).

**Reason:** The fastest path to a binary that people can install is local mode on one platform. Everything else builds on it.

## Feature spec consistency pass (C20) — 2026-11-14

**Decision (decided autonomously):** The 19 in-scope feature specs were checked together. Tool names, error codes, and `requires` agree with each other and with `src/error.rs` and `trust-model`. No cycle exists in `requires`. One order fix: `create-knowledge-base` requires `plain-git-access`, so `plain-git-access` is implemented first. The `console` value of `approval.via` is out of scope. `write-approval` returns `approval_unavailable` for it. `create-knowledge-base` still accepts the value in its input, and its tool must pass it on to that error path.

**Implementation order:** write-documents, search-and-read, quality-checks, manual-edits, move-documents, activity-and-undo, verification, pause-and-resume, write-approval, project-binding, session-start, agent-tool-setup, login-and-identity, join-knowledge-base, plain-git-access, create-knowledge-base, updates, notifications-and-review, maintenance.

**Reason:** `requires` says a spec may start only when its prerequisites are `implemented`.
