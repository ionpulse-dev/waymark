# Decisions: Manual edits and Sync now

> **Note 2026-11-11:** "The app commits manual edits" is superseded by "The server commits manual edits" below. "Sync now" is now the `sync` tool.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## The app commits manual edits — 2026-11-06

**Decision:** The app waits about a minute, runs `lint`, commits under the person, and pushes.

**Reason:** It makes "edits in Obsidian are allowed" work without Git knowledge.

## Sync now — 2026-11-06

**Decision:** The menu has a "Sync now" entry, with `waymark sync` as the CLI equivalent.

**Reason:** A person wants to see sync run at once, also after manual edits.

## The server commits manual edits — 2026-11-11

**Decision:** The `sync` tool and every write flow commit pending manual edits. There is no background watcher.

**Reason:** no app and no background job in local mode. Alternative considered: a file watcher while the server runs. It is an open question.

## `sync` commits with the verb `sync` and a fixed message — 2026-11-14

**Decision:** The commit of manual edits is `waymark: sync "<title>"` for one document, or `waymark: sync "<N> manual edits"` for several. The old message "Manual edit" is dropped.

**Alternatives considered:** "Manual edit" (rejected: `git-sync` fixes one message format and the list of verbs).

**Reason:** One format lets `activity` and `undo` read every commit. Decided autonomously.

## `lint` runs first, and a failing file is not committed — 2026-11-14

**Decision:** `sync` runs `lint` on every pending document before it commits. A document with an error is reported under `blocked` and is not committed. Documents that pass are committed and pushed. If nothing passes and something is blocked, `sync` fails with `validation_failed` and does not pull or push.

**Alternatives considered:**
- Fail the whole call when one file fails (rejected: one bad file in Obsidian would stop all good edits).
- Commit everything and let `lint` warn (rejected: bad documents would reach the remote).

**Reason:** The person keeps working, and the remote stays clean. A failing file is never changed or lost. Decided autonomously.

## Only `.md` documents are committed — 2026-11-14

**Decision:** `sync` commits only `.md` documents below the root. Other files are listed under `skipped`. Files in `.waymark/` are never committed by `sync`, so an edit to `.waymark/settings.toml` cannot loosen a shared limit through waymark.

**Alternatives considered:** Commit everything below the root (rejected: attachments belong outside the knowledge base, and the shared settings file must not change through the tools).

**Reason:** This keeps the repository clean and follows the trust model. Decided autonomously.

## `sync` sets `updated` and `updated_by` — 2026-11-14

**Decision:** For a changed document that passed lint, `sync` sets `updated` to today and `updated_by` to the author, if they differ. It touches no other property and no body text. A new file keeps what the person wrote.

**Alternatives considered:** Leave them (rejected: the standard says every change sets them, and `author-mismatch` would warn). Fill all missing fields of a new file (rejected: guessing a title or description is not safe).

**Reason:** Manual edits then follow the same rules as writes. Decided autonomously.

## No timer; the server commits on `sync` and before writes — 2026-11-14

**Decision:** The server does not commit pending edits after a quiet minute. It commits them on `sync` and before every other write.

**Alternatives considered:** A watcher while the server runs (rejected: waymark has no background job in local mode, an edit that is half done could be committed, and the session may end at any time).

**Reason:** Answers the open question. It keeps the design simple and predictable. Decided autonomously.

## A helper for people without agents is deferred — 2026-11-14

**Decision:** No helper command now. A person without an agent can run `git` or wait for a session.

**Reason:** MCP is the only interface (see `mcp-server`). Demand is unknown. See `todo.md`. Decided autonomously.

## A failed pending edit does not stop an agent write — 2026-11-14

**Decision:** Before a write, passing manual edits are committed in their own commit. A failing edit stays in the folder, adds a notice, and does not stop the write. If the write changes that same file, it fails with `stale_base` and a `next` sentence.

**Alternatives considered:** Fail every write while a bad edit exists (rejected: one bad file would block all agent writes). Overwrite the edit (rejected: it would lose the work of the person).

**Reason:** The agent write stays safe, and nothing of the person is lost. Decided autonomously.

## `sync` stays listed when writes are not allowed — 2026-11-14

**Decision:** `sync` is listed when `access` is `read-only` or writes are paused. In that case it only pulls. `tools_denied` can still hide it. The plan of `mcp-server` (step 4) hides write tools in those states, so it needs an exception for `sync`.

**Alternatives considered:** Hide `sync` like other write tools (rejected: the criterion requires that it pulls in read-only mode, and a person with a read-only limit still wants the newest documents).

**Reason:** Pulling is a read. This is a conflict with `mcp-server` that is reported. Decided autonomously.

## Hooks on `Repo::sync` and `Repo::write` — 2026-11-14

**Decision:** `Repo::sync` takes a hook that runs after the lock and before the pull. `Repo::write` calls an installed pending-edit hook after the fetch, and refuses with `stale_base` if a path it changes has uncommitted edits. `Context` installs the hook. The `sync` tool file `src/tools/sync.rs` belongs to this spec.

**Alternatives considered:** Each write tool calls `manual::commit_pending` (rejected: tools would forget it, and `git-sync` already owns the write flow).

**Reason:** The write flow stays in one place. These are additions to `git-sync` and are reported. Decided autonomously.

## Implementation notes — 2026-11-14

**Decided autonomously.**

- **No hook object.** The plan had `Repo::install_pending_hook`. The code is simpler: `writes::execute` (the one write path) calls `manual::commit_before_write` after the fetch and inside the lock. `gitsync` stays free of document rules. A `preview` write does not run it.
- **`stale_base` for unpublished edits** is raised in `writes::read_doc`, before the tool builds its change. After the hook, a document that is still changed in the working tree is a blocked edit. The write then fails, also when the new content would be invalid. `execute` also checks the planned files after the build.
- **`sync` has the kind `read`.** The registry lists read tools in every state, which is what the requirements ask. The tool itself calls `check_tool("sync", write)` and, if writes are not allowed, only pulls (`Repo::pull_only`) and adds a notice. `tools_denied` still hides it.
- **Preview** reads the working tree and runs lint. It does not fetch, so nothing is pulled.
- **`lint::run` takes a predicate** (`include`) instead of a `Scope`, so `sync` can check exactly the pending documents. Duplicate titles are still found against all documents.
- **`pulled`** is the number of commits that the pull brought in, as `git rev-list --left-right` counts them before the merge. A rebase of local commits is not counted twice.
- **Hand moves** show as one `deleted` and one `new` in one commit. Git finds the rename, and `show` follows it.
- **Approval** for `sync` uses `writes::approve`. It fails closed until `write-approval` exists. Test T7 waits for it.

## `sync` applies the link rules — 2026-11-14

**Decision (product owner):** `sync` runs the same checks as a write tool, links included. A broken body link that a hand edit adds blocks the edit. A document deleted by hand that others still link to is blocked. A link that was already broken in the last commit stays a warning.

**Reason:** The knowledge base should stay consistent, also when someone edits in Obsidian or by hand. `lint` already blocked errors. The link rules of `write-documents` now hold for every way into the knowledge base.
