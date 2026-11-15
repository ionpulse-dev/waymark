# Todo: Login and identity

## Bugs

None.

## Deferred

- Sign-in on a server (single sign-on, tokens) and the Git credential of a server. Out of scope for now (server mode).
- The MX record check. Needs DNS in the write path.
- The check against the verified emails of the Git host account. Needs a token that waymark never holds.
- Device login (OAuth device flow), after the first release.
- Align the standalone team skill (`waymark-trail`, example domains blocked) with the default blocklist of this spec, or document the difference in `plain-git-access`.

## Open Questions

- Which Git hosts besides GitHub and GitHub Enterprise are supported for the login? (server mode, later)
- Which OIDC providers must work in the first server release? (server mode, later)

## Notes 2026-11-15

- The deferred item "the check against the verified emails of the Git host account" is planned in the draft spec [personal-server](../../foundation/personal-server/requirements.md) for HTTP mode. There the server holds the user token only during the sign-in, so the reason "needs a token that waymark never holds" does not block it. Local mode is not affected.

## Notes 2026-11-14

- A9 does not read the stderr log of the server. Add a harness option to capture stderr if the server ever logs more.
