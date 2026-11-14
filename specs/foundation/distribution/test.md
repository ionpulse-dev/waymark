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
- [x] [auto] `scripts/changelog-entry.sh` prints the entry of a version and fails for a missing entry (`scripts/tests/run.sh`)
- [x] [auto] `scripts/release.sh` refuses a missing version, a bad version, and a version without a changelog entry before it changes anything (`scripts/tests/run.sh`)
- [x] [auto] Every version in `Cargo.toml` and every git tag `v*` has an entry in `CHANGELOG.md` (`scripts/tests/run.sh`)
- [ ] [manual] The next real release runs `scripts/release.sh`. Expected: all steps pass, the GitHub release shows the entry of the changelog as its text, and `brew upgrade` installs the version.
- [ ] [manual] After the tap is public: `brew install ionpulse-dev/tap/waymark` on a clean Mac, then `waymark --version`. Expected: the version of the tag.
- [ ] [manual] Add the README snippet to Claude Code, GitHub Copilot, and OpenAI Codex. Start a session. Expected: the waymark tools show.

## Notes

The local tests use a local tap and a local tarball. They publish nothing.
