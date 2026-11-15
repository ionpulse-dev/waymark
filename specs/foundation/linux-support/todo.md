# Todo: Linux support

_Known bugs, deferred improvements, and open questions. Updated throughout the lifecycle._

## Bugs

- [ ] The integration tests fail in the sandbox of 2026-11-15 with `bad config line 1 in file /dev/null`. The sandbox `/dev/null` was a regular file. Probably an environment problem. To confirm on a clean Linux machine.

## Deferred

- [ ] A release archive for Linux arm64. Build and tests on arm64 are in scope (the dev container on Apple silicon runs arm64). Many hosts offer arm64 and it is cheaper. The archive needs a second release build.
- [ ] A Dev Containers configuration for one-click attach in an editor, if the maintainer wants it.
- [ ] Running the suite as x86_64 on the Mac through OrbStack (`ARCH=amd64`), as a manual check. Verify the speed, and whether the whole suite passes.
- [ ] Packages for `apt` or other package managers.
- [ ] A static (musl) build, if the glibc build is too limiting.
- [ ] Windows and Intel Macs.

## Open Questions

- See [requirements.md](requirements.md).
