---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-11
requires:
  - mcp-server
---

# Login and identity

## User Story

> As a person, I want waymark to use my own Git login and my own name and a reachable email, so that every document I publish shows who is responsible, without waymark handling any secret.

## Personas

- **Every person** — signs in with Git once and confirms the email address.
- **Knowledge base owner** — can limit allowed email domains.
- **Operator** — configures sign-in and Git credentials of a server.

## Behaviour

### Git login (local mode)

waymark does not log in and stores no token. `git` uses what the person set up: an SSH key, or a credential helper that was filled by `gh auth login` or by an earlier `git push`. The server only checks that the remote answers (`status`). If not, the result says in plain words what to do, for example "Git cannot sign in to this repository. Sign in with Git first, for example with `gh auth login`, or set up an SSH key." The error is `sync_failed`. IT can prepare the login for a whole team.

No tool receives or returns a secret, and no secret is an argument of the MCP entry.

### Author

The commit author is the name and email of the person. Order: the arguments `--author-name` and `--author-email` (or the environment variables), then the Git settings of the person. If neither gives a usable address, writes fail with `setup_problem` and say how to set one.

### Email check

waymark accepts only an address that reaches a person. It checks at start and again before every commit. If a check fails, the write stops and says what to fix. Checks, in this order:

1. **Syntax.** One `@`, a local part, a domain with a dot.
2. **Blocklist.** The address must not contain these parts (case does not matter):

   | Kind | Blocked parts |
   |---|---|
   | No-reply | `noreply`, `no-reply`, `no_reply`, `donotreply`, `do-not-reply`, `do_not_reply`, `dontreply`, `dont-reply`, `noanswer`, `no-answer` |
   | Git host placeholders | `users.noreply.github.com`, `noreply.gitlab.com`, `localhost`, `localdomain` |
   | Non-personal | `nobody@`, `null@`, `devnull`, `mailer-daemon`, `postmaster@`, `bounce`, `bounces`, `notifications@`, `notification@`, `automated`, `bot@`, `[bot]` |
   | Fake or test | `example.com`, `example.org`, `example.net`, `test@`, `placeholder`, `changeme`, `unknown@`, `none@` |
   | Reserved domains | ending in `.invalid`, `.local`, `.localhost`, `.test`, `.example` |

   A knowledge base adds its own parts in `people.email_blocklist`.
3. **Allowed domains (optional).** If `people.email_domains` is set, the address must use one of them. This is the strongest check.
4. **Domain receives mail.** The domain must have an MX record.
5. **Verified on the Git host, when possible.** On a server with a Git host token that can read account emails, the address must be a verified email of the account. If not possible, skip this check.

### Sign-in (server mode)

On a server, people sign in with single sign-on (see `server-mode`). Name and email come from the token and go through the same email check. The server pushes with its own Git credential (a GitHub App preferred). The commit author is still the signed-in person.

## Interfaces & Contracts

No tool. The inputs are the arguments and variables of the MCP entry (see `mcp-server`) and, on a server, the token.

## Acceptance Criteria

- **Given** a Git login that works, **When** `status` runs, **Then** the Git login check is ok and no secret was read.
- **Given** an expired Git login, **When** a sync runs, **Then** it fails with `sync_failed` and a plain `next` sentence.
- **Given** the address `jane@users.noreply.github.com`, **When** the check runs, **Then** it fails on the blocklist.
- **Given** `people.email_domains = ["corp.example.org"]`, **When** the address uses another domain, **Then** the check fails.
- **Given** a domain without an MX record, **When** the check runs, **Then** it fails.
- **Given** a commit by waymark, **When** it is created, **Then** the author email passed the check.
- **Given** any result or log, **When** it is inspected, **Then** it holds no token or password.

## Constraints & Edge Cases

- waymark never stores, prints, or passes a secret.
- Device login (OAuth device flow) comes after the first release.

## Open Questions

- Which Git hosts besides GitHub and GitHub Enterprise are supported at first?
- Which OIDC providers must work in the first server release?
