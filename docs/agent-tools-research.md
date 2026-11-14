# Agent tools research

Research for `specs/features/agent-tool-setup`. It records what is known on 2026-11-14 about how Claude Code, GitHub Copilot, and OpenAI Codex add a local MCP server (stdio). The snippets in `README.md` and `docs/install.md` come from here.

**Legend.** `verified` means the text of the current official documentation (page and date below) states it. `unverified` means the documentation read did not state it, or the page was not read. No snippet was run in a real tool yet. The `[manual]` tests of `agent-tool-setup` do that before a release.

The entry for waymark has the server name `waymark` in all tools. The examples use `example.org`.

## Claude Code

Source: `https://code.claude.com/docs/en/mcp` and `https://code.claude.com/docs/en/permissions`, read on 2026-11-14.

| Item | Status | Content |
|---|---|---|
| Add by command | verified | `claude mcp add --transport stdio waymark -- waymark --repo <url>`. The `--` separates the options of Claude from the server command. `--env KEY=value` sets variables. Put another option between `--env` and the server name. |
| Scopes | verified | `local` (default): `~/.claude.json` under the project path. `project`: `.mcp.json` in the project root, shared by Git. `user`: `~/.claude.json`, all projects. Flag `--scope`. |
| JSON format | verified | Top-level `mcpServers`. An entry has `type` (`stdio`), `command`, `args`, `env`. The syntax `${VAR}` and `${VAR:-default}` works in these fields. |
| Same name in several places | verified | Local wins over project, project over user. The whole entry of the winner is used. Fields are not merged. |
| Approval of a project server | verified | An interactive session asks the person to approve each server of `.mcp.json` before use. `claude mcp reset-project-choices` resets the choices. `claude -p` and Agent SDK sessions load project servers without asking. |
| Permission rule names | verified | `mcp__<server>__<tool>`. `mcp__waymark` and `mcp__waymark__*` match all tools of the server. Rules go in `permissions.allow`, `permissions.ask`, `permissions.deny` of `~/.claude/settings.json` (user), `.claude/settings.json` (project), or `.claude/settings.local.json`. |
| Wildcards | verified | An allow rule accepts a glob only after the literal prefix `mcp__<server>__`. Rules for MCP tools cannot have parentheses (they are skipped). |
| Mode without prompts (`bypassPermissions`) | verified | It skips permission prompts. The page lists exceptions ("actions no mode auto-approves"). |
| Does an explicit `ask` rule on an MCP tool still prompt in `bypassPermissions`? | unverified | The page says the mode skips prompts. It does not say, for MCP tools, whether an `ask` rule wins. Do not rely on it. |
| Tool kind hints (`readOnlyHint`, `destructiveHint`) shown or used | unverified | Not stated in the pages read. |
| Elicitation (a form request from the server) | unverified | Not read. `write-approval` is out of the current scope. |
| Server `instructions` field used | unverified | Not read. `session-start` has the same open item. |
| Managed MCP config for IT | partly verified | The page names an organization setting `managedMcpServers` that ranks above all scopes. Details not read. |

Minimal entry (`.mcp.json`, project scope):

```json
{
  "mcpServers": {
    "waymark": {
      "type": "stdio",
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/knowledge.git"],
      "env": { "WAYMARK_AUTHOR_NAME": "Jane Doe", "WAYMARK_AUTHOR_EMAIL": "jane@example.org" }
    }
  }
}
```

Permission rules (`~/.claude/settings.json`): allow the read tools, ask for the write tools.

```json
{
  "permissions": {
    "allow": ["mcp__waymark__status", "mcp__waymark__search", "mcp__waymark__show", "mcp__waymark__list", "mcp__waymark__folders", "mcp__waymark__stats", "mcp__waymark__updates", "mcp__waymark__lint", "mcp__waymark__activity"],
    "ask": ["mcp__waymark__add", "mcp__waymark__update_document", "mcp__waymark__dispute", "mcp__waymark__deprecate", "mcp__waymark__delete", "mcp__waymark__add_empty_folder", "mcp__waymark__remove_folder", "mcp__waymark__move", "mcp__waymark__undo", "mcp__waymark__sync", "mcp__waymark__index", "mcp__waymark__init"]
  }
}
```

## OpenAI Codex

Source: `https://developers.openai.com/codex/mcp`, which redirects to `https://learn.chatgpt.com/docs/extend/mcp?surface=cli`, read on 2026-11-14. The ChatGPT desktop app, the Codex CLI, and the IDE extension share this configuration.

