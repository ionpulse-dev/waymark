# Decisions: Login and identity

> **Note 2026-11-11:** "Reuse `gh` or paste a token" is superseded by "Git handles the login" below. "The email must reach a person" stays valid.

_Records why technical choices were made. Never delete entries — add new ones as decisions are made. This file helps future AI sessions and developers understand settled choices without re-debating them._

---

## Reuse `gh` or paste a token — 2026-11-06

**Decision:** The first release reuses the `gh` token, or the person pastes a token that setup helps to create. Token in the Keychain. Device login comes later.

**Alternatives considered:**
- OAuth device flow first — needs an OAuth app per Git host.

**Reason:** Small to build and no OAuth app to register now.

## The email must reach a person — 2026-11-06

**Decision:** waymark shows the email from the Git host account and the person confirms or corrects it. The checks run in setup and before every commit.

**Alternatives considered:**
- A separate list of people in the repository — extra state.
- Trusting the account email — it is often a private no-reply address.

**Reason:** Every document has a reachable author through Git history. Contact happens outside waymark. If the author is gone, the team folder is the fallback.

## Git handles the login — 2026-11-11

**Decision:** waymark stores no token and runs no login. Git uses the SSH key or credential helper of the person. A server uses single sign-on and its own Git credential.

**Reason:** the agent and the MCP entry must never hold a secret. Alternative considered: a Keychain item and a credential helper of waymark. It needs an app and a login flow.

## Scope: only the author and the Git login — 2026-11-14

**Decision:** In the current scope, this spec covers the commit author (arguments, environment, Git config), the offline email check, and the rule that Git handles the login. Server sign-in, the MX check, and the check against the Git host account are cut and listed in `todo.md`.

**Alternatives considered:** Keep the MX check (rejected: it needs DNS in every write, fails offline, and the commit would then fail on a plane). Keep the Git host check (rejected: it needs a token, which waymark never holds).

**Reason:** The scope is local mode. Decided autonomously.

## The author is resolved at every write — 2026-11-14

**Decision:** `identity::Author::resolve` runs at every write call and in `status`. The order is argument, environment variable, Git config. Each part (name, email) is resolved on its own.

**Alternatives considered:** Resolve once at start (rejected: a person who fixes the Git config would have to restart the agent tool).

**Reason:** Matches the rule of `trust-model` that settings are read at every call. Decided autonomously.

## The default blocklist does not block the example domains — 2026-11-14

**Decision:** The default blocklist has the groups no-reply, Git host placeholders, non-personal, and fake. The rows `example.com`, `example.org`, `example.net`, and the reserved endings are removed from it. A knowledge base can add them in `people.email_blocklist`.

**Supersedes:** the blocklist table of the first version of the requirements (the rows "Fake or test" and "Reserved domains").

**Alternatives considered:** Keep all rows (rejected: the specs, the docs, and the test harness use `jane@example.org` as the example address, so every test and every first try from the README would fail). Add a hidden switch for tests (rejected: a switch that disables the check is a loophole).

**Reason:** The check guards against addresses that cannot receive mail by accident (a Git host placeholder, a no-reply address). A fake domain is a rare mistake. Decided autonomously. Conflict to note: the standalone team skill (`waymark-trail`) blocks the example domains. The two checks differ on purpose. See the report.

## The email check is in `identity`, a module outside `gitsync` — 2026-11-14

**Decision:** The code is in `src/identity/` (`author.rs`, `email.rs`, `redact.rs`). `gitsync` takes a checked `Author` as input. `gitsync` does not resolve or check the address itself.

**Supersedes:** step 2 of the technical approach of `git-sync` ("Author. From `--author-name` ..."), which resolved the author inside `gitsync`.

**Alternatives considered:** Keep it in `gitsync` (rejected: `status` and the tools need the same code, and the Git module should not know the blocklist).

**Reason:** One place for the rules. `gitsync` stays a wrapper of the `git` binary. Decided autonomously.

## A bad address is `setup_problem` — 2026-11-14

**Decision:** A failed email check or a missing author ends a write with `setup_problem`, not `validation_failed`.

**Alternatives considered:** `validation_failed` (rejected: that code is for the content of a document).

**Reason:** The person fixes their setup, not the document. Decided autonomously.

## URLs are shown without a password — 2026-11-14

**Decision:** `identity::redact_url` removes the password part of the user info of any URL before it enters a result, an error, or a log.

**Reason:** A person may put a token in a URL by mistake. The result then does not repeat it. Decided autonomously.

## Implementation notes — 2026-11-14

- The clap `env` attribute is removed from `--author-name` and `--author-email`. `identity::resolve` reads the variables itself, so it can tell the source (`argument` or `environment`). Decided autonomously.
- `Repo` holds the author in a `Mutex`. `Context::checked_author` resolves and checks it at every write (through `writes::author_string`) and sets it on the repo. At the first open, a missing author does not stop reads: the repo gets an empty author, and a write resolves again. Decided autonomously.
- `gitsync::Author::resolve` is removed. `identity::resolve` replaces it.
- The login failure mapping is in `mcp::error::error_json`, not in `gitsync::cmd`. One place maps every `sync_failed` whose message holds a login marker (`identity::is_login_failure`). The same function redacts passwords in all messages (`redact_text`). Decided autonomously.
- `status` has the checks `author` and `git_login` and `data.author` now. The `git_login` check runs `git ls-remote origin HEAD` in a thread with a 10 second time-out. `maintenance` extends the other checks.
- Test A9 collects every result and error. It does not collect the stderr log, because the harness lets stderr pass through. The server logs no argument values (checked by reading the code: nothing logs `Args`).
