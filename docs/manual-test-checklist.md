# Manual test checklist

Every `[manual]` item of the specs that waymark implements, with its steps. Each item names its spec. Tick an item here and in the `test.md` of the spec when it passes, and record the result where the item asks for it.

Before you start: install waymark (`brew install ionpulse-dev/tap/waymark` once the release exists, or `make brew-local` before that) and have a private test repository on a real Git host. The guide for the MCP entries is [install.md](install.md). Results about the agent tools replace the word "unverified" in [install.md](install.md) and [agent-tools-research.md](agent-tools-research.md).

## agent-tool-setup

Spec: [agent-tool-setup](../specs/features/agent-tool-setup/requirements.md), tests in [test.md](../specs/features/agent-tool-setup/test.md).

- [ ] M1: Claude Code. Steps: install waymark with Homebrew. Run `claude mcp add --transport stdio waymark -- waymark --repo <test repository URL>`. Start `claude`. Run `/mcp`. Expected: `waymark` is connected and lists its tools. Repeat with the `.mcp.json` entry and approve the project server at the prompt. Add the permission rules and call `search` (no prompt) and `add` (prompt). Write the Claude Code version in the docs.
- [ ] M2: GitHub Copilot in VS Code. Steps: add `.vscode/mcp.json`, click **Start**, open Copilot Chat in Agent mode, open the tools icon. Expected: the tools of waymark are listed. Note what Copilot asks before a write tool (this answers an unverified item).
- [ ] M3: GitHub Copilot CLI. Steps: `copilot mcp add waymark -- waymark --repo <url>` or edit `~/.copilot/mcp-config.json`. Start `copilot`. Run `/mcp`. Expected: waymark is listed with its tools. Confirm the folder trust prompt for a project entry.
- [ ] M4: OpenAI Codex. Steps: add the table to `~/.codex/config.toml`. Run `codex mcp list`, then start `codex`. Expected: the tools of waymark are available, and a write tool asks before it runs (`writes` mode).
- [ ] M5: a fifth tool that speaks MCP over stdio (any). Steps: enter `waymark` as the command with the arguments of the README. Expected: it lists the tools.
- [ ] M6: for each tool, record whether it honors an explicit `ask` rule without prompts, the kind hints, elicitation, and server instructions. Expected: the answers replace the word "unverified" in `docs/install.md` and `docs/agent-tools-research.md`.

## create-knowledge-base

Spec: [create-knowledge-base](../specs/features/create-knowledge-base/requirements.md), tests in [test.md](../specs/features/create-knowledge-base/test.md).

- [ ] M1: Obsidian. Steps: clone the repository that `init` made. Open the folder as a vault in Obsidian. Create a link between two documents through the link dialog. Expected: the vault opens, the link is a relative Markdown link with `.md`, and `git status` does not list `.obsidian/workspace.json`.
- [ ] M2: real Git host. Steps: create an empty private repository on a real host. Add the entry to an agent tool with the SSH URL. Start a session, call `status` (state `empty`), then ask the agent to call `init`. Expected: the files appear on the host in one commit, and a second person with only the URL can search.

## join-knowledge-base

Spec: [join-knowledge-base](../specs/features/join-knowledge-base/requirements.md), tests in [test.md](../specs/features/join-knowledge-base/test.md).

- [ ] M1: real Git host, new Mac user or clean path. Steps: install waymark with Homebrew. Add the entry with the SSH URL of a private test repository to Claude Code. Start a session, and call `status`. Expected: the clone is at `~/Projects/<name>`, `status` is ok, and the time from install to the first search is under 5 minutes.
- [ ] M2: a Mac without the command line tools (or a test user without them). Steps: start a session with the entry. Call `status`. Expected: the check `git` is not ok, the fix says to install the tools, and the Apple dialog does not block the agent tool.

## login-and-identity

Spec: [login-and-identity](../specs/features/login-and-identity/requirements.md), tests in [test.md](../specs/features/login-and-identity/test.md).

- [ ] M1: real Git host. Steps: with a test repository on a real host, sign out of Git (remove the credential helper entry or use a key without access). Start a session and call `status`, then `sync`. Expected: `git_login` is not ok with the plain `fix` sentence, and nothing waits for input. Sign in with `gh auth login`. Call `status` again. Expected: all ok.

## maintenance

Spec: [maintenance](../specs/features/maintenance/requirements.md), tests in [test.md](../specs/features/maintenance/test.md).

- [ ] M1: real Git host. Steps: with a private test repository, sign out of Git (or use a key without access). Start a session and call `status`. Expected: `git_login` is not ok with the sign-in fix, in under 15 seconds. Sign in with `gh auth login`, call `status` again. Expected: `healthy` is true.
- [ ] M2: a real Mac. Steps: install waymark with Homebrew, add the entry to Claude Code, start a session, call `search`. Then follow the "Remove waymark" steps. Then run `find ~ /opt/homebrew -iname '*waymark*' -not -path '*/Projects/*' 2>/dev/null`. Expected: no result, except the knowledge base folder that you chose to keep, and `~/.claude.json` has no `waymark` entry.

