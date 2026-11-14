# Todo: Integration testing

## Bugs

None.

## Deferred

- Tests for the HTTPS transport, when server mode is in scope.
- A fake credential helper, to test Git login failures.

## Open Questions

None.

## Notes from the implementation (2026-11-14)

- Deferred: a test that the client panics with a clear message when the server does not answer in 30 seconds. It needs a second child program that never answers, and the test would wait 30 seconds. The code path exists (`McpClient::request`).
- `make test-integration` exiting 0 is shown by `make check`, which runs it. There is no separate shell test, because it would run the whole suite twice.
