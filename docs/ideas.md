# Ideas after the first release

Features that we want later, but not in the first release. When an idea moves into the plan, it gets a spec in `specs/` and leaves this list.

## Web console

| Idea | What it does |
|---|---|
| Knowledge browser | A console page to search and read documents, rendered, with author and history. People can read without an agent. Until then: open the content repo as an Obsidian vault (see [similar-projects.md](similar-projects.md#obsidian-and-obsidian-sync)). |
| Quick capture | A console page where a person writes a document, with a live size counter and the same checks as `add`. |
| Message channels | Email or webhook messages from a server, for example when someone disputes a document. |
| Connect a Git account | An owner connects a Git account with "Sign in with GitHub". The server stores the token encrypted. Needs a key plan and a review of the attack surface. |

## Login and access

| Idea | What it does |
|---|---|
| Device login (OAuth device flow) | Log in with a short code in the browser instead of the Git login that IT prepared. Needs an OAuth app per Git host. |
| Company rollout | Company IT deploys the binary and the system config file (`trust-model`) through device management. People add one MCP entry. |

## Documents

| Idea | What it does |
|---|---|
| Anchor check | A tool flags documents whose `git` or `url` anchor changed. Watch for false alarms on `url` anchors. |
| Personal default folders | A personal default filter for `search` and `updates`, if people ask for it. |
| Attachments | A text document with a binary attachment. Today a document describes binary data and links to it. |

## Agent integration

| Idea | What it does |
|---|---|
| MCP resources | Offer documents as MCP resources besides the tools, if clients use them well. |
| `waymark doctor` | A small command for people without an agent, to check the setup. |

## Personal use and mobile

| Idea | What it does | Open point |
|---|---|---|
| Phone access | Server mode already reaches the chat apps. The console must work on a phone screen. | The documents that an AI reads still go into the conversation at the AI vendor. See [similar-projects.md](similar-projects.md#personal-ai-memory-with-a-self-hosted-mcp-server). |
| Strict separation of private and company use | A private computer or a separate user account. | A project entry already limits mixing in one session. |

## Approval by a second person

| Idea | What it does | Limit |
|---|---|---|
| Pull request gate | A write becomes a pull request on the Git host. A second person approves it, shown in the console. Could be one setting for the whole knowledge base. Reviewers come from the code owners file. | The only server-side control for approval. Needs branch protection on the content repo. |

## Enforce that writes go through waymark

| Idea | What it does | Limit |
|---|---|---|
| Server as the only writer | On a server, people have read access to the content repository and only the credential of the server may write to `main`. A raw `git push` by a person or agent fails. | Needs branch protection on the content repo. Local mode cannot do this. |
| Git hooks | `pre-commit` and `pre-push` in the clone reject unless `waymark` started them. | Stops accidents. `--no-verify` or editing the hooks gets around it. |
| Signed commits plus branch protection | `waymark` signs every commit with a key that other programs cannot use. The Git host rejects unsigned pushes to `main`. Optional hard mode for companies. | Needs branch protection on the content repo. Verify the details per Git host. |

## Permission levels on a server

| Idea | What it does | Limit |
|---|---|---|
| Separate connections for read and write | A server offers one MCP connection or OAuth scope for each permission level. The sign-in carries the limit, so the agent cannot change it. The agent tool can allow the read connection and ask for the write one. | Server mode only. Local mode has no sign-in. Check which scopes the chat apps support. See `server-mode`. |

## Per-folder settings

| Idea | What it does |
|---|---|
| Rules per folder | A different level (`read-only`, approval, `read-write`) for each folder, as ordered rules. The first release has a global `access` and the `folders` list. | A later layer can add these rules without changing the meaning of the limits. |
| Approval per folder | Different approval rules for some folders. The `folders` limit already restricts where tools may act. Per-folder approval is too complex for the first release. |

## Access without waymark

The plain-Git rules are no longer an idea. They are a standalone skill, copied into the `AGENTS.md` of every content repo (see `specs/features/plain-git-access`). Checked: Claude Code in the cloud (web and iPhone app) runs `git` and reaches GitHub through the environment's GitHub integration. Not verified: the plain Claude chat tab, and non-GitHub hosts. Obsidian skills for Claude exist, but they use local file tools or the Obsidian CLI, or sit on an MCP server. None teaches plain Git for a vault in a chat sandbox.
