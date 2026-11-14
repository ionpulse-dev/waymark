---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - project-binding
---

# Session start

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Server mode, with many memory repositories and sign-in, is out of scope for now (see [project decisions](../../../docs/decisions.md)).

## User Story

> As a person, I want my agent to know at the start of a session which memory repository it uses and which limits apply, so that I am never surprised.

## Personas

- **Every person** — can see the memory repository in use and the limits.

## Behaviour

When an MCP client connects, the server answers `initialize` with its **instructions**. They are a short, neutral summary of facts. They hold no behavior advice: no "should", no "when to read or write".

The instructions hold:

- what waymark is, in one sentence, and the meaning of a document, a folder, and an ID,
- the memory repository in use (name and ID),
- the limits that apply (see `trust-model`),
- open notices.

The same facts are in the `status` tool, which an agent or a person can call at any time. Every tool result also carries the current `settings`, so a change of a limit applies at once.

- The instructions are made once, at `initialize`, from local data only. They make no network call and never wait for a pull. A first clone (see `project-binding`) happens before `initialize`.
- The instructions say "at session start". Later changes show in `settings` and `notices` of each result.
- Whether the agent tells the person, and what it does first in a session, is the choice of the team, in its `AGENTS.md`.
- The summary only informs. Every tool checks the same limits itself, so an ignored summary cannot allow a forbidden action.
- The instructions have at most 1200 characters. With default limits they have fewer than 900.

### The text

With a memory repository:

```text
waymark is a memory repository of Markdown documents in a Git repository. A document is one Markdown file. A folder groups documents. The ID of a document is its path below the memory repository root, without ".md".
Memory repository: "Example Company" (id 6f1c2a9e-3b7d-4c55-9a0e-1d2f3a4b5c6d).
Limits at session start: access read-write; writes paused: no; search returns at most 5 documents and 800 tokens; show opens at most 20 documents per session; folders: all; tools switched off: none; approval required for: none.
Notices: none.
Every result carries the current settings and notices. Tool "status" returns these facts.
```

Rules for the text:

- The lines are as above. Values change. A list is comma-separated. An empty list is `none`, and an empty `folders` list is `all`.
- `Notices:` lists at most 3 notices. A longer list ends with `and <n> more in status`.
- Names and notices that come from files (the memory repository name) are cut at 80 characters.
- The text holds none of the words `should`, `must`, `always`, `never`, `please`, `prefer`, and `you`.
- Every sentence names a fact. None names an action.

Without a memory repository (see `project-binding`):

```text
waymark is a memory repository of Markdown documents in a Git repository.
No memory repository is set up: <message of the problem>
Tool "status" returns the fix.
```

### The `status` tool

`status` is read-only and works in every state. `maintenance` adds `checks` and the binary version. This spec defines these fields of the result:

```json
{
  "message": "Memory repository \"Example Company\" is in use. Access is read-write.",
  "settings": {
    "access": "read-write",
    "paused": false,
    "tools_denied": [],
    "folders": [],
    "max_results": 5,
    "max_tokens": 800,
    "max_documents_opened": 20,
    "approval": { "required_for": [], "via": "elicitation" }
  },
  "notices": [],
  "memory_repository": {
    "name": "Example Company",
    "id": "6f1c2a9e-3b7d-4c55-9a0e-1d2f3a4b5c6d",
    "slug": "example",
    "path": "/Users/jane/Projects/example-agent-memory",
    "remote": "git@example.org:team/waymark-agent-memory.git",
    "found_by": "repo_argument"
  }
}
```

| Field | Type | Meaning |
|---|---|---|
| `memory_repository.name`, `id`, `slug` | string | From `.waymark/settings.toml` |
| `memory_repository.path` | string | Absolute path of the clone |
| `memory_repository.remote` | string or null | The `origin` URL with no user and no password. `null` if there is no remote. |
| `memory_repository.found_by` | string | One of `path_argument`, `repo_argument`, `working_folder`, `default_folder` |
| `settings` | object | The effective limits. The same object is in every result. |

The `settings` object has exactly the keys above, in that order. It holds the limits that matter to an agent. It does not hold `allowed_repo_hosts`.

If no memory repository is bound, `memory_repository` is `null` and the result has `problem`:

```json
{
  "message": "No memory repository is set up.",
  "memory_repository": null,
  "problem": {
    "code": "setup_problem",
    "message": "No memory repository is set up. waymark found none in the working folder, and none or several in ~/Projects.",
    "next": "Add --repo <git url> or --path <folder> to the MCP entry."
  },
  "settings": { "access": "read-write", "paused": false, "tools_denied": [], "folders": [], "max_results": 5, "max_tokens": 800, "max_documents_opened": 20, "approval": { "required_for": [], "via": "elicitation" } },
  "notices": []
}
```

`status` is not an error in this case. The result has no error flag.

### Notices at session start

A notice is a plain sentence. These can exist at start, from local data:

- "Writes are paused." (see `pause-and-resume`)
- "This repository has no remote." (see `git-sync`)
- "The settings file has keys that this version does not know: <keys>." (see `memory-repository`)
- "The memory repository was found in the working folder." and the same for the default folder. It is a notice because the person did not name it.

## Acceptance Criteria

- **Given** a bound project, **When** a client connects, **Then** the instructions name the memory repository and carry the effective limits.
- **Given** the instructions, **When** they are read, **Then** they hold no advice on when to read or write.
- **Given** writes are paused, **When** a client connects, **Then** the instructions and a notice say so.
- **Given** an agent that ignores the instructions, **When** it runs a write, **Then** the server still enforces all limits.
- **Given** no memory repository is set up, **When** a client connects, **Then** the instructions say so and `status` shows how to fix it.
- **Given** a limit changes during a session, **When** any tool runs, **Then** `settings` in the result shows the new value.
- **Given** a remote that is not reachable, **When** a client connects, **Then** `initialize` still answers at once.
- **Given** default limits, **When** the instructions are built, **Then** they have fewer than 900 characters, and never more than 1200.
- **Given** `status`, **When** it runs, **Then** its facts equal the facts in the instructions, with the current values.

## Constraints & Edge Cases

- Some clients ignore server instructions. The facts are in `status` and in every result. The manual tests check which clients use them.
- The instructions stay short, because they use context in every session.
- The instructions are a snapshot. A pause set after `initialize` shows in the next result, not in the instructions.
- A name with a line break is cut at the first line break, so a file cannot add lines to the instructions.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). Which clients use server instructions is checked by the manual tests in `test.md`.
