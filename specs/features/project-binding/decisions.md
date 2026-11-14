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

_Note 2026-11-14: the personal default is a user-level entry that names `--repo` or `--path`. An entry without both is no longer valid (see "Explicit location: `--repo`, `--path`, or both")._

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

**Superseded by:** Explicit location: `--repo`, `--path`, or both

## The slug of a URL — 2026-11-14

**Decision:** The default folder name comes from the last part of the URL, by the rule in `requirements.md`. The `slug` in `settings.toml` is not used before the first clone, because the file does not exist yet.

**Alternatives considered:** Clone to a temporary folder, read the slug, and move it. Rejected: more steps, and a path that depends on file content is harder to predict.

**Reason:** The person can predict the path from the URL. Decided autonomously.

**Superseded by:** Explicit location: `--repo`, `--path`, or both

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

## Explicit location: `--repo`, `--path`, or both — 2026-11-14

**Supersedes:** How the server finds the knowledge base; The slug of a URL

**Decision:** The entry must name the memory repository with `--repo`, `--path`, or both. The server has no default folder under `~/Projects`, no scan, and no search of the working folder and the folders above it. Without `--path`, `--repo` clones into `~/.waymark/<name>`, where `<name>` is the last part of the URL without `.git`. The owner is not part of the name. The table in `requirements.md` has the nine cases. An entry with neither argument is `setup_problem`. `found_by` has two values: `path_argument` and `repo_argument`.

**Alternatives considered:**
- Keep `~/Projects/<slug>-agent-memory`. Rejected by the product owner: too opinionated for people who do not use `~/Projects`.
- Keep the scan and the working-folder search as a fallback. Rejected: it is a guess, and a guess can pick the wrong memory repository.
- Put the owner in the name (`~/.waymark/<owner>/<name>`). Rejected: SSH and HTTPS URLs of one repository must give one folder, and the owner is not always in the URL. A name collision is a clear error with the fix `--path`.

**Reason:** People without Git knowledge paste one URL and never handle a clone. People who want a visible folder add `--path`. The rules are short and have no guessing.

## An entry with neither argument starts the server and reports the problem — 2026-11-14

**Decision:** "Fails at start" means that the server starts, `status` works, and every other tool returns `setup_problem` with the sentence "The MCP entry names no memory repository" and a `next` that names `--repo` and `--path`. The process does not exit.

**Alternatives considered:** Exit with a non-zero code and a message on stderr. Rejected: agent tools show a dead server as "failed to connect" and hide stderr, so the person sees no hint. This is the reason of the earlier decision "How the server finds the knowledge base". Confirmed by the product owner on 2026-11-14: report the error, do not exit.

## Same remote: compare `host/path` of `origin` — 2026-11-14

**Decision:** The check reads only the `origin` remote. Both URLs are turned into `host/path`: scheme, user, password, port, a trailing `/`, and a trailing `.git` are removed, and the host is lower case. The path keeps its case. So `git@github.com:acme/k.git`, `ssh://git@github.com/acme/k`, and `https://github.com/acme/k.git` are the same. A folder without an `origin` remote passes, as before.

**Alternatives considered:** Compare the raw strings after removing `.git` (the old rule). Rejected: SSH and HTTPS forms of one repository would count as different, and people would see a false mismatch. Compare all remotes. Rejected: the clone always has `origin`, and other remotes are the person's own.

**Reason:** The person may paste the URL in the other form than the one that made the clone.

## The `slug` setting stays — 2026-11-14

**Decision:** `slug` in `.waymark/settings.toml` stays as a short label of the memory repository (`init` and `status` use it). It no longer names a folder, and the comment in the template says so.

**Alternatives considered:** Remove the key. Rejected: it changes the Waymark Memory Standard and every existing settings file for no gain.

## A folder that is empty counts as missing — 2026-11-14

**Decision:** An existing folder with no entries is a clone target. This covers a folder that the person made for the vault or that Git left behind.

**Reason:** The table of the product owner treats "missing or empty" the same. The clone goes to a temporary folder and is renamed, so the empty folder is replaced and never mixed.

## `find_root` never climbs out of a Git repository — 2026-11-14

**Decision:** `memrepo::layout::find_root` still looks above its start folder for a `.waymark` marker, but it stops at a folder that has a `.git`. Found when the default folder moved to `~/.waymark/<name>`: the folder `~/.waymark` itself looked like a marker for every clone without one (an empty repository), and the settings were read from the wrong place.

**Reason:** The default clone folder lives inside a folder that is named like the marker. A repository is the boundary of its memory repository. Unit test: `the_search_above_stops_at_a_git_repository`.
