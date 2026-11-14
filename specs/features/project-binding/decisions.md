# Decisions: Project binding

> **Note 2026-11-11:** "Bind a folder to one knowledge base" stays valid. "The binding holds a UUID", "Name and URL in the binding, setup by confirmation", and "A personal default for private use" are superseded by "The binding is an MCP entry" below.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Bind a folder to one knowledge base — 2026-11-07

**Decision:** A hidden `.waymark/` folder in the project root binds it to exactly one knowledge base.

**Alternatives considered:**
- Only one active knowledge base at a time — too rigid, and a closed one cannot be switched per session.
- A global active knowledge base — mixing needs only one wrong switch.
- Guess from the working directory — implicit and fragile.

**Reason:** Mixing needs two knowledge bases in one session, and a session sees only the one of its folder.

## The binding holds a UUID — 2026-11-07

**Decision:** The binding is a UUID that `init` stores in the shared file. The agent passes it on every call.

**Alternatives considered:**
- The slug — two organizations may use the same slug.

**Reason:** Explicit and robust. The agent knows which knowledge base it works in, and the skill stays static and version-matched.

## Name and URL in the binding, setup by confirmation — 2026-11-07

**Decision:** The binding also holds the name and Git URL, used for messages and a setup offer. A person must confirm any clone.

**Reason:** Easy joining, without the risk of a poisoned knowledge base.

## A personal default for private use — 2026-11-07

**Decision:** A person can set a default knowledge base that works in every folder. A project binding wins. The default is off unless set.

**Reason:** Private use works everywhere. Binding a folder changes which knowledge base is used, so an agent must not do it.

## The binding is an MCP entry — 2026-11-11

**Decision:** A project binds a knowledge base by a project-level MCP entry with `--repo`. A user-level entry is the personal default. There is no `.waymark/` folder and no `link` command.

**Reason:** the config of the agent tool is the natural place, it is committed with the project, and one server serves one knowledge base. Alternative considered: a hidden folder with an ID. It needs a CLI and an app to bind safely.

## How the server finds the knowledge base — 2026-11-14

**Decision:** The order is: `--path`, then `--repo` (default folder `~/Projects/<slug>-agent-memory`), then the working folder and its parents, then exactly one `~/Projects/*-agent-memory`. This fills in the decision "One knowledge base per process" of `specs/foundation/mcp-server/decisions.md`. The first match wins. No match is not a crash: the server starts without a knowledge base, `status` works, and every other tool gives `setup_problem` with a `next` sentence.

**Alternatives considered:**
- Exit with an error. Rejected: the agent tool shows a dead server with no hint, and `status` is the place that tells what to fix.
- Always require `--repo` or `--path`. Rejected: a person who works inside the knowledge base folder, or a private person with one knowledge base, would need arguments for nothing.
- Pick the newest of several default folders. Rejected: guessing mixes knowledge bases.

**Reason:** Predictable, short rules, and a clear failure. The working folder is read only to find a marker. The binding itself never depends on it after start. Decided autonomously.

## The slug of a URL — 2026-11-14

**Decision:** The default folder name comes from the last part of the URL, by the rule in `requirements.md`. The `slug` in `settings.toml` is not used before the first clone, because the file does not exist yet.

**Alternatives considered:** Clone to a temporary folder, read the slug, and move it. Rejected: more steps, and a path that depends on file content is harder to predict.

**Reason:** The person can predict the path from the URL. Decided autonomously.

## One decision table of start errors — 2026-11-14

**Decision:** The codes at start are `setup_problem` for identity and layout problems, `sync_failed` for a failed clone, and `access_denied` for a host that `allowed_repo_hosts` refuses.

**Alternatives considered:** `setup_problem` for all. Rejected: the host check is a limit, and `trust-model` says a limit gives `access_denied`.

**Reason:** The agent and the person see which kind of fix is needed. Decided autonomously.

## The host check covers the `origin` of an existing folder — 2026-11-14

**Decision:** `allowed_repo_hosts` is checked for the `--repo` URL before a clone, and for the `origin` of an existing folder (also when the folder was found by the working folder rule). A folder with no `origin` passes.

**Alternatives considered:** Check only before a clone (the original text). Rejected: a poisoned project that contains a `.waymark/` folder would be used without a check.

**Reason:** The working-folder rule would otherwise bypass the limit. Decided autonomously.

## The origin must match `--repo` — 2026-11-14

**Decision:** If `--repo` and an existing folder are both given, the `origin` of the folder must equal the URL. Both are compared after removing a trailing `/` and `.git`. A different form of the same address (for example `https` against `ssh`) counts as different.

**Alternatives considered:** Ignore the URL when the folder exists. Rejected: the entry would claim one knowledge base and use another. Compare by host and path. Rejected: more rules for little gain.

**Reason:** A wrong match is a `setup_problem` with a clear `next`, and the person can fix the entry. Decided autonomously.

## Clone to a temporary folder — 2026-11-14

**Decision:** The first clone goes to `<path>.waymark-clone` and is renamed to `<path>` when it succeeds. A leftover temporary folder is deleted at the next start before a new clone.

**Alternatives considered:** Clone directly into the path. Rejected: a cut clone (the client kills the server at its start time-out) would look like an existing folder and fail later with confusing errors.

**Reason:** The start time-out of agent tools is a real risk for large repositories. Decided autonomously.

## Linked worktrees of the knowledge base are not supported — 2026-11-14

**Decision:** If `.git` of the knowledge base repository is a file (a linked worktree), the server gives `setup_problem`. A worktree of the code project is no problem, because the entry file is part of the project.

**Alternatives considered:** Support it with `git rev-parse --git-common-dir` for the lock path. Deferred: no use case yet.

**Reason:** `git-sync` puts the lock in `.git/`. This answers the open question about worktrees. Decided autonomously.

## Limits per knowledge base are cut — 2026-11-14

**Decision:** The section "Limits per knowledge base" (a personal config section keyed by the ID) is moved to `todo.md`.

**Alternatives considered:** Build it now. Rejected: `trust-model` has no such layer. The shared file of a knowledge base already holds limits for that knowledge base, and the project entry can hold arguments such as `--access`.

**Reason:** Keep the first release small. Decided autonomously.

## Agent-tool behavior is verified by manual tests — 2026-11-14

**Decision:** The claims that a project entry overrides a user entry of the same name, and that a tool asks before it starts a project-level server, are not specified as waymark behavior. They are manual tests per tool. The README states only what the tests confirm.

**Reason:** They belong to the agent tools, and waymark cannot test them. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously:**
- The folder is found at the first use of the repository, not at process start, and then kept. The server answers `initialize` at once, so a slow first clone cannot hit the start time-out of the client. The result is the same: one knowledge base for the whole process.
- The binding code is `src/mcp/binding.rs` (`locate`, `check_folder`, `clone_checked`) and `src/mcp/repo_url.rs`. `Context::repo` calls them. There is no `Binding` struct. `Context::found_by` gives the value that `status` shows.
- A knowledge base found in the working folder uses the Git root of the matching folder as the repository folder.
- A URL goes into a message only through `repo_url::redact`.
- The identity check runs at every `Context::kb` call, because the shared file is read from disk at each call.