## manual-edits

Spec: [manual-edits](../specs/features/manual-edits/requirements.md), tests in [test.md](../specs/features/manual-edits/test.md).

- [ ] T20: edit in a real Obsidian vault and publish.
- [ ] T21: a new document without properties in Obsidian is blocked.

## move-documents

Spec: [move-documents](../specs/features/move-documents/requirements.md), tests in [test.md](../specs/features/move-documents/test.md).

- [ ] Links still work in Obsidian after a folder move. Steps: run `move` on a folder with 5 linked documents through a real agent tool, pull in a vault, click each link and open the graph. Expected: no unresolved link shows.

## notifications-and-review

Spec: [notifications-and-review](../specs/features/notifications-and-review/requirements.md), tests in [test.md](../specs/features/notifications-and-review/test.md).

- [ ] T15: a real agent tool shows the notice to the person.

## pause-and-resume

Spec: [pause-and-resume](../specs/features/pause-and-resume/requirements.md), tests in [test.md](../specs/features/pause-and-resume/test.md).

- [ ] The kill switch with a real agent tool. Steps: (1) Add the MCP entry to Claude Code and start a session. (2) Ask the agent to add a test document. It works. (3) In a terminal, add `paused = true` under `[limits]` in `~/.config/waymark/config.toml`. (4) Ask the agent to add another document. Expected: the agent reports that writes are paused, and no commit appears in the clone. (5) Set `paused = false` and ask again. Expected: the write works.
- [ ] The README steps work as written, on a Mac with no config file and on a Mac with a config file that has `[limits]`. Expected: no `setup_problem`.

## plain-git-access

Spec: [plain-git-access](../specs/features/plain-git-access/requirements.md), tests in [test.md](../specs/features/plain-git-access/test.md).

- [ ] M1: Claude Code on the web. Steps: create a private repository with the `init` tool of waymark. Open it in Claude Code on the web (no MCP, no install). Ask the agent: "Save a note about X as a document." Expected: the agent reads `AGENTS.md`, asks before it writes (the skill starts in manual mode), writes a document with the required properties, commits `kb: add "<title>"`, and pushes. Then start a waymark server on a clone and call `lint`. Expected: no finding for the new document.
- [ ] M2: Claude Code on a Mac with the waymark entry. Steps: open the knowledge base clone as the working folder. Ask the agent to find a document. Expected: it uses the `search` tool and does not run `git` or `grep` on the repository.
- [ ] M3: Claude Code on iPhone or the web with a team repository (`audience: team`). Steps: ask the agent to write a document with a no-reply email in the Git config. Expected: the agent stops and asks for a real address.

## project-binding

Spec: [project-binding](../specs/features/project-binding/requirements.md), tests in [test.md](../specs/features/project-binding/test.md).

- [ ] M1, Claude Code: the project entry wins. Steps: (1) Add an entry named `waymark` with `--repo <A>` to the user-level MCP config. (2) In a project folder, commit a `.mcp.json` with an entry named `waymark` and `--repo <B>`. (3) Start Claude Code in the project and ask the agent to call `status`. Expected: `status` shows the knowledge base B. Outside the project it shows A. Record the exact config locations.
- [ ] M2, GitHub Copilot in VS Code: as M1. Expected and record: which entry wins.
- [ ] M3, OpenAI Codex CLI: as M1. Expected and record: which entry wins.
- [ ] M4, the first-start prompt. Steps: clone a project that has a committed project entry. Start each of the three tools in it. Expected: the tool asks the person before it starts the server. Record the result for each tool. If one does not ask, the README names it.
- [ ] M5, the working folder. Steps: use the entry `{ "command": "waymark" }` with no arguments, put the knowledge base at `~/Projects/example-knowledge-base`, and start each tool in an unrelated folder. Expected: `status` shows the knowledge base with `found_by: "default_folder"`. Then start each tool inside the knowledge base folder. Expected: `found_by: "working_folder"`. Record any tool that starts the server in another folder.
- [ ] M6, a Git worktree of the code project. Steps: `git worktree add ../wt` in a project with a committed `.mcp.json`. Start Claude Code in `../wt`. Expected: `status` shows the same knowledge base as in the main folder.
- [ ] M7, a large first clone. Steps: use a repository of at least 500 MB as the remote. Start the agent tool with a new `--repo`. Expected: either the server starts, or the tool reports a start time-out and the second start succeeds. Record the time-out of the tool.

## session-start

Spec: [session-start](../specs/features/session-start/requirements.md), tests in [test.md](../specs/features/session-start/test.md).

