# Decisions: CLI core

> **Note 2026-11-11:** This spec is superseded by `mcp-server`. The decisions below are kept as a record. They no longer apply.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## A portable Rust CLI and a separate Swift app — 2026-11-06

**Decision:** The CLI is written in Rust and holds all logic: documents, checks, Git calls, config, skill install. The menu bar app is Swift, macOS only, holds no document logic, and calls the CLI.

**Alternatives considered:**
- Go (the first choice) — menu bar, dialogs, and Keychain are workarounds.
- Swift for everything — the CLI would be macOS-only, but agents run on all systems.
- Bash — no good YAML parsing, weak error handling, hard to test.
- Python or TypeScript — need a runtime on every machine.
- Rust for the whole product — the macOS UI stays native Swift.

**Reason:** Agents run on every OS and almost nothing in the CLI is macOS-specific. A portable CLI avoids a rewrite. Human safeguards need native UI per OS.

## The agent is the user interface — 2026-11-06

**Decision:** The CLI has no window and no server. Output has machine fields plus a plain message and a `notices` list.

**Reason:** No server means no push to people. Gap accepted: a person who uses no agent at that moment sees notices at the next command.

## Settings in every result — 2026-11-07

**Decision:** Every result carries the current settings.

**Reason:** A change in the app applies at once, also in a running session. Enforced parts apply at once anyway.

## One entry point — 2026-11-06

**Decision:** The CLI is the single entry point for setup, read, write, sync, and checks. Skills call it, and agents do not run raw Git commands in the content repository. Without the CLI, the plain-Git rules apply.

**Reason:** One place for rules and Git safety.

## Skill and templates are compiled in — 2026-11-06

**Decision:** The release is one file per platform. The skill text and the templates for `waymark.toml` and `AGENTS.md` are embedded in it.

**Reason:** Binary and skill versions always match.

## English only — 2026-11-06

**Decision:** The skill text and the `message` fields are English. The skill tells the agent to speak the person's language and to write documents in the language of the knowledge base. App windows are localizable from the start.

**Reason:** One text is easier to test, and agents follow English instructions well.

## No MCP server first — 2026-11-06

**Decision:** Start with conventions, a skill, and a CLI. A thin MCP adapter over the CLI can come later (`waymark mcp`).

**Alternatives considered:**
- An MCP server first — adds a runtime, a version to keep current, a different configuration per tool, and a single point of failure.

**Reason:** Less install friction. The MCP decision stays cheap to change.

## Conformance tests — 2026-11-06

**Decision:** Fixture repositories with expected outputs define the behavior.

**Reason:** Behavior stays fixed when the CLI is rebuilt or ported.
