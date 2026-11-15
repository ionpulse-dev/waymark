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
