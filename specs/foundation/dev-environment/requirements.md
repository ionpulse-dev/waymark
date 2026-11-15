---
type: foundation
status: implemented
created: 2026-11-14
updated: 2026-11-14
---

# Development environment

> **Note 2026-11-15:** The spec [linux-support](../linux-support/requirements.md) (`implemented`) makes the commands and scripts work on Linux (x86_64 and arm64) too, and adds a development container with `make linux-*` commands. The out-of-scope line "Linux and Intel Macs" below now means Intel Macs only.

## Technical Goal

One command sets up a new Mac for work on waymark, and a small set of commands covers all daily work: build, test, check, run, and prepare a release. A new person, or a new AI session, needs no other knowledge to build and test the project.

## Dependents

- Every spec with implementation tasks uses `make test` and `make check` after each task (see `specs/WORKFLOW.md`).
- `integration-testing` — adds `make test-integration`.
- `distribution` — adds `make release` and `make brew-local`.

## Deliverables

- `Makefile` with the targets below.
- `rust-toolchain.toml` that pins the Rust version and the components.
- `scripts/doctor.sh`, `scripts/setup.sh`, and `scripts/spec-check.sh`.
- `.github/workflows/ci.yml.disabled`: build, test, and lint on macOS.
- A development guide, `docs/development.md`, linked from `README.md`, that names `make setup` and `make help`.

## Scope

**In scope:** macOS on Apple silicon. Setup, build, test, lint, format, run, spec checks, and release preparation.

**Out of scope:** Intel Macs (Linux is covered by `linux-support`). Publishing a release (see `distribution`). Installing agent tools.

## Constraints

- The Makefile works with GNU Make 3.81, the version that ships with macOS. It uses no feature of a newer Make.
- `make setup` needs no `sudo`. It is safe to run again. It changes nothing that is already right.
- `make setup` installs only from official sources: `rustup` and its toolchain servers, `crates.io`, and Homebrew.
- Every target prints what it does. `make help` lists all targets with one line each. `make` with no target runs `help`.
- The Makefile holds no secret and no personal path.
- Tool versions come from one place: `rust-toolchain.toml` for Rust, and variables at the top of the Makefile for other tools.

### Targets

| Target | What it does |
|:--|:--|
| `help` | List all targets. |
| `setup` | Install what is missing: Xcode command line tools check, Homebrew check, `rustup`, the pinned Rust toolchain with `rustfmt` and `clippy`, and the extra tools in `scripts/setup.sh`. Then run `doctor`. |
| `doctor` | Check that the machine is ready. Print each tool and its version. Fail with a clear message if one is missing. |
| `build` | Debug build. |
| `release` | Release build for `aarch64-apple-darwin`. |
| `test` | Unit and conformance tests. |
| `test-integration` | Integration tests (see `integration-testing`). |
| `test-all` | `test` and `test-integration`. |
| `fmt` | Format the code. |
| `lint` | `cargo fmt --check` and `cargo clippy` with warnings as errors. |
| `check` | `lint`, `test-all`, and `spec-check`. Run before each commit. |
| `spec-check` | Check the spec folders (see below). |
| `run` | Start the server on a sample memory repository. |
| `sample-memory` | Create a temporary sample memory repository in `tmp/`. |
| `brew-local` | Install the binary from a local tap (see `distribution`). |
| `clean` | Remove build output and `tmp/` test data. |

### `spec-check`

- Every folder under `specs/features/` and `specs/foundation/` that is not superseded has the six files.
- The `status` in each `requirements.md` equals the status in `specs/INDEX.md`.
- Relative links in `specs/` and `docs/` point to existing files.
- Every `requires` entry names an existing spec folder.

## Acceptance Criteria

- **Given** a Mac with Xcode command line tools and Homebrew but no Rust, **When** `make setup` runs, **Then** it installs the pinned toolchain and `make doctor` passes.
- **Given** a ready machine, **When** `make setup` runs again, **Then** it changes nothing and succeeds.
- **Given** a clean checkout, **When** `make build` and `make test` run, **Then** both succeed.
- **Given** the Makefile, **When** `make -n` runs with GNU Make 3.81, **Then** it parses without error.
- **Given** a spec folder with a missing file or a status that differs from `INDEX.md`, **When** `make spec-check` runs, **Then** it fails and names the problem.
- **Given** `make help`, **When** it runs, **Then** it lists every target of the table above.
- **Given** a missing tool, **When** `make doctor` runs, **Then** it names the tool and the command that fixes it.

## Open Questions

None.
