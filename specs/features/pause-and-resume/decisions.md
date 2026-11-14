# Decisions: Pause and resume

> **Note 2026-11-11:** "Resume needs the dialog" is superseded by "Pause is a limit" below. "Two states, no timer" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Two states, no timer — 2026-11-06

**Decision:** Running or paused, no automatic resume.

**Alternatives considered:**
- A pause with a timer — a forgotten timer silently re-enables writes.

**Reason:** A kill switch must be simple and predictable.

## Resume needs the dialog — 2026-11-06

**Decision:** `resume` uses the native approval dialog.

**Reason:** An agent must not be able to switch the switch off.

## Pause is a limit — 2026-11-11

**Decision:** Pause is the limit `paused`. The person sets it in the personal config, IT in the system config, and a server user in the console. Only the layer that set it can clear it.

**Reason:** the dialog is dropped. Honest limit: a personal flag is a safety default in local mode.

## The person sets the pause by editing a file — 2026-11-14

**Decision:** The key is `paused` in `[limits]` of `~/.config/waymark/config.toml` (or `$XDG_CONFIG_HOME/waymark/config.toml`). There is no tool and no command line option. The README gives exact steps. The binary keeps `--help` and `--version` as its only commands, so no exception to `mcp-server` is needed.

**Alternatives considered:**
- `waymark pause` and `waymark resume` commands. Rejected: they break the rule of `mcp-server` that only `--help` and `--version` exist.
- A separate flag file such as `~/.config/waymark/PAUSED`. Easier to create with `touch`, but it is a new layer that `trust-model` does not know, and it needs a second rule for the strictest-wins merge.
- A tool that pauses (no resume). Rejected: no tool may change limits.

**Reason:** The file already exists as a layer in `trust-model`, it is read at every call, and the strictest-wins rule applies with no new code path. Decided autonomously.

## Pause is global, not per knowledge base — 2026-11-14

**Decision:** The personal pause stops all writes of all knowledge bases of the person. The shared file of one knowledge base can pause that knowledge base only.

**Alternatives considered:** A section per knowledge base ID in the personal file. Rejected: `trust-model` has no such layer, and a kill switch must be simple.

**Reason:** One switch, one meaning. This answers the old open question. Decided autonomously.

## Error code order: `access_denied` before `paused` — 2026-11-14

**Decision:** If `access` is `read-only` and `paused` is true, a write fails with `access_denied`. If only `paused` is true, it fails with `paused`.

**Alternatives considered:** `paused` first. Rejected: the person would resume and still get an error.

**Reason:** The code names the cause that a resume can fix. Decided autonomously.

## Write tools are hidden while paused, and a call still gives `paused` — 2026-11-14

**Decision:** While paused, `tools/list` hides the write tools (as `mcp-server` plans for `read-only` and `paused`). A call of a hidden write tool gives `paused`, not "unknown tool". The server sends `notifications/tools/list_changed` when the visible list changes between two calls.

**Alternatives considered:** Keep the tools listed and fail at call time. Rejected: the list is how the agent learns the state. Detect file changes with a file watcher. Rejected: the server reads the limits at every call already, so a check after each call is enough and needs no thread.

**Reason:** The agent sees the state in the tool list and in the error. Decided autonomously. This adds the `list_changed` notification to the plan of `mcp-server` (see the report).

## Limits are checked again after an approval wait — 2026-11-14

**Decision:** `Repo::write` reads the effective limits again after the approval answer and before the commit. A pause or `read-only` set meanwhile ends the write.

**Reason:** An approval can wait 2 minutes. A kill switch must work during that time. Decided autonomously. This adds a step to the write flow of `git-sync` (see the report).

## No timer — 2026-11-14

**Decision:** The decision "Two states, no timer" of 2026-11-06 stays valid. There is no automatic resume.

**Reason:** A forgotten timer silently re-enables writes.
