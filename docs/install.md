# Install waymark

waymark runs on macOS with Apple silicon and on Linux (x86_64). It runs as a local MCP server: your agent tool starts it over stdio for one session.

This guide is for people and for AI agents that install waymark for a person. The "Quick path" lists every step. The section "Install the binary" is owned by the `distribution` spec. The section "Agent tools" is owned by the `agent-tool-setup` spec. The facts and sources behind each snippet are in [agent-tools-research.md](agent-tools-research.md).

## Quick path

**Requirements:** macOS on Apple silicon or Linux on x86_64, and `git` 2.28 or newer installed.

Follow these steps in order. If you are an AI agent, do them for the person. Ask before you install anything. Ask for what you cannot know, such as the repository URL. Ask one question at a time. Never ask for a password or a key, and never type one.

1. **Install the binary.** On macOS, run `brew install ionpulse-dev/tap/waymark`. On Linux, download the archive (see "Install the binary").
2. **Get the URL of the memory repository.** Ask the person for the Git URL of their memory repository. If they have none, they create an empty private repository on their Git host and give you the URL. waymark does not create repositories. The person's own Git login must reach it: `git ls-remote <url>` must succeed.
3. **Add the MCP entry.** Use the entry for the agent tool in use (see "Agent tools") with `--repo <url>`. Most people need nothing else. To keep the clone in a folder of their choice, add `--path <folder>` (see "Where the memory repository lives").
4. **Restart the agent tool.** Approve the `waymark` server if the tool asks.
5. **Check.** Call the `status` tool. The first clone runs in the background, so the state may be `cloning` for a moment. Call `status` again until it is `ready` or `empty`.
6. **Set up an empty repository.** If the state is `empty`, call the `init` tool once. It needs a `name` (ask the person) and takes an optional `audience`: `personal` for one person, `team` (the default) for a team.
7. **Rules for the agent.** Ask the person which of the three suggestions in "Your rules in `AGENTS.md`" they want, and add it to their `AGENTS.md`.
8. **Tell the person it is done** and how to use it: ask their agent to search, save, or update documents in waymark.

A team does the steps once. Everyone else follows steps 1, 3, and 4 with the same URL (see "Share a memory repository with a team").

## Install the binary

### macOS (Apple silicon)

Install with Homebrew:

```bash
brew install ionpulse-dev/tap/waymark
```

### Linux (x86_64)

> **Note:** The Linux steps are not yet tested on a real machine. If a step fails, please open an issue.

Tested on Ubuntu LTS and Debian. Other systems with a recent glibc should work.

