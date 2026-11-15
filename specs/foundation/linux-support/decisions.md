# Decisions: Linux support

_Records why technical choices were made. Never delete entries — add new ones as decisions are made._

---

## Linux support is its own spec, before `personal-server` — 2026-11-15

**Decision:** Linux support is a separate foundation spec, and `personal-server` requires it. The project decision of 2026-11-14 "Linux is out of scope" changes for Linux on x86_64. The other parts of that decision stay: server mode, multi-tenancy, sign-in, and the web console are still out of scope until the product owner changes it.

**Alternatives considered:**
- Add Linux work inside `personal-server` — rejected: Linux also helps local mode users, and it changes the README, the scripts, and the release. A mixed spec would hide that.
- Build a Linux binary only for the container — rejected: servers run Linux, and a macOS-only code base cannot be tested there. The code and the tests must work on Linux first.

**Reason:** A server host runs Linux, so every later server spec depends on a Linux build that is tested. Doing it first, on its own, keeps each spec small.

## Measure before fixing — 2026-11-15

**Decision:** The first task is a full test run on a clean Linux machine. The spec lists only what was seen. On 2026-11-15 in a sandbox: the build worked, 214 unit tests passed, and the integration tests could not be judged because `/dev/null` in that sandbox was a regular file.

**Alternatives considered:** Guess the problems from the code — rejected: the build already works, so guesses would add work that is not needed.

**Reason:** The facts are small and cheap to get, and they set the real size of the work.

## System texts decide at run time — 2026-11-15

**Decision:** Texts about updating and installing Git come from one module, `src/platform.rs`. It picks the text from the system and from the install path of the binary. The update hint says `brew upgrade waymark` only when waymark runs from a Homebrew folder.

**Alternatives considered:**
- `cfg(target_os)` at compile time — rejected: a Linux binary can be installed with Homebrew, and a macOS binary can be copied by hand, so the system alone does not tell how to update.
- Keep the `brew` text everywhere — rejected: it is wrong advice for most Linux users.

**Reason:** The advice must fit how the person installed waymark.

## Enabling CI stays a decision of the product owner — 2026-11-15

**Decision:** The workflow files stay `.disabled`. The spec makes them ready, with a Linux and macOS matrix.

**Reason:** The release checklist (`docs/next-steps/release-checklist.md`) names the CI release workflow as an open item for the product owner.

## Development also happens in a Linux container — 2026-11-15

**Decision:** The spec delivers a development container and `make linux-*` commands that run the normal work inside it from a Mac. The container image is for development only, and CI uses the same definition. A developer may still work on the host. The container is an option that always works.

**Alternatives considered:**
- Test on Linux only in CI — rejected: the maintainer works on macOS only, and a failing CI after every push is a slow loop.
- A Linux virtual machine — rejected: heavier to set up and share than a container, and it does not give CI and the developer one definition.
- Force all development into the container — rejected: macOS stays the first-class system, and the container needs a runtime that not everyone has.

**Reason:** The maintainer's computer is a Mac. The container is the Linux machine that is always at hand, and the same definition in CI keeps the results equal.

## Test Linux on arm64 and x86_64, release x86_64 first — 2026-11-15

**Decision:** Build and tests are in scope for Linux arm64 and x86_64. The first release archive is x86_64 only.

**Alternatives considered:** x86_64 only — rejected: a container on an Apple silicon Mac is arm64, so arm64 is tested anyway, and the code is the same. An arm64 archive from the start — deferred: it needs a second release build.

**Reason:** The dev container makes arm64 the natural test target on the Mac, and servers and CI usually run x86_64. Testing both costs little and avoids a surprise.

## The Linux docs are checked once by hand on a small OrbStack machine — 2026-11-15

**Decision:** The Linux setup steps in the README and `docs/install.md` are written first from the commands, with a visible "not yet tested on a real machine" note. The maintainer then follows them once on a clean Ubuntu machine in OrbStack, fixes what is wrong, deletes the machine, and removes the note. No script and no tooling for this check.

**Alternatives considered:**
- Leave the Linux docs untested — rejected: a small machine is cheap to make in OrbStack, and a wrong install text is the first thing a Linux user would meet.
- A script that follows the steps in a clean container — rejected for now: it is more to build and keep than one manual check. It stays in `todo.md` in case the check repeats.
- A permanent Linux machine — rejected: a throwaway machine is clean every time and needs no upkeep.

**Reason:** One short manual check gives real evidence at almost no cost, so it fits the "low key" aim.

## OrbStack is the tested container runtime — 2026-11-15

**Decision:** The commands are tested with OrbStack, the runtime of the maintainer. They use only the Docker-compatible command line, so other runtimes can work and are best effort.

