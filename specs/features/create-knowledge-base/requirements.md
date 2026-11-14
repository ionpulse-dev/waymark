---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - join-knowledge-base
  - plain-git-access
---

# Create a knowledge base

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Creating the remote repository through a Git host API, and a check whether the repository is public, are out of scope for now (see [project decisions](../../../docs/decisions.md)). They are in `todo.md`.

## User Story

> As the first person of a company, a team, or just myself, I want to create a knowledge base in a few minutes, so that others and my agents can use it.

## Personas

- **Knowledge base owner** — creates the knowledge base and sets the shared settings.
- **Private person** — creates a private knowledge base for their own AI memory.

## Behaviour

1. The person creates an empty, private repository on the Git host. waymark does not create repositories.
2. The person adds an MCP entry with `--repo <url>` (see `join-knowledge-base`) and starts a session. The server clones the empty repository. The state is `empty`.
3. `status` reports that the repository is empty and that the `init` tool can set it up.
4. The `init` tool creates the structure, commits, and pushes. The server then has the state `ready`, in the same session.

`init` is a write tool. The limits apply: `access = "read-only"` blocks it (`access_denied`), `paused` blocks it (`paused`), `tools_denied = ["init"]` hides it, and `approval.required_for` (`init` or `write`) asks first. The `folders` limit does not apply to `init`.

### The repository must be empty

A repository is empty when it has no commit, or when all tracked files are among `README.md`, `README`, `LICENSE`, and `LICENSE.md`. These are the files that a Git host creates. Any other file makes `init` fail with `validation_failed` and no change. `init` never overwrites a file: the empty rule means no file of the layout can exist. The check runs again after the fetch inside the write flow, so two people who call `init` at the same time cannot both succeed.

### A folder with the marker and no settings file

A repository that has a `.mkbase` folder at its root or in `knowledge/`, and no `waymark.toml` there, was made for the older MKBase standard. `init` adopts it. It writes only `.mkbase/waymark.toml` (or `knowledge/.mkbase/waymark.toml`) and commits and pushes that one file. It touches no document and no other file, and it does not create `AGENTS.md`, `.gitignore`, `.obsidian`, or folders. The inputs `name`, `slug`, `email_domains`, `limits`, `approval`, and `preview` work as below. The inputs `folders` and `audience` have no meaning here and give `invalid_arguments`. The result has `adopted: true` and `files` with the one path. If `waymark.toml` exists, the repository is already a knowledge base (`validation_failed`, no change).

### Inputs

| Input | Type | Required | Stored in | Default |
|---|---|---|---|---|
| `name` | string, 1 to 80 characters, not blank | yes | `.mkbase/waymark.toml` `name` | none |
| `slug` | string, lowercase letters, digits, and single `-`, at most 40 characters | no | `.mkbase/waymark.toml` `slug` | derived from `name`: lowercase, every run of other characters becomes one `-`, no `-` at the ends. If nothing is left: `knowledge-base`. |
| `email_domains` | array of strings (domain names) | no | `[people] email_domains` | none |
| `limits` | object with the keys of `[limits]` (see `knowledge-base-repo`) | no | `[limits]` | none |
| `approval` | object with `required_for` (array of strings) and `via` (`elicitation` or `console`) | no | `[approval]` | none |
| `folders` | array of folder paths below `knowledge/`, at most 50 | no | folders in `knowledge/` | none |
| `audience` | `personal` or `team` | no | which plain-Git rules go into `AGENTS.md` (see `plain-git-access`) | `team` |
| `preview` | boolean | no | not stored | `false` |

An unknown key, a wrong type, a bad slug, or a folder path with `..`, an absolute path, or a name that breaks the file-name rule of the standard is `invalid_arguments` with a `next` sentence that names the field. The server does not decide the audience from the email domains.

### What `init` does

