# Tests: Distribution

## Acceptance Criteria Coverage

| Criterion | Test Case |
|:--|:--|
| `brew install <account>/tap/waymark` puts `waymark` on the `PATH` | local tap test (auto), public tap test (manual) |
| A new release updates the formula with the right checksum | `make_formula_fills_checksum` |
| The README snippet shows the tools in a session | manual |
| Gatekeeper passes the release as notarized | out of scope for now (source build) |
| The container image answers on HTTPS | out of scope for now |
| Installed tool descriptions match the binary | `tools_match_binary` |

## Test Cases

- [x] [auto] `scripts/brew-local.sh` fills URL and checksum for a local tarball (`scripts/tests/`)
- [x] [auto] `make brew-local` installs the formula from a local tap, `waymark --version` works from the `PATH`, and the server answers `tools/list` over stdio (`scripts/tests/`)
- [x] [auto] `brew audit --formula` and `brew style` pass on the formula in the local tap (`scripts/tests/`)
- [x] [auto] The release binary links only to system libraries (`scripts/tests/`)
- [x] [auto] `scripts/public-check.sh` flags a planted secret and passes the repository (`scripts/tests/`)
- [ ] [manual] After the tap is public: `brew install ionpulse-dev/tap/waymark` on a clean Mac, then `waymark --version`. Expected: the version of the tag.
- [ ] [manual] Add the README snippet to Claude Code, GitHub Copilot, and OpenAI Codex. Start a session. Expected: the waymark tools show.

## Notes

The local tests use a local tap and a local tarball. They publish nothing.