**Alternatives considered:**
- Name one runtime as required — rejected: it would force a product and its license on every contributor.
- Test with several runtimes — rejected for now: the maintainer uses one, and the Docker-compatible command line is the common part.

**Reason:** Test what the maintainer really runs, and keep the interface to the standard command line.

**Note:** The speed of x86_64 containers on Apple silicon through OrbStack is not verified here.

## The container image for development is not the image of `personal-server` — 2026-11-15

**Decision:** They are two files with two jobs. They use the same Linux base.

**Reason:** One holds the toolchain and the source mount, and the other holds only the binary, `git`, and certificates and runs a server. Mixing them would put build tools into a server image.

## Findings of the first Linux run — 2026-11-15

**Decision:** The measure step ran the suite in the development container (Ubuntu 22.04, arm64 native, and x86_64 through OrbStack emulation). The `/dev/null` problem was only the sandbox: on a clean system the integration tests run. The real problems were:
1. **`kill -KILL -<pid>` in `src/gitsync/cmd.rs`.** The `kill` of procps (Linux) reads `-<pid>` as an option. It killed every process of the user, including the test run and the server. The fix is `kill -KILL -- -<pid>`. This is a bug that affects real Linux users, not only the tests. It is in the changelog.
2. **System config path.** The code read only `/Library/Application Support/waymark/config.toml`. The trust-model spec says `/etc/waymark/config.toml` on Linux. `src/config/layers.rs` now picks the path at run time.
3. **Git worktrees in the container.** `.git` of a worktree is a file with an absolute host path. `scripts/linux.sh` mounts the Git folder at the same path. `scripts/public-check.sh` now fails when Git does not work, because it passed silently before.
4. **`\b` in `public-check.sh`.** macOS and Linux regular expressions differ. The pattern for an internal host flagged `~/.local/bin` on Linux. It now needs a host name before the dot.
5. **Shell scripts.** `sed -i ''`, `shasum`, `otool`, and Homebrew-only steps do not work on Linux. They are portable now, or skipped with a message when a tool does not exist.

**Reason:** The spec lists what was seen, not what was guessed.

## The development container is Ubuntu 22.04 — 2026-11-15

**Decision:** `.devcontainer/Dockerfile` uses `ubuntu:22.04`. The release archive for Linux is built in this container, so the binary needs glibc 2.35 or older.

**Alternatives considered:**
- Ubuntu 24.04 or Debian 13 — rejected: a newer glibc makes the binary fail on Ubuntu 22.04 and Debian 12.
- A static musl build — deferred: it needs a second target and a check that `git` calls still work. See `todo.md`.

**Reason:** The older the build system, the more systems run the binary. This answers the open question about the C library for now: glibc, built on an old base. The shell tests check the needed glibc version.

## Named volumes per architecture and checkout — 2026-11-15

**Decision:** `scripts/linux.sh` names the volumes `waymark-target-<arch>-<hash of the path>` and `waymark-cargo-<arch>-<hash>`. `CARGO_TARGET_DIR` is `/cache/target`, outside the mounted folder.

**Alternatives considered:** One shared volume — rejected: builds of two architectures or two worktrees would overwrite each other. A `target/` volume inside the source mount — rejected: Docker would create an empty `target` folder in the project, maybe owned by root.

**Reason:** Speed, and no file in the project folder owned by `root`. The folders in the image are world-writable, so a volume that starts from them is writable by the host user ID.

## The release workflow and `release.sh` both create the release — 2026-11-15

**Decision:** Both check if the release exists, create it if not, and upload with `--clobber`. The order does not matter. `scripts/release.sh` stays the flow of the maintainer on a Mac (macOS archive and tap). The workflow adds the Linux x86_64 archive. The Homebrew formula stays macOS only.

**Alternatives considered:** Let the workflow do everything — rejected: the tap update and the checks of `release.sh` are the maintainer flow of the distribution spec. A Linux block in the formula — deferred (see `todo.md`).

**Reason:** Both run after a tag. One must not fail because the other was first.

## Tests use the hint of the running system — 2026-11-15

**Decision:** Tests that checked `brew upgrade waymark` or `xcode-select` now check `waymark::platform::update_step()`, `update_sentence()`, and `git_install_hint()`. The unit tests of `src/platform.rs` cover every branch of both systems with the `_for` functions.

**Reason:** The test binary and the binary under test run from `target/`, which is not a Homebrew folder, so the generic update text applies. A test must not depend on how the machine was set up.

## The doctor can fake the system for its tests — 2026-11-15

**Decision:** `DOCTOR_OS` and `DOCTOR_ARCH` replace `uname` in `scripts/doctor.sh`. They exist only for the shell tests.

**Reason:** The Linux hints and the platform warning must be tested on macOS too, with one test suite.
