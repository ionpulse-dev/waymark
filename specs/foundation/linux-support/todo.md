# Todo: Linux support

_Known bugs, deferred improvements, and open questions. Updated throughout the lifecycle._

## Bugs

- [x] The integration tests failed in the sandbox of 2026-11-15 with `bad config line 1 in file /dev/null`. Confirmed on 2026-11-15: it was the sandbox. In a clean container all 633 tests pass (see `decisions.md`).
- [x] `kill -KILL -<pid>` killed every process on Linux (fixed with `--`).
- [x] The system config path was only the macOS path (fixed).
- [x] Git did not work in the container for a Git worktree (fixed in `scripts/linux.sh`).

## Deferred

- [ ] A release archive for Linux arm64. Build and tests on arm64 are in scope (the dev container on Apple silicon runs arm64). Many hosts offer arm64 and it is cheaper. The archive needs a second release build.
- [ ] A script that follows the Linux install steps in a clean Debian or Ubuntu container and checks `waymark --version`. Only if the manual check on an OrbStack machine has to be repeated often.
- [ ] A Dev Containers configuration for one-click attach in an editor, if the maintainer wants it.
- [x] Running the suite as x86_64 on the Mac through OrbStack (`ARCH=amd64`). Checked 2026-11-15: the whole suite (633 tests) passes. A full run with the first build took about 6 minutes.
- [ ] Packages for `apt` or other package managers.
- [ ] A static (musl) build, if the glibc build is too limiting.
- [ ] A Linux block in the Homebrew formula and a second archive, if the tap should serve Linux. Today the formula is macOS only.
- [ ] A Linux arm64 job in CI (`ubuntu-24.04-arm`), so that arm64 is tested in CI and not only in the container on a Mac.
- [ ] The `Unreleased` section of `CHANGELOG.md` becomes the entry of the next version when the release skill runs.
- [ ] Windows and Intel Macs.

## Open Questions

- See [requirements.md](requirements.md). The open questions that the work answered are marked there.
