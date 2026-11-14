# Install waymark

waymark runs on macOS with Apple silicon. It runs as a local MCP server: your agent tool starts it over stdio for one session.

This guide has two parts. The section "Install the binary" is owned by the `distribution` spec. The section "Agent tools" is owned by the `agent-tool-setup` spec. The facts and sources behind each snippet are in [agent-tools-research.md](agent-tools-research.md).

**Status of the snippets.** No snippet has run in a real tool yet. Each snippet says "not yet tested" until the manual tests of `agent-tool-setup` ran it. A statement that the research could not confirm carries the word "unverified" next to it.

## Install the binary

Install with Homebrew:

```bash
brew install ionpulse-dev/tap/waymark
```

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

Tested with: not yet tested.

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

Tested with: not yet tested.

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
      "mcp__waymark__list",
      "mcp__waymark__folders",
      "mcp__waymark__stats",
      "mcp__waymark__updates",
      "mcp__waymark__lint",
      "mcp__waymark__activity"
    ],
    "ask": [
      "mcp__waymark__add",
      "mcp__waymark__update_document",
      "mcp__waymark__dispute",
      "mcp__waymark__deprecate",
      "mcp__waymark__delete",
      "mcp__waymark__add_empty_folder",
      "mcp__waymark__remove_folder",
      "mcp__waymark__move",
      "mcp__waymark__undo",
      "mcp__waymark__sync",
      "mcp__waymark__index",
      "mcp__waymark__init"
    ]
  }
}
```

The rules name each tool. A glob works only after the prefix `mcp__waymark__`, and the read tools share no pattern.

A mode without prompts (`bypassPermissions`) skips permission prompts. Whether an explicit `ask` rule still prompts in that mode is unverified, so do not rely on it. The other way to get a yes is the approval of waymark itself, through elicitation (see the README, section "Approval"). Whether Claude Code shows that question is unverified.

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

Tested with: not yet tested.

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

Tested with: not yet tested.

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

Tested with: not yet tested.

The `tools` field chooses which tools exist. It is not an approval setting. How Copilot CLI asks before a write tool is unverified. Whether a project entry overrides a user entry of the same name is unverified.

To remove the entry: delete the `waymark` entry in the JSON file.

### Your rules in `AGENTS.md`

waymark ships no behavior text. Three suggestions to copy and change. waymark does not insert or check them. The same text is in the README.

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

### Tighten the limits

Limits come from layers, and the strictest layer wins. Your personal file is `~/.config/waymark/config.toml`. The team's shared file is `.waymark/settings.toml` in the memory repository. IT owns the system file. See the README, section "Limits", and the `trust-model` spec.

## Update waymark

Update the binary with Homebrew:

```bash
brew upgrade waymark
```

The tool descriptions come with the binary. The new version applies at the next start of your agent tool. waymark does not check for updates and calls no server.

**When a team's memory repository needs a newer waymark.** The settings file `.waymark/settings.toml` has a `schema` number. If it is higher than your waymark supports, every tool except `status` fails with "This memory repository needs a newer waymark". Run `brew upgrade waymark` and restart your agent tool. waymark never changes the number by itself. When a file is only missing something that older versions did not write, waymark adds it by itself at your next write, in its own commit (`waymark: repair "settings"`). It never changes a value that you set. See the section "Versions and updates" of the [memory-repository spec](../specs/foundation/memory-repository/requirements.md).

**When your MCP entry needs a change.** waymark never writes into your agent tool. If an update needs a different entry, waymark still starts, and tells you: the agent reports "WAYMARK NEEDS AN UPDATE" with the exact fix, and `status` shows it as a failing check `entry`. Then run `claude mcp remove waymark`, add the entry again as the message says, and restart your agent tool.

## Check that waymark works

Ask your agent to call the `status` tool. It runs a list of checks and gives a plain fix for each problem. It changes nothing.

## Remove waymark

waymark does not remove itself. It never wrote into your agent tools, so these four steps remove everything:

1. Remove the MCP entry from the agent tool. For Claude Code run `claude mcp remove waymark`. For Codex delete the table `[mcp_servers.waymark]` in `config.toml`. For GitHub Copilot delete the `waymark` entry in the JSON file.
2. Check the clone for changes that are not pushed. The folder is `~/.waymark/<name>`, or the `--path` of your entry. In the folder, run `git status` and `git log @{u}..`. Delete the folder if you no longer need it. The remote repository stays.
3. Delete the personal settings folder. If IT set a system file, `/Library/Application Support/waymark/config.toml` belongs to IT.

   ```bash
   rm -r ~/.config/waymark
   ```

4. Remove the binary:

   ```bash
   brew uninstall waymark
   ```
