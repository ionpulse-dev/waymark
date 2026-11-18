# Project decisions

Decisions that apply to the whole project. Decisions that belong to one spec are in the `decisions.md` of that spec. Never delete an entry. Add new ones, and mark a replaced one as superseded.

<br/>

## Index

- [Storage is Markdown in Git, and nothing else — 2026-11-06](#storage-is-markdown-in-git-and-nothing-else--2026-11-06)
- [Everything runs locally, no servers that we operate — 2026-11-06](#everything-runs-locally-no-servers-that-we-operate--2026-11-06)
- [No cloud agents — 2026-11-06](#no-cloud-agents--2026-11-06)
- [Users need no Git knowledge — 2026-11-06](#users-need-no-git-knowledge--2026-11-06)
- [A generic, open source product — 2026-11-06](#a-generic-open-source-product--2026-11-06)
- [The name is `waymark` — 2026-11-06](#the-name-is-waymark--2026-11-06)
- [The specification is public — 2026-11-08](#the-specification-is-public--2026-11-08)
- [Sources of the design problem — 2026-11-06](#sources-of-the-design-problem--2026-11-06)
- [The standalone skill is a separate project — 2026-11-08](#the-standalone-skill-is-a-separate-project--2026-11-08)
- [License and copyright holder — 2026-11-08](#license-and-copyright-holder--2026-11-08)
- [MCP is the only interface for agents — 2026-11-11](#mcp-is-the-only-interface-for-agents--2026-11-11)
- [The server enforces, the team advises — 2026-11-11](#the-server-enforces-the-team-advises--2026-11-11)
- [Two modes, one core — 2026-11-11](#two-modes-one-core--2026-11-11)
- [No macOS app — 2026-11-11](#no-macos-app--2026-11-11)
- [The permission ceiling is layered — 2026-11-11](#the-permission-ceiling-is-layered--2026-11-11)
- [Git credentials stay out of waymark and out of the agent — 2026-11-11](#git-credentials-stay-out-of-waymark-and-out-of-the-agent--2026-11-11)
- [Specs are superseded, never deleted — 2026-11-11](#specs-are-superseded-never-deleted--2026-11-11)
- [The specification follows the MCP-only direction — 2026-11-11](#the-specification-follows-the-mcp-only-direction--2026-11-11)
- [The MKBase standard lives in this repository — 2026-11-11](#the-mkbase-standard-lives-in-this-repository--2026-11-11)
- [Current scope: local mode on macOS, delivered by Homebrew — 2026-11-14](#current-scope-local-mode-on-macos-delivered-by-homebrew--2026-11-14)
- [Feature spec consistency pass (C20) — 2026-11-14](#feature-spec-consistency-pass-c20--2026-11-14)
- [Vocabulary: agent memory and memory repository — 2026-11-14](#vocabulary-agent-memory-and-memory-repository--2026-11-14)
- [The `index` tool is removed — 2026-11-14](#the-index-tool-is-removed--2026-11-14)
- [Tool names follow one scheme, and tools grow in five steps — 2026-11-15](#tool-names-follow-one-scheme-and-tools-grow-in-five-steps--2026-11-15)
- [A remote MCP server for one person is drafted — 2026-11-15](#a-remote-mcp-server-for-one-person-is-drafted--2026-11-15)
- [Linux on x86_64 comes back into scope, first — 2026-11-15](#linux-on-x86_64-comes-back-into-scope-first--2026-11-15)
- [Leaner tools: `list` joins `search`, small edits get small calls — 2026-11-15](#leaner-tools-list-joins-search-small-edits-get-small-calls--2026-11-15)
- [Linux support is implemented — 2026-11-15](#linux-support-is-implemented--2026-11-15)
- [The personal server is built, and tested in real use at the end — 2026-11-15](#the-personal-server-is-built-and-tested-in-real-use-at-the-end--2026-11-15)
- [A menu bar app as an MCP client is drafted — 2026-11-17](#a-menu-bar-app-as-an-mcp-client-is-drafted--2026-11-17)
- [Superseded specs can be removed; the old Mac app specs are removed — 2026-11-17](#superseded-specs-can-be-removed-the-old-mac-app-specs-are-removed--2026-11-17)
- [The menu bar app is in scope — 2026-11-17](#the-menu-bar-app-is-in-scope--2026-11-17)

---

<br/>

## Storage is Markdown in Git, and nothing else — 2026-11-06

**Decision:** Documents are Markdown files in a Git repository. There is no backend interface and no wiki backend such as Confluence.

**Alternatives considered:**
- A second backend for a wiki. A live wiki read has no local copy, and a cache of a wiki gets out of sync fast.

**Reason:** One storage keeps the tool small and lets every feature use Git fully: local search, blame per line, history, and undo. Reasons against a wiki as agent knowledge: pages are long and mix current and old content, so agents load too much; every read is an API call that is slow and rate-limited; there are no format or secret checks before a write; history is per page version, not per line; the page format is not plain Markdown; and it needs a license, while admins often restrict write access through the API.

<br/>

## Everything runs locally, no servers that we operate — 2026-11-06

**Decision:** The Git remote belongs to the owner. The project operates no servers.

**Reason:** A hard user constraint. The only shared part is the Git host that the owner already has.

**Amended by:** MCP is the only interface for agents. The part "no servers that we operate" stays valid. The part "everything runs locally" is widened by a second mode that anyone can host. See "Two modes" below and [architecture-direction.md](architecture-direction.md).

<br/>

## No cloud agents — 2026-11-06

**Decision:** Every agent action happens in a local session that a person starts, with the person's own license. There is no background curator.

**Reason:** The person stays in control. Capture must be part of normal work, so the skill tells the agent when to write.

**Amended by:** MCP is the only interface for agents. There is no skill. In server mode, a person's login stands for the local session. Every agent action still belongs to a person.

<br/>

## Users need no Git knowledge — 2026-11-06

**Decision:** waymark does all Git work. Users never see a branch, a commit, or a conflict.

**Reason:** Managers and other non-developers are users too.

<br/>

## A generic, open source product — 2026-11-06

**Decision:** waymark is a generic knowledge base and AI memory for companies, teams, and private people. One knowledge base is one Git repository, whoever owns it. The project is open source under the MIT license. Nothing company-specific is in this repository.

**Reason:** Private people want their own AI memory under their control, and other companies can use the same tool.

<br/>

## The name is `waymark` — 2026-11-06

**Decision:** The product and CLI are named `waymark`.

**Reason:** A waymark is a marker on a trail: each traveler leaves a mark for the next one, and agents do the same with knowledge. The name is easy to say and type, and a search found no developer tool or AI project with the same name.

<br/>

## The specification is public — 2026-11-08

**Decision:** The specification, the design decisions, the roadmap, and the ideas are public and live in this repository. The project follows spec-driven development. Business, company-specific, and internal planning material stays outside.

**Reason:** In spec-driven development the spec and the code share one checkout, and contributors must see the spec to improve it.

<br/>

## Sources of the design problem — 2026-11-06

**Decision:** The hard problems are write quality, concurrent writes, and staleness, not storage. The design starts with the file layout and the write rules for these three.

**Reason:** Git plus Markdown is a good base: every agent can read files and use Git, history, blame, and review come free, and there is no lock-in. Up to some thousand files, plain search is enough.

<br/>

## The standalone skill is a separate project — 2026-11-08

**Decision:** The plain-Git rules for agents without the CLI are the `waymark-trail` skills, in their own repository. waymark depends on them, never the other way around.

**Amended by:** "The MKBase standard lives in this repository" below. The standard moved here. The home of the skills after `waymark-trail` is retired is open.

**Reason:** They are useful without waymark and can spread on their own.

<br/>

## License and copyright holder — 2026-11-08

**Decision:** MIT license. The copyright holder is IonPulse, the company behind the project's GitHub organization, not a person. The Unlicense and CC0 were rejected: the rights of an author cannot be fully waived in some countries (for example Austria and Germany), and the Unlicense has no fallback license.

**Reason:** MIT is simple and permissive, and the holder should be the organization that owns the repositories.

<br/>

## MCP is the only interface for agents — 2026-11-11

**Decision:** waymark is one binary that is an MCP server and holds all logic. Agents use it only through MCP. There is no CLI for agents. waymark does not ship or install a skill. Teams write the behavior they want in their own `AGENTS.md`, and the README gives examples.

**Alternatives considered:**
- CLI plus skill (the earlier design). The skill can fail to load, and it needs an installer for each agent tool.
- A CLI and an MCP server as two interfaces. Two interfaces to keep in step.
- MCP only for chat apps. Coding agents also support MCP, and tool descriptions are always in their context.

**Reason:** One interface, one behavior to test, and a standard that the three target agent tools support (to be verified). The standalone plain-Git skill remains the fallback for agents without MCP.

**Supersedes:** "No MCP server first" in `specs/foundation/cli-core/decisions.md` (to be marked there when `cli-core` is revised).

<br/>

## The server enforces, the team advises — 2026-11-11

**Decision:** Hard rules live in the server as limits: format, size, secrets, duplicates, result caps, a read budget per session, access mode, and which tools exist. Behavior advice is not built in. There are no tools to change limits or credentials.

**Reason:** Text can be ignored, code cannot. A limit makes the wrong action impossible, and a clear rejection message teaches the agent.

<br/>

## Two modes, one core — 2026-11-11

**Decision:** The core is independent of the transport. Local mode uses MCP over stdio and the Git login of the person. Server mode uses MCP over HTTPS with OAuth and is multi-tenant by design. Build order: local mode, then a self-hosted server for one organization, then multi-tenant.

**Alternatives considered:**
- One server per person. It does not fit a shared company server.
- Remote only. It breaks the quick start with no infrastructure.

**Reason:** Local mode needs no infrastructure. Central limits, single sign-on, one Git credential for an organization, and audit need a shared server. One core serves both.

<br/>

## No macOS app — 2026-11-11

**Superseded by:** "The menu bar app is in scope — 2026-11-17".

**Decision:** The macOS app is dropped. Setup uses the MCP config and the Git login of the person. Approval uses client prompts, elicitation, and a web console in server mode.

**Reason:** The app's real value was a channel that the agent cannot reach. A web console with a separate sign-in gives the same, on every OS. The app is a large cost for one OS.

<br/>

## The permission ceiling is layered — 2026-11-11

**Decision:** Limits come from the user's MCP config, an optional managed config, the shared file in the knowledge base, and the server config. The strictest layer wins. A layer can only tighten. In server mode only the server config is a hard limit.

**Reason:** In local mode a user controls their own machine, so a ceiling guards against mistakes only. In server mode the operator needs limits that the user cannot change.

<br/>

## Git credentials stay out of waymark and out of the agent — 2026-11-11

**Decision:** In local mode, Git uses its own login (SSH key or credential helper). In server mode, the secret comes from the secret store of the host. The commit author is the person who used the agent. The push credential is separate.

**Reason:** The agent and the MCP config must never hold a secret. Separating the push credential from the author keeps the commit history correct, whatever the credential is.

<br/>

## Specs are superseded, never deleted — 2026-11-11

**Decision:** A spec that other specs replace gets the status `superseded` and a `superseded_by` list. Its folder stays. `INDEX.md` lists it in a separate table and does not count it. A short note at the top of the spec and of its `decisions.md` says so.

**Alternatives considered:**
- Delete the old specs. The workflow forbids it, and Git history is not as easy to read.
- Keep them as `draft`. They would still look like part of the plan.

**Reason:** The specification describes the current state, and the history of why it changed stays readable.

**Superseded by:** "Superseded specs can be removed; the old Mac app specs are removed — 2026-11-17" (only the rule that a superseded spec is never deleted).

<br/>

## The specification follows the MCP-only direction — 2026-11-11

**Decision:** All specs were rewritten for the direction above on the same day. Five specs are superseded: `cli-core`, `agent-skill`, `mac-app`, `app-link`, and `approval-dialog`. Five are new: `mcp-server`, `server-mode`, `multi-tenancy`, `write-approval`, and `web-console`. Every spec stays `draft`.

**Reason:** The state of the repository should reflect the new direction completely. See [architecture-direction.md](architecture-direction.md).

<br/>

## The MKBase standard lives in this repository — 2026-11-11

**Decision:** The Waymark Memory Standard 1.0, with its conformance fixtures, is part of waymark. It lives in `standard/` and is maintained here. The `waymark-trail` repository is to be retired when this repository goes public. Until then, both hold the same text, and changes are made here only.

**Alternatives considered:**
- A separate repository for the standard. It adds a repository for a file that this project owns.
- Keep it in `waymark-trail`. That repository is to be taken down.

**Reason:** One repository, one spec workflow, and no second copy to keep in step. See `specs/foundation/document-format/decisions.md`.

<br/>

## Current scope: local mode on macOS, delivered by Homebrew — 2026-11-14

**Decision:** The current task covers only local mode of the `waymark` binary on macOS on Apple silicon, installed with Homebrew from an own tap. Out of scope for now: server mode, multi-tenancy, sign-in, the web console, Linux, and Intel Macs. The specs for these stay `draft` and are not implemented. The affected specs get no new work until the scope changes.

**Autonomous work:** The product owner asked on 2026-11-14 for all in-scope specs to be finished and implemented end to end, using the judgment of the AI, as `specs/WORKFLOW.md` allows under "Approval Gates". Decisions made this way are marked "decided autonomously" in the `decisions.md` of their spec, so they can be reviewed. The AI does not stop for questions. It writes each open question into the spec it belongs to, or into the working plan, and decides with the best default. The AI never performs these outward actions on its own: publishing or releasing anything, creating the Homebrew tap, making this repository public, and creating accounts or domains. It prepares them and leaves them for the product owner.

**Done means implemented:** The product owner runs the `[manual]` tests at the end. A spec stops at `implemented`. Only the product owner moves it to `done`.

**Release:** The Homebrew tap and the release files must be public. This repository is private today. It becomes public at the release, after a check for secrets and business details. The tap is created last, with the approval of the product owner.

**Alternatives considered:** Build local and server mode together (rejected: server mode needs sign-in, tenancy, and a console, which delay a usable binary). Support Linux now (rejected: macOS is the only goal for the first release).

**Reason:** The fastest path to a binary that people can install is local mode on one platform. Everything else builds on it.

<br/>

## Feature spec consistency pass (C20) — 2026-11-14

**Decision (decided autonomously):** The 19 in-scope feature specs were checked together. Tool names, error codes, and `requires` agree with each other and with `src/error.rs` and `trust-model`. No cycle exists in `requires`. One order fix: `create-memory-repository` requires `plain-git-access`, so `plain-git-access` is implemented first. The `console` value of `approval.via` is out of scope. `write-approval` returns `approval_unavailable` for it. `create-memory-repository` still accepts the value in its input, and its tool must pass it on to that error path.

**Implementation order:** write-documents, search-and-read, quality-checks, manual-edits, move-documents, activity-and-undo, verification, pause-and-resume, write-approval, project-binding, session-start, agent-tool-setup, login-and-identity, join-memory-repository, plain-git-access, create-memory-repository, updates, notifications-and-review, maintenance.

**Reason:** `requires` says a spec may start only when its prerequisites are `implemented`.

<br/>

## Vocabulary: agent memory and memory repository — 2026-11-14

**Decision:** The words "knowledge base" and "content repository" are replaced by
"memory repository" in normative text, code, and messages. "Knowledge base" stays
only as a use case. The earlier entries of this file keep their original words,
because they record what was said at the time. The rules and the reasons are in
the spec [terminology](../specs/foundation/terminology/decisions.md).

<br/>

## The `index` tool is removed — 2026-11-14

**Decision:** waymark has no `index` tool and no `index-stale` check. An
`index.md` is a normal document. The older entries of this file and of the
feature specs keep their words. The reasons are in the spec
[remove-index](../specs/foundation/remove-index/decisions.md).

<br/>

## Tool names follow one scheme, and tools grow in five steps — 2026-11-15

**Decision:** Tool names follow the scheme of [tool-conventions](../specs/foundation/tool-conventions/requirements.md): `<verb>_document` for a write on one document, `<verb>_folder` for folders, and bare words for the rest. Nine tools were renamed. Old names work in calls until 0.5.0 and in limits for good. In the same step the tools got a description for every parameter and a "Returns" sentence, and five specs added capability: `search-filters`, `restore-and-replace`, `document-history`, `manage-tags`, and `tool-conventions`.

**Alternatives considered:** No rename (an agent cannot guess `add_empty_folder` from `add`). A rename with no alias, as in 0.2.0 (a limit with an old name would stop working without a warning, and a switched-off tool would come back).

**Reason:** An agent that has learned one tool can guess the next one. A rename must never loosen a limit. The details are in the `decisions.md` of each spec.

<br/>

## A remote MCP server for one person is drafted — 2026-11-15

**Decision:** A new foundation spec, [personal-server](../specs/foundation/personal-server/requirements.md), describes a remote MCP server for one person. It is for phone apps and other MCP clients. It has a sign-in door of its own, "Sign in with GitHub" for one account, Git access through a GitHub App, no stored state, and one setup guide for agents and people. It stays `draft`. The decision "server mode is out of scope for now" is not changed. The spec waits until the product owner changes the scope.

**Alternatives considered:** Extend `server-mode` (rejected: it is built for organizations, with roles and tenancy). Wait with the design (rejected: the design shapes later work, and a draft costs nothing).

**Reason:** The product owner wants a personal remote knowledge base that is as easy as possible for technical people. The reasons for each choice are in the `decisions.md` of the spec.

<br/>

## Linux on x86_64 comes back into scope, first — 2026-11-15

**Decision:** A new foundation spec, [linux-support](../specs/foundation/linux-support/requirements.md), makes waymark build, pass its tests, and run on Linux x86_64. It is `draft`. The spec `personal-server` requires it. The decision of 2026-11-14 "Linux is out of scope" changes for Linux x86_64 when the product owner approves the spec. Server mode, multi-tenancy, sign-in, and the web console stay out of scope.

**Alternatives considered:** Build Linux only inside a container for the server (rejected: the code and the tests would not be proven on Linux, and Linux users of local mode gain nothing). Wait (rejected: servers run Linux, so every later server spec depends on it).

**Reason:** A remote server needs a tested Linux build. On 2026-11-15 waymark already built on Linux and its 214 unit tests passed. The integration tests were not judged in that sandbox, so the first task of the spec is a run on a clean Linux machine.

**Added the same day:** The maintainer works on macOS only. So the spec also delivers a development container and `make linux-*` commands, which run all work in a Linux container on the Mac. That container is arm64 on Apple silicon, so build and tests are in scope for Linux arm64 and x86_64. The first release archive is still x86_64 only.

<br/>

## Leaner tools: `list` joins `search`, small edits get small calls — 2026-11-15

**Decision:** The new feature spec [lean-tools](../specs/features/lean-tools/requirements.md) cuts the tool list from 24 to 23 tools and makes small changes cheap. `list` is a mode of `search` (no query lists). `update_document` takes `replace`, `append`, `add_tags`, and `remove_tags`. `show` returns `deprecation_reason`. `updates` pages with `offset`. The call aliases of the nine 0.4.0 renames end in 0.5.0, as promised, and `list` is the last call alias, until 0.6.0. A test now fails when a hint names a tool that does not exist (it found seven such hints).

**Alternatives considered:** A settings tool, a CLI for setup, one `set_status` tool, one `delete` tool, and `folder_tree` or `list_tags` inside `stats` (all rejected: earlier decisions made them on purpose, see the `decisions.md` of `lean-tools`). A new `patch_document` tool (about 1 KB of context for a case that parameters cover).

**Reason:** The tool schemas cost context in every session. Merging two read tools and shortening repeated texts saves more than the new parameters cost, and an agent that follows a hint must never land on a tool that does not exist.

<br/>

## Linux support is implemented — 2026-11-15

**Decision:** The spec [linux-support](../specs/foundation/linux-support/requirements.md) is `implemented`. waymark builds, passes all 633 tests, and runs on Linux (arm64 native in the development container, x86_64 through emulation). The release workflow (still disabled) adds a Linux x86_64 archive. The Linux install steps carry a note until the maintainer checks them once on a clean Linux machine.

**Reason:** The first Linux run found a serious bug: a `kill` call that killed every process of the user on Linux. It is fixed, and it is in the changelog. Server mode, multi-tenancy, sign-in, and the web console stay out of scope.

<br/>

## The personal server is built, and tested in real use at the end — 2026-11-15

**Decision:** The product owner asked to work on the personal server, so server mode is back in scope for the small, single-owner case: the spec [personal-server](../specs/foundation/personal-server/requirements.md). The organization version (`server-mode`), multi-tenancy, and the web console stay out of scope. The server is built from the best knowledge of the AI apps and GitHub and is tested against a fake GitHub. The real-use pass (`test.md`, M1 to M9) runs after the build.

**Alternatives considered:** Test every app before building (rejected: one maintainer, and some tests need paid plans and admin access).

**Reason:** The unknowns touch only the sign-in door and the Git credential code, not the MCP core. A wrong assumption is a fix in one module. See the decisions of the spec.

<br/>

## A menu bar app as an MCP client is drafted — 2026-11-17

**Superseded by:** "The menu bar app is in scope — 2026-11-17".

**Decision:** A new feature spec, [menu-bar-app](../specs/features/menu-bar-app/requirements.md), describes a macOS menu bar app that reads a memory repository. It is an MCP client of `waymark`: it starts its own `waymark` process, polls at an interval to show changes to watched documents and folders, and lists the person's recent writes for a fast review. Its only write is `undo` of the person's own writes. It stays `draft`. The decision "No macOS app — 2026-11-11" and "A desktop app" under out of scope in [PROJECT.md](../specs/PROJECT.md) are not changed. The spec waits until the product owner changes the scope.

**Alternatives considered:** Restore the superseded `mac-app` spec (rejected: it was built on the CLI and the app link, which no longer exist; see the `decisions.md` of the new spec). A watch feature with MCP subscriptions in `waymark` (rejected: polling works with no change to `waymark`, now that every read updates the clone at its start).

**Reason:** The product owner wants a native window that shows a document or a folder and updates by itself. As an MCP client, the app keeps "MCP is the only interface" and adds no logic to `waymark`.

<br/>

## Superseded specs can be removed; the old Mac app specs are removed — 2026-11-17

**Supersedes:** "Specs are superseded, never deleted — 2026-11-11", for the rule that a superseded spec is never deleted. A replaced spec still gets the status `superseded` first.

**Decision:** The product owner can decide to remove a superseded spec. The steps are in `specs/WORKFLOW.md`, under "Removing a superseded spec". The first use: the three old Mac app specs `foundation/mac-app`, `foundation/app-link`, and `features/approval-dialog` are removed. Their last version is in commit `1e588d9`, for example `git show 1e588d9:specs/foundation/mac-app/requirements.md`. The superseded specs `cli-core` and `agent-skill` stay.

**Alternatives considered:**
- Keep them, with a line at the top that points to `menu-bar-app` (rejected by the product owner: the old specs still confuse).
- Remove all five superseded specs (not asked for now. The same steps apply later).

**Reason:** The new draft [menu-bar-app](../specs/features/menu-bar-app/requirements.md) describes a Mac app again. Three old specs with a different architecture next to it confuse people and agents. Git keeps them, and this entry names the commit. The decisions of the new spec record what it took from `mac-app`.

<br/>

## The menu bar app is in scope — 2026-11-17

**Supersedes:** "No macOS app — 2026-11-11".

**Decision:** The product owner approved the spec [menu-bar-app](../specs/features/menu-bar-app/requirements.md) and asked to build it end to end, with a release prepared up to the publish step. A macOS menu bar app is in scope. It is an MCP client of `waymark`, it has no document or Git logic of its own, and its only write is `undo`. "A desktop app" under out of scope in [PROJECT.md](../specs/PROJECT.md) now names this one exception. Approval in general still uses elicitation and, later, the web console. The app is not an approval channel for agents.

**Alternatives considered:** Keep the app out of scope and use only agents (rejected by the product owner: a native window that updates by itself, and a fast review of the person's own writes, are the user story).

**Reason:** The cost that "No macOS app" named was the app link, the settings that only the app could change, and native code in many specs. The new app has none of these: it uses the existing MCP tools, and `waymark` gets no change for it.

