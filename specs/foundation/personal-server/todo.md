# Todo: Personal server

_Known bugs, deferred improvements, and open questions. Updated throughout the lifecycle._

## Bugs

_None. Nothing is implemented._

## Deferred

- [ ] Small storage (a folder or an S3-compatible bucket such as Cloudflare R2). It would enable a "Connect GitHub" button, revoking one client, and changing the repository with no edit of the host settings. Blocked by: a decision that waymark may store an encrypted secret.
- [ ] A one-time setup page, protected by a code that the server prints in its log on the first start. It would test the storage and show what to paste. It is not the web console.
- [ ] Setup through the GitHub App manifest flow (one click creates the App and returns all secrets). Could be a command (see `waymark-doctor` in `PROJECT.md`) or only a part of the guide.
- [ ] Several memory repositories on one server. This changes the rule "one process serves one memory repository" and needs its own decision. Two small servers work today.
- [ ] Renewal tokens that rotate, with the state this needs.
- [ ] Other identity providers (Google, a company OIDC login) through the identity interface.
- [ ] A guide for a second hosting service.

## Open Questions

- See the open questions in [requirements.md](requirements.md) and [plan.md](plan.md).
- Before the code is final: test the sign-in of Claude, ChatGPT, and Gemini against a dummy door.
- Scope: server mode is out of scope for the current task. When does the product owner want to change that?