- [ ] M1, Claude Code uses the instructions. Steps: (1) Start Claude Code in a project with the MCP entry. (2) Without any tool call, ask: "Which knowledge base does waymark use, and what are its limits?" Expected: the agent answers with the name and the limits, or says it needs to call `status`. Record which.
- [ ] M2, GitHub Copilot in VS Code: as M1. Record the result.
- [ ] M3, OpenAI Codex CLI: as M1. Record the result.

## verification

Spec: [verification](../specs/features/verification/requirements.md), tests in [test.md](../specs/features/verification/test.md).

- [ ] Obsidian shows the trust fields. Steps: turn verification on in `.mkbase/waymark.toml`, run `verify` through a real agent tool, open the document in Obsidian. Expected: the Properties panel shows `trust`, `verified_by`, and `verified` without errors.

## write-approval

Spec: [write-approval](../specs/features/write-approval/requirements.md), tests in [test.md](../specs/features/write-approval/test.md).

- [ ] M1, Claude Code CLI shows the request. Steps: (1) Set `[approval] required_for = ["add"]` in `~/.config/waymark/config.toml`. (2) Start Claude Code in a project that has the MCP entry. (3) Ask the agent to add a document. Expected: Claude Code shows the request with the knowledge base, folder, title, and body, and a box "Approve this write". After Accept with the box ticked, the document is committed. After Decline, the agent reports `approval_denied` and nothing is committed.
- [ ] M2, Claude Code in a mode that skips permission prompts. Steps: as M1, but start Claude Code with the permission mode that skips prompts. Expected: record if the request still appears. If it does not, the client auto-answers, and the README says so.
- [ ] M3, Claude Code in VS Code and Claude Desktop. Steps: as M1 in each. Expected: either the request shows, or the tool fails with `approval_unavailable` and nothing is committed. Record which. A silent write is a bug.
- [ ] M4, GitHub Copilot in VS Code. Steps: as M1. Expected and record: as M3.
- [ ] M5, OpenAI Codex CLI. Steps: as M1. Expected and record: as M3. Also record which setting of Codex controls the request.
- [ ] M6, a long wait. Steps: as M1 in each client that showed the request, and wait 100 seconds before the answer. Expected: the write continues after Accept. If the client cuts the call earlier, record the time-out of the client.
- [ ] M7, automatic answers. Steps: in Claude Code, add a hook that answers elicitation requests with `accept` and `approve` true. Run M1. Expected: the write continues with no question. The README states that a hook can do this.
- [ ] M8, the permission-prompt way. Steps: in Claude Code, set the permission rule `ask` for `mcp__waymark__add`. Ask the agent to add a document. Expected: Claude Code asks before it calls the tool. Record the result with a mode that skips prompts.

## write-documents

Spec: [write-documents](../specs/features/write-documents/requirements.md), tests in [test.md](../specs/features/write-documents/test.md).

- [ ] A document written by `add` opens in Obsidian with its properties, links, and tags shown. Steps: run `add` through a real agent tool with `tags`, `related`, and a body link, open the clone as a vault in Obsidian. Expected: the Properties panel shows every field, the links are clickable, and no error shows.

## distribution

Spec: [distribution](../specs/foundation/distribution/requirements.md), tests in [test.md](../specs/foundation/distribution/test.md).

- [ ] After the tap is public: `brew install ionpulse-dev/tap/waymark` on a clean Mac, then `waymark --version`. Expected: the version of the tag.
- [ ] Add the README snippet to Claude Code, GitHub Copilot, and OpenAI Codex. Start a session. Expected: the waymark tools show.

## document-format

Spec: [document-format](../specs/foundation/document-format/requirements.md), tests in [test.md](../specs/foundation/document-format/test.md).

- [ ] The sample vault opens in the latest stable Obsidian desktop release and in Obsidian 1.4 (the floor).

## git-sync

Spec: [git-sync](../specs/foundation/git-sync/requirements.md), tests in [test.md](../specs/foundation/git-sync/test.md).

- [ ] A real Git host with the person's own login: write a document and see it on the host. Steps: configure the MCP entry with an SSH URL, write a document through an agent, open the repository on the host. Expected: one commit with the person's name and email.

## knowledge-base-repo

Spec: [knowledge-base-repo](../specs/foundation/knowledge-base-repo/requirements.md), tests in [test.md](../specs/foundation/knowledge-base-repo/test.md).

- [ ] A repository made by `init` opens in Obsidian as a vault. Steps: run `make sample-kb`, open the folder in Obsidian, create a link with the link dialog. Expected: the link is a relative Markdown link and `git status` shows only the new document.

## mcp-server

Spec: [mcp-server](../specs/foundation/mcp-server/requirements.md), tests in [test.md](../specs/foundation/mcp-server/test.md).

- [ ] A real agent tool lists the tools. Steps: add the MCP entry from the README to Claude Code, GitHub Copilot, and OpenAI Codex, start a session, ask the agent to call `status`. Expected: the tool list shows and `status` returns the knowledge base name.
