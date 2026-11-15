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
