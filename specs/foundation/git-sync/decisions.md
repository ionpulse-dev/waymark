# Decisions: Git sync

> **Note 2026-11-11:** "The running app syncs in the background" is superseded by "No app, pull in the session" below. "Always fetch before a write, check the base version" and "One lock for all commands" stay valid. The lock is now a file in `.git/`, and the commands are MCP tools.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Use the `git` binary — 2026-11-06

**Decision:** waymark calls the `git` binary. If it is missing, setup starts Apple's installer for the command line tools.

**Alternatives considered:**
- A pure-library Git (`go-git`, first choice) — a wrapper of our own for rebase and login, and risk of gaps.

**Reason:** Full Git features and no own Git wrapper. Users still need no Git knowledge, because waymark runs every command.

## No offline mode and no retries — 2026-11-06

**Decision:** A failing write or sync stops with a clear error.

**Reason:** Agent tools need the internet, so without it nothing works anyway. It keeps the first release simple.

## Always fetch before a write, check the base version — 2026-11-06

**Decision:** Every write fetches first with no age check, and changes check `--base <hash>`.

**Reason:** No silent overwrites and a short time between pull and push.

## One lock for all commands — 2026-11-06

**Decision:** A lock file in the repository keeps background runs and agent writes apart.

**Reason:** A background run never collides with an agent write.

## The running app syncs in the background — 2026-11-06

**Decision:** The menu bar app does the background sync. An earlier design used a separate LaunchAgent.

**Reason:** One visible process instead of a hidden job. If the app is not running, reads still pull first.

## Keep both versions on a real conflict — 2026-11-06

**Decision:** Abort the rebase, keep both versions as two files, and report it.

**Reason:** Managers must never resolve a merge conflict by hand, and no content may be lost.

## No app, pull in the session — 2026-11-11

**Decision:** There is no app and no background job in local mode. The server pulls at start, before a read when the clone is a few minutes old, and in every write flow. Server mode adds a background pull.

**Supersedes:** "The running app syncs in the background".

**Alternatives considered:**
- A background job per computer. It needs an installer and a process that no session owns.

**Reason:** The server lives for one session, and that is when the clone must be current.

## Git handles the login — 2026-11-11

**Decision:** `git` uses the credentials that the person set up. waymark never handles a secret. waymark sets only the commit author.

**Reason:** The agent and the MCP entry must never hold a secret.
