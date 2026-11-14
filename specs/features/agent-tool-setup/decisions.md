# Decisions: Agent tool setup

> **Note 2026-11-11:** All decisions below are superseded by "Documentation, not installation" at the end of this file.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Three tools from day one — 2026-11-06

**Decision:** Claude Code, GitHub Copilot, and OpenAI Codex. Each runs locally per person, with their own license. No cloud-hosted or background agents.

**Reason:** These are the tools the first users have. Local sessions keep every write under a person's control.

## A static block, found by id — 2026-11-07

**Decision:** The block and the skill hold no path. The agent finds the knowledge base by the id in `.waymark/id`. The first design used a fixed path in the block, `~/Projects/<slug>-agent-memory`; the id binding replaced it. _(Note 2026-11-14: `~/Projects/<slug>-agent-memory` is no longer a default anywhere. See `project-binding`.)_

**Reason:** The text stays static and version-matched, and several knowledge bases can coexist. A known location of the id still lets agents in other repositories find the knowledge base without an MCP server.

## Markers and a manifest — 2026-11-06

**Decision:** Blocks sit between marker comments. Setup records every item in a manifest.

**Reason:** Updates and uninstall touch exactly what waymark wrote.

## Documentation, not installation — 2026-11-11

**Decision:** waymark installs nothing into agent tools. The README gives MCP entries and example `AGENTS.md` snippets.

**Supersedes:** "Three tools from day one" (the three tools stay the documented ones), "A static block, found by id", and "Markers and a manifest".

**Alternatives considered:**
- A helper command that writes the MCP entry. It would be a second interface and would write into files of other tools.

**Reason:** No skill, no instruction block, and no installer.

## Facts come from the official documentation, and unverified items are marked — 2026-11-14

**Decision:** The formats and locations of the three tools were read from the current official documentation on 2026-11-14 and are in `docs/agent-tools-research.md`. Claude Code and Codex are verified for the entry, the scopes, and the permission rules. Copilot is verified for the VS Code project file and the CLI files. Every other statement (kind hints, `ask` rules without prompts, elicitation, server `instructions`, same-name precedence for Codex and Copilot, Copilot permission prompts) is marked "unverified" in the documents. No snippet has run in a real tool yet. Decided autonomously.

**Alternatives considered:**
- Write only what a real run proved (rejected: nothing can be run in this task, and the documentation is a good first source).
- Leave unverified items out (rejected: the person should know what is not known).

**Reason:** The product owner needs working snippets, and an honest mark where the proof is missing.

## The docs live in `docs/install.md`, this spec owns the section — 2026-11-14

**Decision:** `distribution` creates `docs/install.md`. This spec owns the section "Agent tools" in it. The README holds one generic entry and a link. `docs/agent-tools-research.md` holds the sources.

**Alternatives considered:** A second file for the agent tools (rejected: two install guides drift apart).

**Reason:** One install guide for the reader. Decided autonomously. Conflict to note: `distribution/plan.md` lists `docs/install.md` as its new file too. The task order decides who writes it first. The section is added by whoever comes second.

## No automatic check against the real tools — 2026-11-14

**Decision:** The snippets are checked in two ways. An `[auto]` test parses them and checks the rules of the requirements. A `[manual]` test runs each one in the real tool before a release.

**Reason:** A real agent tool needs a person, a login, and a license, so it cannot run in CI.

## Permission advice names tools one by one — 2026-11-14

**Decision:** The Claude Code advice lists each tool as `mcp__waymark__<tool>`, not a wildcard for the read tools.

**Alternatives considered:** `mcp__waymark__*` in `ask` (rejected: it asks for reads too). A glob such as `mcp__waymark__get_*` (rejected: the tool names have no common prefix).

**Reason:** An allow rule accepts a glob only after the literal server prefix, and the read tools share no pattern. A test checks that every name in the advice is a tool of the registry.

## Implementation notes — 2026-11-14

- `docs/install.md` did not exist, so this spec created it with a short "Install the binary" part. `distribution` extends that part later. Decided autonomously.
- The `AGENTS.md` examples live in `docs/install.md` and in the README. Test A3 checks that both copies are identical, so they cannot drift. Decided autonomously.
- The advice names `updates` and `init`, which are built in later specs (`updates`, `create-memory-repository`). Test A5 allows these two names through the list `NOT_YET_BUILT` and fails when one of them becomes a real tool, so that the list is cleaned. Decided autonomously.
- Test A6 checks a fixed list of phrases that must sit on a line with the word "unverified". Add a phrase when a new unverified statement enters the guide. Decided autonomously.
- The test for the generic entry starts the real binary with the `args` of the README entry. The clone starts lazily at the first tool use, so `tools/list` works without network.

## The README is minimal — 2026-11-14

**Decision:** The README keeps only the pitch, the setup with the generic entry, links, and the development commands. The `AGENTS.md` examples live in `docs/install.md` only. Test A3 checks the install guide alone.

**Alternatives considered:** Keep both copies in sync (rejected: it forces the README to stay long).

**Reason:** A short README is easier to read. The install guide is the place for details.

## No test reads the README — 2026-11-14

**Decision:** Test A2 (start the binary with the generic entry of the README) is removed. No test reads `README.md` any more. Test A1 checks the snippets of `docs/install.md` only. The generic entry is covered by manual test M5.

**Reason:** The README is a short project page. A test that pins its content makes edits costly for little value.
