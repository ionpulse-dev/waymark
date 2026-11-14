# Plan: Login and identity

## Overview

A small `identity` module resolves the commit author from the arguments, the environment, and the Git config, and checks the email address offline. The login itself is left to `git`. The module also removes passwords from URLs before they are shown. No tool is added.

## Components Affected

- `git-sync` (`gitsync`) — takes a checked `Author` as an input of `Repo::write`. Step 2 of its approach moves here (see `decisions.md`). `gitsync::cmd` sets `GIT_TERMINAL_PROMPT=0` already.
- `mcp-server` (`mcp::args`, `mcp::context`) — passes the argument values to `identity`, and gets an `Author` from the `Context` at every write. `mcp::error` calls `redact_url` for messages.
- `knowledge-base-repo` (`kb::settings`) — the `[people]` section (`email_domains`, `email_blocklist`) is the input of the check.
- `maintenance` — the `status` checks `author` and `git_login` call this module.

## New Files

- `src/identity/mod.rs` — public API: `Author`, `Source`, `AuthorProblem`.
- `src/identity/author.rs` — `resolve(args, env, git_config) -> Result<Author, AuthorProblem>` and the Git config reader (`git config --get`, with the isolated environment of `gitsync::cmd`).
- `src/identity/email.rs` — `check_email(address, &PeopleSettings) -> Result<(), EmailProblem>`: syntax, default blocklist, extra blocklist, allowed domains. Each problem has a plain `next` sentence.
- `src/identity/redact.rs` — `redact_url(&str) -> String` and `redact_text(&str) -> String` (finds URLs with a password in free text, for example Git error output).
- `tests/it_login_and_identity.rs` — end-to-end tests.

## Technical Approach

1. **Resolve.** Name and email are resolved separately. For each part: the argument, then the environment variable, then `git config user.name` or `git config user.email` run in the clone (so the global and the local config both count). An empty or blank value is missing. The result keeps the source of each part.
2. **Check.** `check_email` trims and lowercases. Syntax: exactly one `@`, a non-empty local part, a domain with at least one dot and no empty label, no white space. The blocklist is a constant array of substrings. A match is an `EmailProblem::Blocked(part)`. Then the extra blocklist from `[people]`. Then, if `email_domains` is set, the domain (or a subdomain ending in `.<domain>` is not accepted: exact match only) must be in the list.
3. **When.** The `Context` calls `resolve` and `check_email` before every write tool runs the write flow, and `status` calls them for its check. A problem maps to `ToolError { code: setup_problem, message, next }`. The check is not part of `Repo::write`, so the order of the write flow of `git-sync` stays as it is. The author is needed from step 7 on, so the check runs before step 1 and fails early.
4. **Login.** No code. `gitsync::cmd` runs every `git` call with `GIT_TERMINAL_PROMPT=0`, with `GIT_ASKPASS` unset, and without a TTY. A login failure in the output of `git` (`Permission denied (publickey)`, `Authentication failed`, `could not read Username`) maps to `sync_failed` with the fixed `next` sentence of the requirements. The mapping function lives in `src/gitsync/cmd.rs` (owned by `git-sync`) and is called by the tools. This spec adds its test.
5. **Redact.** All error messages and the `status` data pass `redact_text`. The `--repo` value is redacted when it is stored in the `Context`.
6. **No secret in logs.** Logs go to stderr at level `warn` and contain no argument values except the redacted URL.

## Cross-Feature Dependencies

- `maintenance` — `status` reports `author` and `git_login`. The check names are fixed there.
- `create-knowledge-base` — the `people` settings written by `init` (`email_domains`) feed the check.
- `plain-git-access` — the plain-Git rules for the author in `AGENTS.md`. They differ from the server check (see `decisions.md`).

## Risks

- The blocklist is a substring match and can block a real address (for example a person named Bounce). Mitigation: the `next` sentence names the part, and the person can use another address. Rare.
- `git config` can return a value from an include file or a conditional include. That is correct for the person, and the source is shown as `git_config`.

## Open Questions

None.
