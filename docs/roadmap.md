# Roadmap

The order follows one idea: **prove the skill first, then build the comfort.** The rules in the skill (when to write, what is worth a document) decide whether waymark is useful. The app makes it comfortable, but it cannot fix bad documents.

No dates. Each phase ends with a clear result. The status of each spec is in [specs/INDEX.md](../specs/INDEX.md).

## Overview

| Phase | Result | Main risk it removes |
|---|---|---|
| 0. Close the design | All first-release specs have reviewed requirements, the skill is drafted | Building the wrong thing |
| 1. Core and CLI | Agents read and write documents through `waymark` | Does the skill produce useful documents? |
| 2. Dogfooding | Real use on own knowledge bases for some weeks | Noise, friction, sync problems |
| 3. Mac app | People install, set up, approve, and undo without a terminal | Adoption by non-developers |
| 4. Pilot | A team and private users use it daily | Does it work beyond the author? |
| 5. Public release | Open source release, signed app, docs | — |
| After the first release | Items from [ideas.md](ideas.md) | — |

## Phase 0: Close the design

- [ ] Review and approve the requirements of every spec, foundation first.
- [ ] Draft the skill (`agent-skill`). It is the most important text of the project.
- [ ] Write the document schema and the sample vault (`document-format`), and check the vault in Obsidian.
- [ ] Write the `.mkbase/waymark.toml` schema and the templates for a content repository (`knowledge-base-repo`).
- [ ] Release the standalone plain-Git skill from the `waymark-trail` project and pin a version (`plain-git-access`). Test it in Claude Code on the web.
- [ ] Verify the global instruction and skill paths of Claude Code, GitHub Copilot, and OpenAI Codex (`agent-tool-setup`).
- [ ] Decide the build environment: minimum macOS version, Rust toolchain version, Xcode and Swift version.
- [ ] Specify the message format of the app link (`app-link`).
- [ ] Get an Apple Developer account for signing and notarization.
- [ ] Install Obsidian on the test Mac for the sample vault check.

## Phase 1: Core and CLI

The portable Rust `waymark` CLI with the conformance tests (fixture repo, command, expected output). No app yet.

- [ ] `cli-core`, `document-format`, `knowledge-base-repo`, `trust-model`, `git-sync`.
- [ ] Setup and maintenance: `create-knowledge-base`, `join-knowledge-base` (non-interactive with flags), `login-and-identity`, `agent-tool-setup`, `maintenance`.
- [ ] Binding and session: `project-binding`, `session-start`.
- [ ] Read: `search-and-read`, `updates`.
- [ ] Write: `write-documents`, `move-documents`, `quality-checks`, `plain-git-access`.
- [ ] `pause-and-resume`. In this phase, `resume` asks in the terminal.

Result: an agent in Claude Code, Copilot, or Codex can search, read, write, and dispute documents in a real Git repo.

## Phase 2: Dogfooding

Use waymark for real, before anyone else does:

- A private knowledge base of the author.
- The documents of this project as a second test.

Measure and fix:

- How many documents per week, and how many are noise?
- Do searches find the right document? Is the read budget right?
- How often does a push fail?
- Is the skill clear enough for all three agent tools?

Result: a skill and CLI that the author uses every day without workarounds.

## Phase 3: Mac app

- [ ] `mac-app`, `app-link`, `distribution`.
- [ ] `approval-dialog`, `activity-and-undo`, `notifications-and-review`, `manual-edits`.
- [ ] The native versions of `pause-and-resume`, setup, and login.
- [ ] Signing, notarization, `.dmg`, `install.sh` or a Homebrew cask.

Result: a person without a terminal can install, set up, approve, pause, and undo.

## Phase 4: Pilot

- One team with 3 to 10 people, including at least one non-developer.
- Some private users.
- Collect feedback on noise, approval friction, and setup.

Result: a list of fixes, and a go or no-go for the public release.

## Phase 5: Public release

- [ ] Review the repository for private content, company names, and internal host names.
- [ ] Check the trademark and the domains of the name.
- [ ] Decide how to handle outside contributions (for example a contributor license agreement).
- [ ] README, user docs, and a short demo.
- [ ] Open the sample vault in the current Obsidian version and check properties, links, graph, and tags.
- [ ] Make the repository public and publish the signed release.

## After the first release

See [ideas.md](ideas.md). The largest items:

- A self-hosted MCP server for Claude and ChatGPT connectors. This is the way to use the memory on mobile.
- An optional iPhone app for approval and review only (no LLM inside).
- Knowledge browser, Spotlight, quick capture, company rollout through device management.
- Anchor check (see `docs/ideas.md`), device login.