| Item | Status | Content |
|---|---|---|
| Config file | verified | `~/.codex/config.toml` (user). `.codex/config.toml` in the project root (trusted projects only). |
| Add by command | verified | `codex mcp add <name> --env VAR=VALUE -- <command> [args]`. `codex mcp list` lists servers. |
| TOML keys | verified | Table `[mcp_servers.<name>]` with `command` (required), `args`, `env` (table), `env_vars` (names to forward), `cwd`, `enabled`, `startup_timeout_sec` (default 10), `tool_timeout_sec` (default 60), `enabled_tools`, `disabled_tools`. |
| Approval | verified | `default_tools_approval_mode` with `auto`, `prompt`, `writes`, `approve`. `writes` prompts for tools not marked read-only. `[mcp_servers.<name>.tools.<tool>] approval_mode` overrides one tool. |
| Same name in user and project config | unverified | Not stated. |
| Approval or trust prompt for a project server | partly verified | Project config loads for trusted projects only. The prompt flow is not described. |
| Elicitation, server `instructions` | unverified | Not read. |
| Whether `writes` uses the `readOnlyHint` of the tool | unverified | The page says "tools not marked read-only". The mark is assumed to be the MCP annotation. |

Entry (`~/.codex/config.toml`):

```toml
[mcp_servers.waymark]
command = "waymark"
args = ["--repo", "git@github.com:example/knowledge.git"]
default_tools_approval_mode = "writes"

[mcp_servers.waymark.env]
WAYMARK_AUTHOR_NAME = "Jane Doe"
WAYMARK_AUTHOR_EMAIL = "jane@example.org"
```

The first clone runs in the background, so the default start time-out of 10 seconds is enough (see `specs/features/join-knowledge-base`).

## GitHub Copilot

Sources: `https://docs.github.com/en/copilot/customizing-copilot/using-model-context-protocol/extending-copilot-chat-with-mcp` (VS Code) and `https://docs.github.com/en/copilot/how-tos/copilot-cli/customize-copilot/add-mcp-servers` (CLI), read on 2026-11-14.

| Item | Status | Content |
|---|---|---|
| VS Code, project file | verified | `.vscode/mcp.json`, top-level `servers`. An entry has `type` (`stdio`), `command`, `args`, `env`. The person clicks **Start** above the entry, then checks the tools icon in Agent mode. |
| VS Code, user scope | unverified | The page says personal `settings.json` can hold servers. The exact key was not seen. Use one location per server. |
| CLI, add by command | verified | `copilot mcp add <name> -- <command> [args]`. It writes the user config. `/mcp add` in a session opens a form. |
| CLI, files | verified | User: `~/.copilot/mcp-config.json`. Project: `.mcp.json` or `.github/mcp.json`. |
| CLI, JSON format | verified | Top-level `mcpServers`. An entry has `type` (`local` for a stdio server), `command`, `args`, `env`, `tools` (`["*"]` is the default). Project files may also use the bare form (server names as top-level keys). |
| CLI, project servers | verified | They load only after the person confirms folder trust on first launch. |
| Same name in user and project config | unverified | Not stated. |
| Permission prompt per tool, and how to ask only for write tools | unverified | The pages read do not describe it. The `tools` field limits which tools exist. It is not an approval setting. |
| Kind hints, elicitation, server `instructions` | unverified | Not read. |
| Copilot in other IDEs (JetBrains, Xcode, Visual Studio) | unverified | Not read. Out of the documented set. |

VS Code entry (`.vscode/mcp.json`):

```json
{
  "servers": {
    "waymark": {
      "type": "stdio",
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/knowledge.git"],
      "env": { "WAYMARK_AUTHOR_NAME": "Jane Doe", "WAYMARK_AUTHOR_EMAIL": "jane@example.org" }
    }
  }
}
```

CLI entry (`~/.copilot/mcp-config.json`):

```json
{
  "mcpServers": {
    "waymark": {
      "type": "local",
      "command": "waymark",
      "args": ["--repo", "git@github.com:example/knowledge.git"],
      "env": { "WAYMARK_AUTHOR_NAME": "Jane Doe", "WAYMARK_AUTHOR_EMAIL": "jane@example.org" },
      "tools": ["*"]
    }
  }
}
```

## Other tools

Any tool that starts an MCP server over stdio can use the command `waymark` with the arguments of `specs/foundation/mcp-server/requirements.md`. This is the generic entry. It is not verified against any tool beyond the three above.

## Open items

1. Run each snippet in the current version of its tool and record the version (manual tests of `agent-tool-setup`).
2. Find out for each tool: kind hints, `ask` rules without prompts, elicitation, server `instructions`, and same-name precedence between user and project entries (Codex, Copilot).
3. Read the permission page of Copilot (VS Code and CLI) and find how to ask only before write tools.
4. Repeat the check before each release. The tools change fast.