1. Make sure `git` is installed and is version 2.28 or newer: `git --version`. If it is missing, install it with the package manager of your system, for example `sudo apt install git`.
2. Download the archive and its checksum from the [releases page](https://github.com/ionpulse-dev/waymark/releases). Pick the files that end in `x86_64-unknown-linux-gnu.tar.gz` and `.tar.gz.sha256`. Put both in one folder.
3. Check the checksum. The command must print `OK`:

   ```bash
   sha256sum -c waymark-<version>-x86_64-unknown-linux-gnu.tar.gz.sha256
   ```

4. Unpack the archive and put `waymark` on your `PATH`:

   ```bash
   tar -xzf waymark-<version>-x86_64-unknown-linux-gnu.tar.gz
   mkdir -p ~/.local/bin
   mv waymark ~/.local/bin/
   ```

   Make sure `~/.local/bin` is in your `PATH` (open a new shell, or add `export PATH="$HOME/.local/bin:$PATH"` to your shell profile).
5. Check it: `waymark --version` prints the version.

To update, download the new archive and replace the file. waymark tells you to update "the way you installed it".

To build from source instead, see [development.md](development.md).

`git` must work and be signed in to your Git host (an SSH key, or `gh auth login`). waymark never sees a Git secret.

## Where the memory repository lives

waymark keeps the memory repository in a folder on your computer. You give it `--repo` (the Git URL), `--path` (a folder), or both. One of them is required. `<name>` is the repository name from the URL, without `.git`.

| `--repo` | `--path` | Folder state | Result |
|---|---|---|---|
| given | none | `~/.waymark/<name>` is missing or empty | Clones into it |
| given | none | holds a repository with the same remote | Uses it |
| given | none | holds a repository with another remote | Error: name collision, set `--path` |
| given | given | `--path` is missing or empty | Clones into `--path` |
| given | given | holds a repository with the same remote | Uses it |
| given | given | holds a repository with another remote | Error: remote mismatch |
| none | given | holds a memory repository | Uses it |
| none | given | missing or empty | Error: add `--repo` to clone it |
| none | none | not applicable | Error: give `--repo` or `--path` |

- **Path wins.** When both are given, `--path` sets the location and `--repo` is only the clone source.
- **No guessing.** waymark does not search your folders. It never uses a repository that you did not name.
- **`~` works** in `--path`, so a project entry that the team shares can hold `~/vaults/acme`.
- **Most people need only `--repo`.** waymark clones into `~/.waymark/<name>` and you never handle the clone. Use `--path` to keep the repository in a visible folder, for example as an Obsidian vault.
- **First start.** The clone runs in the background, so your tool starts at once. `status` shows the state and the folder in use.

## Agent tools

waymark installs nothing into your agent tools and ships no skill. You add one MCP entry, and you write your own rules in `AGENTS.md`. Every entry uses the server name `waymark`, so that an entry in a project can override an entry of the same name for the user (see `project-binding`).

Replace the repository URL with the URL of your memory repository. Add `--path <folder>` to the arguments to choose where the clone lives (see "Where the memory repository lives"). The author name and email are optional. Without them, waymark uses the Git name and email of the person. The examples use `example.org`.

The first time a tool starts a server from a project file, the tool may ask you to approve it. Claude Code does. GitHub Copilot CLI asks you to trust the folder.

### Any tool (generic entry)

A tool that starts an MCP server over stdio needs only a command and arguments.

<!-- snippet: generic -->
```json
{
  "mcpServers": {
    "waymark": {
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git"]
    }
  }
}
```

### Claude Code

Add the entry from the command line (scope `user` for all projects, `project` to write `.mcp.json` that the team shares through Git):

```bash
claude mcp add --transport stdio --scope project waymark -- waymark --repo git@github.com:example/waymark-agent-memory.git
```

Or write the file `.mcp.json` in the project root by hand:

<!-- snippet: claude-code-project -->
```json
{
  "mcpServers": {
    "waymark": {
      "type": "stdio",
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git"],
      "env": {
        "WAYMARK_AUTHOR_NAME": "Jane Doe",
        "WAYMARK_AUTHOR_EMAIL": "jane@example.org"
      }
    }
  }
}
```

A session started by a person asks you to approve a server from `.mcp.json` before it is used. `claude mcp reset-project-choices` resets your answers. A local entry wins over a project entry, and a project entry wins over a user entry. The winner is used whole. Fields are not merged.

**Permissions.** The permission rules are in the settings of Claude Code, not in the MCP entry. Put this in `~/.claude/settings.json` (user) or `.claude/settings.json` (project). The read tools run without a prompt. The write tools ask first.

<!-- snippet: claude-code-permissions -->
```json
{
  "permissions": {
    "allow": [
      "mcp__waymark__status",
      "mcp__waymark__search",
      "mcp__waymark__show",
      "mcp__waymark__folder_tree",
      "mcp__waymark__list_tags",
      "mcp__waymark__history",
      "mcp__waymark__stats",
      "mcp__waymark__updates",
      "mcp__waymark__lint",
      "mcp__waymark__activity"
    ],
    "ask": [
      "mcp__waymark__add_document",
      "mcp__waymark__update_document",
      "mcp__waymark__dispute_document",
      "mcp__waymark__deprecate_document",
      "mcp__waymark__restore_document",
      "mcp__waymark__rename_tag",
      "mcp__waymark__delete_document",
      "mcp__waymark__create_folder",
      "mcp__waymark__delete_folder",
      "mcp__waymark__move",
      "mcp__waymark__undo",
      "mcp__waymark__sync",
      "mcp__waymark__init"
    ]
  }
}
```

The rules name each tool. A glob works only after the prefix `mcp__waymark__`, and the read tools share no pattern.

A mode without prompts (`bypassPermissions`) skips permission prompts. Whether an explicit `ask` rule still prompts in that mode is unverified, so do not rely on it. The other way to get a yes is the approval of waymark itself, through elicitation (see "Tighten the limits"). Whether Claude Code shows that question is unverified.

To remove the entry: `claude mcp remove waymark`.

### OpenAI Codex

Add the table to `~/.codex/config.toml` (user). A trusted project may use `.codex/config.toml` in the project root. Or run `codex mcp add waymark -- waymark --repo git@github.com:example/waymark-agent-memory.git`.

<!-- snippet: codex -->
```toml
[mcp_servers.waymark]
command = "waymark"
args = ["--repo", "git@github.com:example/waymark-agent-memory.git"]
default_tools_approval_mode = "writes"

[mcp_servers.waymark.env]
WAYMARK_AUTHOR_NAME = "Jane Doe"
WAYMARK_AUTHOR_EMAIL = "jane@example.org"
```

`default_tools_approval_mode = "writes"` asks before the tools that are not marked read-only. Whether Codex uses the read-only hint of waymark for this is unverified. The default start time-out of 10 seconds is enough: waymark answers `initialize` at once. Whether a project entry overrides a user entry of the same name in Codex is unverified.

To remove the entry: delete the table `[mcp_servers.waymark]` and its `env` table from `config.toml`.

### GitHub Copilot in VS Code

Write `.vscode/mcp.json` in the project. Click **Start** above the entry, open Copilot Chat in Agent mode, and check the tools icon.

<!-- snippet: copilot-vscode -->
```json
{
  "servers": {
    "waymark": {
      "type": "stdio",
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git"],
      "env": {
        "WAYMARK_AUTHOR_NAME": "Jane Doe",
        "WAYMARK_AUTHOR_EMAIL": "jane@example.org"
      }
    }
  }
}
```

An entry for all your projects goes in the personal `settings.json` of VS Code. The exact key is unverified, so use one location for each server. How Copilot asks before a write tool, and where to set it, is unverified. Check the permission settings of the tool.

To remove the entry: delete the `waymark` entry in `.vscode/mcp.json`.

### GitHub Copilot CLI

Run `copilot mcp add waymark -- waymark --repo git@github.com:example/waymark-agent-memory.git`, or write `~/.copilot/mcp-config.json` (user). A project may use `.mcp.json` or `.github/mcp.json`. The CLI loads project servers only after you confirm folder trust at the first start.

<!-- snippet: copilot-cli -->
```json
{
  "mcpServers": {
    "waymark": {
      "type": "local",
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/waymark-agent-memory.git"],
      "env": {
        "WAYMARK_AUTHOR_NAME": "Jane Doe",
        "WAYMARK_AUTHOR_EMAIL": "jane@example.org"
      },
      "tools": ["*"]
    }
  }
}
```

The `tools` field chooses which tools exist. It is not an approval setting. How Copilot CLI asks before a write tool is unverified. Whether a project entry overrides a user entry of the same name is unverified.

To remove the entry: delete the `waymark` entry in the JSON file.

### Your rules in `AGENTS.md`

waymark ships no behavior text. Three suggestions to copy and change. waymark does not insert or check them.

**Manual: use waymark only when asked**

<!-- example: agents-md -->
```text
Suggestion only. waymark does not insert or check this text.
Use the waymark tools only when the person asks.
Search first, then open at most 3 documents.
Name the author and date when you use a document.
Never write secrets, tokens, or personal data.
```

**Balanced: read alone, write on request**

<!-- example: agents-md -->
```text
Suggestion only. waymark does not insert or check this text.
At the start of a task, search waymark for the topic and open at most 3 documents.
If a document is old or marked disputed, say so and check the facts.
Write only when the person asks, or after you ask "Save this?" and they say yes.
One topic per document, with a one-line description. Update before you add.
Never write secrets, tokens, or personal data.
```

**Proactive: the agent decides, undo is the safety net**

<!-- example: agents-md -->
```text
Suggestion only. waymark does not insert or check this text.
Search waymark before you start work. Save decisions, pitfalls, and contracts that
the next agent needs. Do not save what the code or the history already shows.
Update a document that exists instead of adding a copy. If a document is wrong,
mark it disputed with a reason. Tell the person in one line what you saved.
Never write secrets, tokens, or personal data.
```

### Share a memory repository with a team

A project shares its memory repository through the MCP entry in the project (for example `.mcp.json`). The entry holds the repository URL and no secret. Add `--path` with a `~` path if the team wants the clone in a fixed folder.

```json
{ "mcpServers": { "waymark": { "command": "waymark", "args": ["--repo", "git@github.com:example/waymark-agent-memory.git", "--memory-repository-id", "<id from .waymark/settings.toml>"] } } }
```

- One waymark process serves one memory repository, and no tool can change it.
- `--memory-repository-id` makes the server refuse a repository with another ID.
- `allowed_repo_hosts` in your personal config limits the Git hosts that waymark may use.
- Use the same server name in the personal and the project entry. In the agent tools that were checked, the project entry wins in the project.
- Other tools wait up to 60 seconds for a running clone. A clone that takes more than 5 minutes stops. Clone the repository by hand into the folder (see "Where the memory repository lives") and start again.
- `status` shows the state: `cloning`, `ready`, `empty`, `not_a_memory_repository`, or `broken`.

### Tighten the limits

The server enforces limits. A limit can only tighten, and the strictest layer wins.

| Layer | Who | Strength |
|---|---|---|
| System file (`/etc/waymark/config.toml` on Linux, `/Library/Application Support/waymark/config.toml` on macOS) | IT | Hard in local mode |
| Shared file in the memory repository (`.waymark/settings.toml`) | The team owner | Tools cannot change it |
| Personal file (`~/.config/waymark/config.toml`) | You | A safety default |
| MCP entry | You | A safety default |

See the `trust-model` spec for the details.

**Pause all writes.** Add this to `~/.config/waymark/config.toml`:

```toml
[limits]
paused = true
```

The agent can still read. Write tools disappear from the tool list, and a direct write call fails with `paused`. The change works at the next call, with no restart. There is no timer. To resume, set `paused = false` (or remove the line) in the file that set it. A lower file cannot undo a pause from a higher one. You can also start the server with `--paused`. An agent cannot pause or resume: no tool does it. `access = "read-only"` wins over a pause.

**Ask for a human yes before some writes.** In any config file:

```toml
[approval]
required_for = ["delete_document", "update_document"]   # or ["write"] for every write tool
```

waymark then asks you in the agent tool (MCP elicitation) after all checks passed and before the commit. The question shows the tool, the author, and the full text. You tick "Approve this write" and accept. Decline, cancel, a box that is not ticked, or no answer in 2 minutes means nothing is written (`approval_denied`). A client that cannot show the question gets `approval_unavailable`, and nothing is written. A `preview` never asks. This guards against mistakes. A client hook can answer the question automatically, so it does not stop a deliberate bypass. The other way to get a yes is the permission prompt of your agent tool (see the permission advice above).

## Update waymark

Update the binary the way you installed it. With Homebrew:

```bash
brew upgrade waymark
```

On Linux without Homebrew, download the new archive and replace the file (see "Install the binary").

The tool descriptions come with the binary. The new version applies at the next start of your agent tool. waymark does not check for updates and calls no server.

**When a team's memory repository needs a newer waymark.** The settings file `.waymark/settings.toml` has a `schema` number. If it is higher than your waymark supports, every tool except `status` fails with "This memory repository needs a newer waymark". Update waymark (`brew upgrade waymark` with Homebrew, or the way you installed it) and restart your agent tool. waymark never changes the number by itself. When a file is only missing something that older versions did not write, waymark adds it by itself at your next write, in its own commit (`waymark: repair "settings"`). It never changes a value that you set. See the section "Versions and updates" of the [memory-repository spec](../specs/foundation/memory-repository/requirements.md).

**When your MCP entry needs a change.** waymark never writes into your agent tool. If an update needs a different entry, waymark still starts, and tells you: the agent reports "WAYMARK NEEDS AN UPDATE" with the exact fix, and `status` shows it as a failing check `entry`. Then run `claude mcp remove waymark`, add the entry again as the message says, and restart your agent tool.

## Check that waymark works

Ask your agent to call the `status` tool. It runs a list of checks and gives a plain fix for each problem. It changes nothing.

## Remove waymark

waymark does not remove itself. It never wrote into your agent tools, so these four steps remove everything:

1. Remove the MCP entry from the agent tool. For Claude Code run `claude mcp remove waymark`. For Codex delete the table `[mcp_servers.waymark]` in `config.toml`. For GitHub Copilot delete the `waymark` entry in the JSON file.
2. Check the clone for changes that are not pushed. The folder is `~/.waymark/<name>`, or the `--path` of your entry. In the folder, run `git status` and `git log @{u}..`. Delete the folder if you no longer need it. The remote repository stays.
3. Delete the personal settings folder. If IT set a system file (`/etc/waymark/config.toml` on Linux, `/Library/Application Support/waymark/config.toml` on macOS), it belongs to IT.

   ```bash
   rm -r ~/.config/waymark
   ```

4. Remove the binary:

   ```bash
   brew uninstall waymark
   ```

   On Linux without Homebrew, delete the file that you installed, for example `rm ~/.local/bin/waymark`.
