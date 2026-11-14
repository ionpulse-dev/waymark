# Plan: Distribution

## Overview

For the first release, Homebrew builds `waymark` from source on the person's Mac. This needs no signing and no release files, only a tag. A release workflow that builds a prebuilt Apple silicon binary is prepared for later. Everything is prepared in this repository. The tap repository, the tag, and the public switch are done by the product owner.

## Components Affected

- `README.md` — install section, MCP entries, `AGENTS.md` snippets.
- `Makefile` — `release` and `brew-local`.

## New Files

- `packaging/homebrew/waymark.rb` — the formula template (builds from source with `cargo install`).
- `packaging/homebrew/README.md` — how the tap is created and updated.
- `scripts/make-formula.sh` — fills the template with the URL and the SHA-256 of a tag tarball.
- `scripts/brew-local.sh` — creates a local tap in a temporary folder, builds a tarball with `git archive`, writes the formula, and runs `brew install` from it.
- `.github/workflows/release.yml` — on a tag: build the Apple silicon binary, upload `waymark-<version>-aarch64-apple-darwin.tar.gz` and its checksum. Written, not run.
- `docs/release-checklist.md` — the exact steps for the product owner: public-readiness checks, creating the tap, tagging, updating the formula.
- `docs/install.md` — the install guide with the MCP entry for each agent tool.

## Technical Approach

1. **Formula.** `url` of the tag tarball from the public repository, `sha256`, `license "MIT"`, `depends_on "rust" => :build`, `depends_on "git"`, install step `system "cargo", "install", *std_cargo_args`, and a `test do` block that runs `waymark --version`.
2. **Local test.** `scripts/brew-local.sh` does the same with a local tarball and a local tap path, so the formula is tested before anything is public. It uninstalls after the test when asked (`--keep` leaves it).
3. **Release build.** `make release` runs `cargo build --release --target aarch64-apple-darwin` and prints the path and the size. A test checks that the binary links only to system libraries.
4. **Docs.** `docs/install.md` has the install steps and one MCP entry for each of Claude Code, GitHub Copilot, and OpenAI Codex. Items that I could not verify are marked "unverified" (see `docs/agent-tools-research.md`).
5. **Public readiness.** `scripts/public-check.sh` searches the tracked files for secret patterns, email addresses other than the project's, internal host names, and words from a deny list that the product owner keeps outside the repository (the file path is an argument).
6. **Signing.** Not in the first release. Homebrew installs a source build, so macOS does not quarantine it. The prebuilt binary needs signing and notarization later (see `todo.md`).

## Dependencies on Other Foundation Items

- `mcp-server` — the binary.
- `dev-environment` — the Makefile.

## Risks

- A source build needs a Rust toolchain on every machine and takes a few minutes. Accepted for the first release. A prebuilt bottle comes later.
- The formula depends on the repository being public. Mitigation: `docs/release-checklist.md` orders the steps.

## Open Questions

None.
