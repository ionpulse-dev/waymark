---
type: feature
status: implemented
created: 2026-11-08
updated: 2026-11-14
requires:
  - mcp-server
---

# Login and identity

> **Scope on 2026-11-14:** the current task covers local mode on macOS (Apple silicon) only. Sign-in on a server (single sign-on, tokens, the server's Git credential), the MX check, and the check against the Git host account are out of scope for now (see [project decisions](../../../docs/decisions.md)). They are in `todo.md`.

## User Story

> As a person, I want waymark to use my own Git login and my own name and a reachable email, so that every document I publish shows who is responsible, without waymark handling any secret.

## Personas

- **Every person** — has Git signed in once and a real email in the Git settings.
- **Knowledge base owner** — can limit the allowed email domains.

## Behaviour

### Git login

waymark does not log in and stores no token. `git` uses what the person set up: an SSH key, or a credential helper that was filled by `gh auth login` or by an earlier `git push`. `git` never asks a question (`GIT_TERMINAL_PROMPT=0`), so a missing login fails fast. The `status` tool checks that the remote answers (see `maintenance`). A failed login gives `sync_failed` with a plain `next` sentence, for example: "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key."

No tool receives or returns a secret. No secret is an argument of the MCP entry. A URL that holds a password (`https://user:password@host/...`) is shown in results and logs with the password removed: `https://user@host/...`.

### Author

The commit author is the name and email of the person. waymark resolves each part separately, in this order:

1. The arguments `--author-name` and `--author-email`.
2. The environment variables `WAYMARK_AUTHOR_NAME` and `WAYMARK_AUTHOR_EMAIL`.
3. The Git settings of the person: `git config user.name` and `git config user.email` (the global config, and the config of the clone if it sets them).

The result names the source (`argument`, `environment`, `git_config`). If a part is missing everywhere, a write fails with `setup_problem` and says how to set it. Reads need no author. A change of the Git config applies at the next call, because the author is resolved at every write.

### Email check

waymark accepts only an address that reaches a person. It checks in `status` and again before every commit. A failed check stops the write with `setup_problem` and a `next` sentence that says what to fix. Checks, in this order:

1. **Syntax.** One `@`, a local part, a domain with a dot, no spaces.
2. **Blocklist.** The address must not contain these parts (case does not matter):

   | Kind | Blocked parts |
   |---|---|
   | No-reply | `noreply`, `no-reply`, `no_reply`, `donotreply`, `do-not-reply`, `do_not_reply`, `dontreply`, `dont-reply`, `noanswer`, `no-answer` |
   | Git host placeholders | `users.noreply.github.com`, `noreply.gitlab.com`, `localhost`, `localdomain` |
   | Non-personal | `nobody@`, `null@`, `devnull`, `mailer-daemon`, `postmaster@`, `bounce`, `bounces`, `notifications@`, `notification@`, `automated`, `bot@`, `[bot]` |
   | Fake | `test@`, `placeholder`, `changeme`, `unknown@`, `none@` |

   A knowledge base adds its own parts in `people.email_blocklist`. To block the reserved example domains (`example.com`, `example.org`, `example.net`, and the endings `.invalid`, `.local`, `.localhost`, `.test`, `.example`), a knowledge base lists them there. They are not blocked by default (see `decisions.md`).
3. **Allowed domains (optional).** If `people.email_domains` is set, the address must use one of them. This is the strongest check.

Not in the current scope: the MX record check and the check against the verified emails of the Git host account.

## Interfaces & Contracts

No tool. The inputs are the arguments and variables of the MCP entry (see `mcp-server`). `status` reports the result in its `checks` and in `data.author`:

```json
{
  "author": { "name": "Jane Doe", "email": "jane@example.org", "name_source": "argument", "email_source": "git_config" }
}
```

The `checks` entries `author` and `git_login` are specified in `maintenance`. An error for a bad address:

```json
{
  "code": "setup_problem",
  "message": "The commit email jane@users.noreply.github.com cannot receive mail.",
  "next": "Set a real address with --author-email, WAYMARK_AUTHOR_EMAIL, or git config user.email."
}
```

## Acceptance Criteria

- **Given** an argument, an environment variable, and a Git config that all set the name, **When** waymark resolves the author, **Then** the argument wins, then the variable, then the Git config.
- **Given** no author anywhere, **When** a write runs, **Then** it fails with `setup_problem` and a `next` sentence that says how to set it.
- **Given** a person's global Git author and different `--author-*` arguments, **When** waymark commits, **Then** the commit uses the arguments.
- **Given** a Git login that works, **When** `status` runs, **Then** the Git login check is ok and no secret was read.
- **Given** a login that fails, **When** a sync runs, **Then** it fails with `sync_failed` and a plain `next` sentence, and Git did not wait for input.
- **Given** the address `jane@users.noreply.github.com`, **When** the check runs, **Then** it fails on the blocklist.
- **Given** `people.email_domains = ["corp.example.org"]`, **When** the address uses another domain, **Then** the check fails.
- **Given** a commit by waymark, **When** it is created, **Then** the author email passed the check.
- **Given** any result or log, **When** it is inspected, **Then** it holds no token or password, also when the repository URL held a password.

## Constraints & Edge Cases

- waymark never stores, prints, or passes a secret.
- The check runs offline. It makes no network call.
- A name that is empty or only spaces is missing.
- Device login (OAuth device flow) comes after the first release.

## Open Questions

None. Answered on 2026-11-14 (see `decisions.md`). The question about Git hosts and OIDC providers is about server mode and moved to `todo.md`.
