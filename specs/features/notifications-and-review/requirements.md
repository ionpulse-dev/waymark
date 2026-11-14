---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - updates
---

# Notices and review

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Notices in tool results are in scope. The review list in the web console, the per-person record on a server, and email or webhook messages are out of scope for now (see `todo.md`). See [project decisions](../../../docs/decisions.md).

## User Story

> As a person, I want to hear about problems with my documents and my knowledge base, so that I can act without watching anything.

## Personas

- **Every person** — gets notices in the results of their agent.

## Behaviour

waymark has no background process in local mode and cannot push a message to a person. It uses two ways:

- **Notices in results.** Every tool result may carry `notices` (see `mcp-server`). A notice is one short plain sentence. The server adds a notice for these problems:

| Problem | Notice |
|---|---|
| Someone else disputed documents of the person | "Someone disputed 2 of your documents: `ops/deploy`, `ops/rollback`. Run `updates` for details." |
| Someone else changed, moved, deprecated, or deleted documents of the person | "Others changed 1 of your documents: `ops/deploy`. Run `updates` for details." |
| The last fetch failed | "The last update from the remote failed. The clone may be behind." |
| Writes are paused | "Writes are paused." |

  Notices of other specs (for example "this repository has no remote" and the conflict notice of `sync`) pass through the same filter.
- **The `updates` and `status` tools.** A person or an agent can ask at any time. `updates` lists the changes. `status` lists all open notices (see `maintenance`). This is the local review list.

A document is "of the person" when `created_by` has the email of the person.

### Rules

- **Once per session.** The same notice text appears once in a session. The next result does not repeat it.
- **Until seen.** A notice about documents stays until the problem is gone (the dispute is cleared, or the change is older than the stored time) or the person ran `updates` without `since`. That call moves the stored time (see `updates`).
- **At most 3 sessions.** The same notice about documents appears in at most 3 sessions. Then it stops, until the problem changes.
- **After a pull.** The check for documents runs when the clone has a new commit since the last check, and at the first call of a session.
- **No notice, no field.** If there is no notice, `notices` is absent.
- **A failed check never fails a call.** If waymark cannot compute a notice, it adds none.
- **The `updates` result is not repeated.** A call to `updates` carries no notice about documents. It lists them.

What an agent does with a notice is the choice of the team, in its `AGENTS.md`. A suggested wording for the README: tell the person at once.

## Interfaces & Contracts

A notice is a string in the `notices` list of a result, after the notices that the tool made itself.

```json
{
  "message": "2 documents found.",
  "settings": { "access": "read-write" },
  "notices": [ "Someone disputed 1 of your documents: `backend/payments/retries`. Run `updates` for details." ],
  "results": []
}
```

The state is the file of `updates` (`<state dir>/<kb id>.json`). This spec adds the key `notice_sessions`: an object from a short hash of the notice text to the number of sessions that showed it.

## Acceptance Criteria

- **Given** someone disputes my document, **When** the next tool result is built after the next fetch, **Then** it carries a notice that names the document.
- **Given** someone changes my document, **When** the next tool result is built after the next fetch, **Then** it carries a notice.
- **Given** a failed fetch, **When** a read runs, **Then** the result carries a notice that the clone may be behind.
- **Given** the same problem in one session, **When** several results are built, **Then** the notice appears once.
- **Given** no problems, **When** a result is built, **Then** `notices` is absent.
- **Given** a notice about my documents, **When** I run `updates` without `since`, **Then** later results in the session and in new sessions do not carry it.
- **Given** the same notice in 3 sessions, **When** a fourth session starts, **Then** it does not carry the notice.
- **Given** a dispute of my document that someone then clears, **When** the next result is built, **Then** there is no notice about it.
- **Given** `paused = true`, **When** any tool runs, **Then** the result carries the notice "Writes are paused." once.
- **Given** a change by me, **When** a result is built, **Then** there is no notice about it.
- **Given** `status` runs, **When** notices are open, **Then** it lists all of them, also those already shown in this session.

## Constraints & Edge Cases

- waymark cannot reach a person who uses no agent. This is a known limit of having no background job.
- Desktop notifications do not exist. They belonged to the old app.
- Notices about documents need a Git history window. With no stored time, the window is 7 days (see `updates`).

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`).
