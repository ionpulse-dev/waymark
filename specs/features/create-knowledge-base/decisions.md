# Decisions: Create a knowledge base

> **Note 2026-11-11:** "Interactive for humans, flags for scripts" and "`init` creates the id and runs `setup`" are superseded by "`init` is a tool" below.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Interactive for humans, flags for scripts — 2026-11-06

**Decision:** `init` asks questions only when a human runs it in a terminal or the app. Every question has a flag and a default.

**Reason:** Keeps the product generic and scriptable for company rollout, while agents never meet a prompt.

## `init` creates the id and runs `setup` — 2026-11-07

**Decision:** The UUID is generated once by `init` and stored in the shared file, so every clone knows it. The slug stays a human label.

**Reason:** Binding to a project folder needs a stable identifier.

## `init` is a tool — 2026-11-11

**Decision:** `init` is an MCP tool that runs only on an empty repository. There is no terminal flow and no `setup`.

**Reason:** MCP is the only interface. The first start of the server already clones the repository. Alternative considered: a terminal command for the owner. It would need a second interface.

## Scope: `init` on an empty repository, no remote creation — 2026-11-14

**Decision:** `init` works on an existing empty repository that the person created. It does not create the remote repository through a Git host API, and it does not check whether the repository is public. Both are in `todo.md`.

**Alternatives considered:** Create the repository through the API (rejected: it needs a Git host token, and waymark never holds secrets). Warn in `status` when the repository is public (rejected for now: the host does not tell without an API call).

**Reason:** The scope is local mode, and Git handles the login. Decided autonomously.

## What "empty" means — 2026-11-14

**Decision:** A repository is empty if it has no commit, or if all tracked files are among `README.md`, `README`, `LICENSE`, and `LICENSE.md`. The check is `kb::repo_is_empty`. `join-knowledge-base` uses it for the state `empty`.

**Supersedes:** "runs only on a repository that has no `.mkbase` folder and no `knowledge/` folder" in the first version of the requirements.

**Alternatives considered:** Only a repository with no commit (rejected: a Git host adds a `README.md` and a license when it creates the repository, and the person should not have to remove them). The first rule, no `.mkbase` and no `knowledge/` (rejected: a repository with code would be allowed, and `AGENTS.md` or `.gitignore` could be overwritten).

**Reason:** The brief says an empty repository only. The list covers what Git hosts create, and it makes an overwrite impossible. Decided autonomously.

## `init` is a commit verb of its own — 2026-11-14

**Decision:** The commit message is `waymark: init "<name>"`. `init` is a ninth verb. `undo` refuses a commit with this verb, and `activity` shows it.

**Supersedes:** the list of eight verbs in `git-sync` (`add`, `update`, `dispute`, `deprecate`, `delete`, `move`, `undo`, `sync`). The list in `git-sync` and `activity-and-undo` must add `init`.

**Alternatives considered:** The verb `add` with the name as title (rejected: `undo` would try to revert the whole structure, and `activity` would count it as a document).

**Reason:** It is one commit that is not a document change. Decided autonomously.

## `init` runs inside the write flow — 2026-11-14

**Decision:** `init` calls `gitsync::Repo::write`. The emptiness check and the input check run in step 5 (checks), after the fetch. `gitsync` must handle a clone with no commit: no upstream, an unborn branch, and `git push -u origin HEAD` for the first push.

**Alternatives considered:** A separate path for `init` (rejected: it would skip the lock, the limits, and the approval).

**Reason:** Limits, lock, approval, and the author come for free, and two calls at once are safe. The change to `git-sync` goes in its report. Decided autonomously.

## The audience defaults to `team` — 2026-11-14

**Decision:** `audience` is `team` unless the call says `personal`. The email domains do not decide it.

**Supersedes:** "derived from the email domains, else `team`" in the first version (both branches gave `team`).

**Reason:** The rule was empty. A clear default is simpler. The person can say `personal`. Decided autonomously.

## The `folders` limit does not apply to `init` — 2026-11-14

**Decision:** `init` writes outside any document folder. The `folders` limit does not apply to it.

**Reason:** The limit guards documents. Before `init` there is no document.

## The settings file is written with a TOML library — 2026-11-14

**Decision:** `waymark.toml` is built from a `Settings` struct and written with `toml::to_string`. It has no comments. Only keys that the call set are written, plus `name`, `id`, `slug`, and `verification`.

**Alternatives considered:** A text template with placeholders (rejected: a name with a quote or a backslash breaks it).

**Reason:** Correct escaping. The key list is in `knowledge-base-repo`. Decided autonomously.

## A new test harness constructor — 2026-11-14

**Decision:** `integration-testing` gets `World::empty()`: a bare remote with no commit and no knowledge base. The tests of this spec need it.

**Reason:** `World::new()` initializes a knowledge base in the remote. Reported to the owner of `integration-testing`.
