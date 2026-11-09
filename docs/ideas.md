# Ideas after the first release

Features that we want later, but not in the first release. When an idea moves into the plan, it gets a spec in `specs/` and leaves this list.

## Native app

| Idea | What it does |
|---|---|
| Knowledge browser | A window to search and read documents, rendered, with author and history. People can read without an agent. Until then: open the content repo as an Obsidian vault (see [similar-projects.md](similar-projects.md#obsidian-and-obsidian-sync)). |
| Spotlight search | Documents show up in macOS Spotlight (Cmd-Space), through Core Spotlight. |
| Quick capture | A global hotkey opens a small form. A person writes a document, with a live size counter and the same checks as `waymark add`. |
| Skill viewer | A read-only view of the skill text and the current read and write levels in the app, so people see what their agent is told. No editing. |
| Company rollout | Company IT deploys the app with a pre-filled config (repo URL, email domains) through device management (MDM). People only log in. |
| "Save to waymark" | A share extension. Text from Safari, Slack, or Mail becomes a document draft. |

## Login and access

| Idea | What it does |
|---|---|
| Device login (OAuth device flow) | Log in with a short code in the browser instead of a pasted token. Needs an OAuth app per Git host. The project could own one for github.com, and an organization with its own Git host registers one and puts the client ID in `.mkbds/waymark.toml`. |

## Documents

| Idea | What it does |
|---|---|
| Anchor check | A command flags documents whose `git` or `url` anchor changed. Watch for false alarms on `url` anchors. |
| Personal default folders | A personal default filter for `search` and `updates`, if people ask for it. |
| Attachments | A text document with a binary attachment. Today a document describes binary data and links to it. |

## Agent integration

| Idea | What it does |
|---|---|
| Thin MCP server | `waymark mcp` exposes the CLI commands as MCP tools. Only if a real need shows up. |
| Session start hooks | Use the session start hooks of agent tools to run `waymark session` without a rule in the skill. |

## Personal use and mobile

| Idea | What it does | Open point |
|---|---|---|
| iPhone companion app | Read and search documents, capture documents by voice or share sheet, approve or undo agent writes, review list. | There is no `git` binary on iOS: it needs `libgit2` or the Git host's web API. Coding agents do not run on the iPhone, and phone AI apps cannot reach a local CLI. Without an LLM, the app is only a viewer and approval tool. The mobile AI memory comes from the Claude and ChatGPT apps through the MCP connector (next row). This app is optional. |
| Self-hosted remote MCP server | A small server (container, VPS, or home server with a tunnel) in front of the person's Git repo. Claude and ChatGPT connect as a custom connector with OAuth. Tools: search, read, add, update, and a read-only mode. The memory stays in the person's repo. | Breaks "no server" for this feature. The documents that an AI reads still go into the conversation at the AI vendor. See [similar-projects.md](similar-projects.md#personal-ai-memory-with-a-self-hosted-mcp-server). |
| Strict separation of private and company use | A private Mac or a separate macOS user. | Project binding already limits mixing in one session. |

## Approval by a second person

| Idea | What it does | Limit |
|---|---|---|
| Pull request gate | A write becomes a pull request on the Git host. A second person approves it, shown in the review list. Could be one setting for the whole knowledge base. Reviewers come from the code owners file. | The only server-side control for approval. Needs branch protection on the content repo. |

## Enforce that writes go through waymark

| Idea | What it does | Limit |
|---|---|---|
| Git hooks | `pre-commit` and `pre-push` in the clone reject unless `waymark` started them. `setup` installs them. | Stops accidents. `--no-verify` or editing the hooks gets around it. |
| Credential isolation | The token is in the Keychain, readable only by the signed `waymark` binary. The clone has no other credential, so a raw `git push` fails. | Verify the Keychain access rules. A global Git credential or SSH key defeats it. |
| Signed commits plus branch protection | `waymark` signs every commit with a key that other programs cannot use. The Git host rejects unsigned pushes to `main`. Optional hard mode for companies. | Needs branch protection on the content repo. Verify the details per Git host. |

## Per-folder settings

| Idea | What it does |
|---|---|
| Approval per folder | Different write levels for some folders. Dropped from the first release because it is too complex. |

## CLI-free access

The plain-Git rules are no longer an idea. They are a standalone skill, copied into the `AGENTS.md` of every content repo (see `specs/features/plain-git-access`). Checked: Claude Code in the cloud (web and iPhone app) runs `git` and reaches GitHub through the environment's GitHub integration. Not verified: the plain Claude chat tab, and non-GitHub hosts. Obsidian skills for Claude exist, but they use local file tools or the Obsidian CLI, or sit on an MCP server. None teaches plain Git for a vault in a chat sandbox.
