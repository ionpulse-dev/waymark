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
