# Decisions: Session start

> **Note 2026-11-11:** "`session` informs, commands enforce" is superseded by "Server instructions" below. "Settings in every result" and "No updates in the session result" stay valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## `session` informs, commands enforce — 2026-11-07

**Decision:** The start command only informs. Every command checks the rules itself.

**Reason:** Advice in the skill is not a control.

## Settings in every result — 2026-11-07

**Decision:** The agent follows the `settings` of the latest result, not the ones from the start.

**Reason:** A change in the app then applies at once, also in a running session.

## No updates in the session result — 2026-11-09

**Decision:** `session` does not include `updates`, also at read level `auto`.

**Reason:** A change report at the start costs context in every session. The person can ask for `updates` at any time. The tested rules of the waymark-trail skills dropped the startup report.

## Server instructions — 2026-11-11

**Decision:** The server sends a short neutral summary as MCP instructions at connect. `status` returns the same facts. There is no `session` call and no behavior advice.

**Reason:** no skill tells the agent to call a first command. Alternative considered: a first tool that an agent must call. Nothing can make it call it.

## The exact text and the fields — 2026-11-14

**Decision:** The instructions are five short lines in a fixed format (see `requirements.md`). The limits are the same as the `settings` object, in prose. The `status` result has `knowledge_base` (`name`, `id`, `slug`, `path`, `remote`, `found_by`) and `settings`. The `settings` object has these keys: `access`, `paused`, `tools_denied`, `folders`, `max_results`, `max_tokens`, `max_documents_opened`, and `approval` with `required_for` and `via`. This object is the shape for every result of every tool.

**Alternatives considered:**
- A list of all tools in the instructions (the old example). Rejected: `tools/list` has it already, and it costs context in every session.
- `allowed_repo_hosts` in `settings`. Rejected: it matters to the setup, not to an agent.
- A long text with the meaning of every limit. Rejected: context cost, and the descriptions of the tools carry the meaning.

**Reason:** One shape for the facts, and a stable text that tests can compare. Decided autonomously. `mcp-server` has the short instructions text in its plan, item 9. This spec fills it in (see the report).

## No behavior advice, and a test for it — 2026-11-14

**Decision:** The text uses only statements of fact. A test fails if the text holds `should`, `must`, `always`, `never`, `please`, `prefer`, or `you`.

**Alternatives considered:** Advice such as "read before you write". Rejected: it is the choice of each team in its `AGENTS.md`, and a server cannot enforce it.

**Reason:** `mcp-server` says tool descriptions and instructions carry no behavior advice. A word list is a cheap guard. Decided autonomously.

## Instructions use local data only — 2026-11-14

**Decision:** `initialize` makes no network call. The instructions come from the arguments, the config files, and the shared file in the clone. A first clone happens before `initialize` and belongs to `project-binding`. The pull "at start" of `git-sync` happens at the first read, because a clone with no recorded good pull counts as stale.

**Alternatives considered:** Pull before `initialize`. Rejected: a slow network would delay the start and the client could time out.

**Reason:** A fast start matters more than the freshest limits. Limits in the shared file reach the session at the first call anyway. Decided autonomously.

## The instructions are a snapshot — 2026-11-14

**Decision:** The instructions are made once. The text says "at session start". The current facts are in `settings` and `notices` of every result and in `status`.

**Alternatives considered:** Update the instructions during the session. MCP has no message for it.

**Reason:** This follows the decision "Settings in every result" of 2026-11-07. Decided autonomously.

## `status` without a knowledge base is a normal result — 2026-11-14

**Decision:** `status` returns `knowledge_base: null` and a `problem` object, with no error flag.

**Alternatives considered:** An error result. Rejected: `status` is the tool that tells what is wrong, so it must succeed.

**Reason:** An agent or a person can always read the fix. Decided autonomously.

## The knowledge base name is data, not text of the server — 2026-11-14

**Decision:** The name and every other value from a file are cut at 80 characters and at the first line break before they go into the instructions.

**Alternatives considered:** Insert the value as it is. Rejected: a document of a shared knowledge base could add lines (and so advice) to the instructions of every reader. This is a prompt-injection path.

**Reason:** The instructions stay a fixed text with short values. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously:**
- The folder is found at the first use (see `project-binding`). So `initialize` does not clone. If a clone is still missing, the instructions use the second form: "The knowledge base is cloned at the first tool call." An existing clone gives the full text.
- `settings` no longer holds `allowed_repo_hosts` (the spec lists exactly eight keys). The limit still applies.
- Each tool result keeps its notices as before (paused, settings-file notices, and the notices of the tool). `status` adds the notices "This repository has no remote." and the found-by notice, so `status` shows the same notices as the instructions. The no-remote and found-by notices are not added to every result.
- `status` keeps `healthy` and `checks` (for `maintenance`). The `message` is `Knowledge base "<name>" is in use. Access is <access>.` when the setup is healthy.
- The code is in `src/mcp/session.rs` and `src/mcp/instructions.rs`.