1. Generates the UUID of the knowledge base (version 4) once.
2. Writes these files, relative to the repository root:

   | File | Content |
   |---|---|
   | `AGENTS.md` | The plain-Git rules for the audience (see `plain-git-access`). |
   | `.obsidian/app.json` | `{ "useMarkdownLinks": true, "newLinkFormat": "relative" }` (as [MKBase 1.0 section 11](../../../standard/mkbase-1.0.md) recommends). |
   | `.gitignore` | The lines `.obsidian/*` and `!.obsidian/app.json`. |
   | `knowledge/.mkbase/version` | `1.0` and a final line end. |
   | `knowledge/.mkbase/waymark.toml` | `schema = 1`, `name`, `id`, `slug`, `verification = false`, and the sections that the inputs set. |
   | `knowledge/.mkbase/exclude` | Empty. |
   | `knowledge/<folder>/.gitkeep` | One for each folder of `folders` (Git does not track empty folders). |

   A file that Git hosts created (`README.md`) stays as it is.
3. Commits all of them in one commit, `waymark: init "<name>"`, under the author of the person, and pushes.

## Interfaces & Contracts

Input:

```json
{ "name": "Example", "slug": "example", "email_domains": ["example.org"], "audience": "team", "folders": ["backend", "ops"], "limits": { "max_results": 5 }, "preview": false }
```

Result (`data` fields, besides `message`, `settings`, and `notices`):

```json
{
  "id": "0b1f3c9e-5a77-4d1e-9a41-2f6f0c8d7e11",
  "name": "Example",
  "slug": "example",
  "audience": "team",
  "root": "knowledge",
  "files": ["AGENTS.md", ".gitignore", ".obsidian/app.json", "knowledge/.mkbase/version", "knowledge/.mkbase/waymark.toml", "knowledge/.mkbase/exclude", "knowledge/backend/.gitkeep", "knowledge/ops/.gitkeep"],
  "commit": "a1b2c3d",
  "pushed": true
}
```

With `preview: true`, `id` and `commit` are `null`, `pushed` is `false`, and nothing is written. Errors: `validation_failed` (not empty, or already a knowledge base), `invalid_arguments`, `access_denied`, `paused`, `approval_denied`, `approval_unavailable`, `setup_problem` (no author, Git not working, state `broken`), `sync_failed`. A `validation_failed` result for a repository that is already a knowledge base says: "This repository is already a knowledge base. Nothing was changed."

## Acceptance Criteria

- **Given** an empty remote repository and a running server, **When** `init` runs with a name, **Then** the repository has the skeleton and the first commit is pushed.
- **Given** the result, **When** another person adds the entry with the URL, **Then** they need not enter any shared value.
- **Given** `init` ran, **When** it runs again, **Then** it fails with `validation_failed` and changes nothing.
- **Given** a repository that is not empty, **When** `init` runs, **Then** it fails and changes nothing, and no existing file is overwritten.
- **Given** a repository with only a `README.md` from the Git host, **When** `init` runs, **Then** it succeeds and the `README.md` is unchanged.
- **Given** the new repository, **When** it is opened in Obsidian, **Then** it works as a vault.
- **Given** `access = "read-only"`, **When** `init` runs, **Then** it fails with `access_denied`.
- **Given** `preview: true`, **When** `init` runs, **Then** nothing is written or committed.
- **Given** `init` succeeded, **When** the same session calls `search` or `add`, **Then** the tools work with no restart.
- **Given** a bad input (slug, folder, unknown key), **When** `init` runs, **Then** it fails with `invalid_arguments`, names the field, and changes nothing.
- **Given** a repository with a `.mkbase` marker and documents but no `waymark.toml`, **When** `init` runs with a name, **Then** only the `waymark.toml` is added and the state is `ready` in the same session.
- **Given** that repository, **When** `init` runs with `folders` or `audience`, **Then** it fails with `invalid_arguments` and changes nothing.
- **Given** two people who call `init` at the same time, **When** both end, **Then** exactly one commit of `init` exists.

## Constraints & Edge Cases

- The repository should be private. This is advice in the README. The server does not check it.
- A private person can create a knowledge base on github.com in the same flow.
- There is no terminal flow and no prompt. The agent or a client form collects the inputs.
- `undo` cannot revert an `init` commit (see `activity-and-undo`). The person deletes the repository or resets it on the Git host.
- The default branch is whatever the clone has. `init` pushes to it and sets the upstream.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`): `init` does not create the remote repository, and the defaults of all inputs keep the shared limits optional.
