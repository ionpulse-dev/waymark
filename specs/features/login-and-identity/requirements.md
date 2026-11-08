---
type: feature
status: draft
created: 2026-11-08
updated: 2026-11-08
requires:
  - cli-core
---

# Login and identity

## User Story

> As a person, I want waymark to log in to my Git host once and use my own name and a reachable email, so that every document I publish shows who is responsible.

## Personas

- **Every person** — logs in once and confirms the email address.
- **Knowledge base owner** — can limit allowed email domains.

## Behaviour

### Login

1. If `gh` is installed and logged in to the Git host, setup offers to use its token.
2. Otherwise setup opens the token page of the Git host in the browser, with the needed settings filled in. The person generates the token and pastes it once.

The token is stored in the macOS Keychain, never in a file. `git` gets it through `waymark credential`, which speaks the Git credential helper protocol. After login, waymark reads the name and email of the person from the Git host for the commit author. If the token expires, the next command fails with a clear message and `waymark login` asks for a new one.

### Email check

waymark accepts only an address that reaches a person. It checks in `setup` and again before every commit. If a check fails, the command stops and asks for a valid address. Checks, in this order:

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
5. **Verified on the Git host, when possible.** If the token can read the account's emails, the address must be a verified email of that account. If not, skip this check.

With managed accounts the account email comes from the company directory, so all checks pass unnoticed. With personal accounts the email can be private, so waymark takes it from the account and the person confirms it.

The credential is requested from the app over the app link when the app runs. Without the app (first phase), the CLI reads the Keychain itself.

## Interfaces & Contracts

```text
waymark login [--host <host>]
waymark credential get|store|erase      # Git credential helper, not for people or agents
```

## Acceptance Criteria

- **Given** `gh` is logged in, **When** setup runs, **Then** it offers to reuse the token and stores nothing in a file.
- **Given** an expired token, **When** a command needs the Git host, **Then** it exits with code 3 and tells the person to run `waymark login`.
- **Given** the address `jane@users.noreply.github.com`, **When** the check runs, **Then** it fails on the blocklist.
- **Given** `people.email_domains = ["corp.example.org"]`, **When** the address uses another domain, **Then** the check fails.
- **Given** a domain without an MX record, **When** the check runs, **Then** it fails.
- **Given** a token that cannot read emails, **When** the check runs, **Then** step 5 is skipped.
- **Given** a commit by waymark, **When** it is created, **Then** the author email passed the check.

## Constraints & Edge Cases

- The token never appears in output, logs, or the repository.
- Device login (OAuth device flow) comes after the first release. It needs an OAuth app per Git host.

## Open Questions

- Which Git hosts besides GitHub and GitHub Enterprise are supported at first?
