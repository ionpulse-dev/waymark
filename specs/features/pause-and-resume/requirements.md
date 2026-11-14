---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - mcp-server
  - write-documents
---

# Pause and resume

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. The console switch, the owner pause of a whole knowledge base on a server, and the operator pause are out of scope for now. They stay in `todo.md` (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a person, I want one switch that stops all writes of my agents at once, so that I can stop a runaway agent in seconds.

## Personas

- **Every person** — uses the switch.
- **Knowledge base owner** — can pause writes for everyone through the shared file of the knowledge base.
- **IT** — can pause writes through the system config file.

## Behaviour

- Two states only: running and paused. There is no automatic resume after a time.
- **Pause is the limit `paused`** (see `trust-model`). It is a key in the section `[limits]` of a config layer. There is no agent tool and no command to set it. The person edits a file.
- **The switch of the person** is the personal config file `~/.config/waymark/config.toml`. If `XDG_CONFIG_HOME` is set, the file is `$XDG_CONFIG_HOME/waymark/config.toml`. The person sets `paused = true` under `[limits]`. The server reads the file at every call, so the next call sees it. No restart is needed.
- **Other layers.** IT sets `paused = true` in the system file (`/Library/Application Support/waymark/config.toml`). The owner sets it in `[limits]` of `.waymark/settings.toml` (it takes effect after the next pull). The person can also pass `--paused` in the MCP entry. This takes effect at the next start of the server.
- **The strictest layer wins.** Paused is true if any layer says so. A lower layer cannot clear a pause that a higher layer set.
- **Resume** means setting `paused = false` or deleting the line, in the file that set the pause. The next call sees it.
- **While paused:**
  - Every write tool fails with `paused`. The write tools are `add`, `update_document`, `dispute`, `deprecate`, `delete`, `move`, `undo`, `sync`, `index`, `init`, `verify`, `review`, and `reject`.
  - `tools/list` does not list the write tools. The server sends `notifications/tools/list_changed` after the first call that sees the change.
  - Read tools and `status` work. Reads still pull, so the clone stays current.
  - Every result carries the notice "Writes are paused." and `settings.paused` is `true`.
- **A write that already waits** for an answer (see `write-approval`) checks the limits again after the answer. If a pause was set meanwhile, the write fails with `paused` and nothing is written.
- **Which code wins.** If `access` is `read-only` and `paused` is true, a write fails with `access_denied`, because a resume would not help.
- The person can also stop the MCP entry in the agent tool. This stops all tools of waymark.
- Pause is global for the person. It is not per knowledge base. The shared file pauses one knowledge base for all its people.

## Interfaces & Contracts

How to pause. These exact steps go into the README section "Pause writes":

```toml
# ~/.config/waymark/config.toml
[limits]
paused = true
```

- No file yet: `mkdir -p ~/.config/waymark` and create the file with the two lines above.
- A file exists with a `[limits]` section: add the line `paused = true` below `[limits]`. Do not add a second `[limits]` section.
- Resume: change the line to `paused = false`, or delete it.

The error of a blocked write tool:

```json
{
  "code": "paused",
  "message": "Writes are paused. Reads still work.",
  "next": "Only the person can resume writes: set paused = false in the file that set it, for example ~/.config/waymark/config.toml.",
  "settings": { "access": "read-write", "paused": true }
}
```

`status` and every other result show `"paused": true` in `settings` and the notice `"Writes are paused."` in `notices`.

## Acceptance Criteria

- **Given** `paused = true`, **When** an agent calls a write tool, **Then** it fails with `paused` and writes nothing.
- **Given** `paused = true`, **When** an agent reads, **Then** it works.
- **Given** `paused = true` in the system config, **When** the person sets `false` in the personal config, **Then** writes still fail.
- **Given** paused, **When** a day passes, **Then** it is still paused.
- **Given** paused, **When** `status` runs, **Then** it names the state.
- **Given** an agent, **When** it lists tools, **Then** no tool resumes writes.
- **Given** a running session, **When** the person sets `paused = true` and then `false`, **Then** the next write fails and the one after the resume works, with no restart.
- **Given** a write that waits for approval, **When** a pause is set before the answer, **Then** the write fails with `paused` and nothing is written.
- **Given** `access = "read-only"` and `paused = true`, **When** a write runs, **Then** it fails with `access_denied`.
- **Given** paused, **When** the client lists tools, **Then** the write tools are missing, and the client got a `list_changed` notification.
- **Given** a broken personal config file, **When** any tool runs, **Then** it fails with `setup_problem` and names the file.

## Constraints & Edge Cases

- In local mode, an agent with file access could edit the personal config. This is the honest limit of a personal layer. The system config file is the hard option.
- Pulls continue while paused, so the clone stays current.
- A syntax error in a config file makes every call fail with `setup_problem`, also reads. A pause that was typed wrong therefore never loosens a limit.
- A write that already passed the final check when the pause was set finishes. The pause applies from the next check.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`).
