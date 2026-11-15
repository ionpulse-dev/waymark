# Todo: Linux support

_Known bugs, deferred improvements, and open questions. Updated throughout the lifecycle._

## Bugs

- [ ] The integration tests fail in the sandbox of 2026-11-15 with `bad config line 1 in file /dev/null`. The sandbox `/dev/null` was a regular file. Probably an environment problem. To confirm on a clean Linux machine.

## Deferred

- [ ] Linux on arm64. Many hosts offer it, and it is cheaper. Needs a second release build.
- [ ] Packages for `apt` or other package managers.
- [ ] A static (musl) build, if the glibc build is too limiting.
- [ ] Windows and Intel Macs.

## Open Questions

- See [requirements.md](requirements.md).
