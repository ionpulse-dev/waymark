# Todo: Write approval

## Bugs

None.

## Deferred

- The `console` channel, which the agent cannot answer (server mode, see `web-console`). Out of scope for now.
- An approval channel for server mode with its own sign-in. Out of scope for now.
- A diff for changes to existing documents.
- Ask before the lock is taken, or release the lock during the wait and run the base check again after the answer, so a second process does not fail.
- An asynchronous "pending" result, if a client cuts a long wait (decided by manual test M6).
- A limit for the time-out in `trust-model`.
- A pull request gate where a second person approves (the idea `pull-request-gate`).

## Open Questions

None.

## Added 2026-11-14

- Add the `approval` field to the result of `sync`, also for its preview.
- Test the lock wait of a second process during an approval. It needs a setting for a short lock wait.
