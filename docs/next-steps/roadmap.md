# Roadmap

The order follows one idea: **prove the tools first, then add the shared server.** The server limits and the tool descriptions decide whether waymark is useful. Local mode proves them with one person and no infrastructure. Server mode then serves a whole organization.

The design changed on 2026-11-11: MCP is the only interface, and there is no skill and no app. See [architecture-direction.md](../architecture-direction.md).

No dates. Each phase ends with a clear result. The status of each spec is in [specs/INDEX.md](../../specs/INDEX.md).

## Overview

| Phase | Result | Main risk it removes |
|---|---|---|
| 0. Close the design | All specs have reviewed requirements, and the facts about the agent tools are verified | Building on wrong assumptions |
| 1. Core and local mode | Agents read and write documents through MCP over stdio | Do the tool descriptions and limits produce useful documents? |
| 2. Dogfooding | Real use on own knowledge bases for some weeks | Noise, context cost, sync problems |
| 3. Server mode | A shared server for one organization, with sign-in, roles, and a web console | Does it work for a team and for chat apps? |
| 4. Pilot | A team and private users use it daily | Does it work beyond the author? |
| 5. Public release | Open source release, signed binary, container image, docs | — |
| After the first release | Multi-tenancy and the items from [ideas.md](../ideas.md) | — |

## Phase 0: Close the design

- [ ] Review and approve the requirements of every spec, foundation first.
- [ ] Verify against the current documentation of Claude Code, GitHub Copilot, and OpenAI Codex: MCP config formats and locations, project entries overriding user entries, the approval prompt for project servers, per-tool permission prompts, elicitation, server instructions, managed MCP config, and remote MCP servers.
- [ ] Test write approval in Claude Code, GitHub Copilot, and OpenAI Codex: does elicitation ask when the tool runs without permission prompts, can a client answer it automatically, and do explicit `ask` rules still prompt (`write-approval`, `agent-tool-setup`).
- [ ] Verify the login flows that Claude and ChatGPT accept for a self-hosted MCP server (`server-mode`).
- [ ] Write the document schema and the sample vault (`document-format`), and check the vault in Obsidian.
- [ ] Write the `.mkbase/waymark.toml` schema and the templates for a content repository (`knowledge-base-repo`).
- [ ] Decide where the standalone plain-Git skills live after `waymark-trail` is retired, pin a version, and test it in Claude Code on the web (`plain-git-access`).
- [ ] Decide the build environment: Rust toolchain version, MCP library, and the signing identity.
- [ ] Install Obsidian on a test machine for the sample vault check.

## Phase 1: Core and local mode

The portable Rust `waymark` binary with the conformance tests (fixture repo, tool call, expected result). stdio only.

- [x] `mcp-server`, `document-format`, `knowledge-base-repo`, `trust-model`, `git-sync`.
- [x] Setup: `create-knowledge-base`, `join-knowledge-base`, `login-and-identity`, `agent-tool-setup` (README snippets), `maintenance`.
- [x] Binding and start: `project-binding`, `session-start`.
- [x] Read: `search-and-read`, `updates`, `notifications-and-review` (notices).
- [x] Write: `write-documents`, `move-documents`, `quality-checks`, `manual-edits`, `activity-and-undo`, `pause-and-resume`, `write-approval` (elicitation).
- [x] `plain-git-access`.

**Status 2026-11-14:** built for macOS on Apple silicon and tested with the real binary (all `[auto]` tests pass). The `[manual]` tests with real agent tools are in the `test.md` of each spec (list them with `make manual-tests`). The Homebrew formula works from a local tap. The tap, the release, and the public switch wait for the product owner ([release-checklist.md](release-checklist.md)).

Result: an agent in Claude Code, Copilot, or Codex can search, read, write, and dispute documents in a real Git repo, within enforced limits.

## Phase 2: Dogfooding

Use waymark for real, before anyone else does:

- A private knowledge base of the author.
- The documents of this project as a second test.

Measure and fix:

- How many documents per week, and how many are noise?
- Do searches find the right document? Are the caps and the read budget right?
- How much context do the tool descriptions use?
- Do rejections teach the agent, or does it loop?
- How often does a push fail?

Result: tools and limits that the author uses every day without workarounds, and a tested set of `AGENTS.md` examples for the README.

## Phase 3: Server mode

- [ ] `server-mode`: the HTTPS transport, sign-in, roles, Git credentials, workspace, and audit log.
- [ ] `web-console` and console approval in `write-approval`.
- [ ] The container image and the deploy guide (`distribution`).
- [ ] Test with the chat apps of Claude and ChatGPT.

Result: an organization runs one server, people sign in, and approvals and undo work in a web page.

## Phase 4: Pilot

- One team with 3 to 10 people, including at least one person without Git knowledge.
- Some private users.
- Collect feedback on noise, approval friction, and setup.

Result: a list of fixes, and a go or no-go for the public release.

## Phase 5: Public release

- [x] Review the repository for private content, company names, and internal host names (`scripts/public-check.sh`, result in the local `tmp/`).
- [ ] Check the trademark and the domains of the name.
- [ ] Decide how to handle outside contributions (for example a contributor license agreement).
- [ ] README, user docs, and a short demo.
- [ ] Open the sample vault in the current Obsidian version and check properties, links, graph, and tags.
- [ ] Make the repository public and publish the signed binary and the image.
- [ ] Retire the `waymark-trail` repository. The standard is in `standard/` here. Move or archive the skills first (see open questions).

## After the first release

See [ideas.md](../ideas.md). The largest items:

- Multi-tenancy (`multi-tenancy`): separate organizations on one server.
- A console page to connect a Git account.
- Knowledge browser, company rollout through device management.
- Anchor check, device login.
