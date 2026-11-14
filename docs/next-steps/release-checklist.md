# Release checklist

Releases 0.1.0 to 0.3.0 are published (2026-11-14). 0.3.0 makes the place of the memory repository explicit (`--repo`, `--path`) and removes the `index` tool. 0.2.0 renames "knowledge base" to "memory repository". 0.1.5 explains every setting in the settings file and keeps a header comment on top when it adds the schema key. 0.1.4 adds the update path: an unknown or wrong argument in the MCP entry no longer exits, and every problem that only an update fixes shows in `initialize`, in the description of `status`, and in the error of every tool. 0.1.3 adds the settings schema and the automatic repairs of the settings file (see the spec `memory-repository`, section "Versions and updates"). 0.1.1 added `init` for a folder with the older `.waymark` marker, and `--help` examples. 0.1.2 added the link rules (a write never creates a broken body link, `delete` refuses linked documents, `sync` applies the same rule) and the folder tools `folders`, `add_empty_folder`, and `remove_folder`. `brew upgrade` to 0.1.2 works. The steps below show what is done and what is left.

## Done

- [x] Repository `ionpulse-dev/waymark` is public.
- [x] Tap `ionpulse-dev/homebrew-tap` exists, with `Formula/waymark.rb` and a README.
- [x] Tag `v0.1.0` is pushed. The GitHub release holds the archive and its `.sha256`, built by hand with `scripts/package.sh`.
- [x] The formula has the real version and checksum.
- [x] `brew audit --strict --new ionpulse-dev/tap/waymark` exits with 0.
- [x] Release 0.2.0 is published (2026-11-14), and `brew install ionpulse-dev/tap/waymark` installs it.
- [x] Real install (H8): `brew install ionpulse-dev/tap/waymark` works, and `waymark --version` prints `0.1.0`.

## Left

- [ ] Run the manual tests with real agent tools (`make manual-tests`). Add the MCP entry from `docs/install.md` first. One of them checks which channel of "WAYMARK NEEDS AN UPDATE" each agent tool shows.
- [ ] Choose a license for the tap repository. It has no LICENSE file.
- [ ] Check the trademark and the domains of the name (roadmap, phase 5).
- [ ] Decide how to handle outside contributions.
- [ ] Check the sample vault in Obsidian (`standard/conformance/vault`).
- [ ] Rename `.github/workflows/release.yml.disabled` to `release.yml` once you want releases built by CI. The workflow is untested.

## For the next release

1. Write the entry for the new version at the top of `CHANGELOG.md`: a short description for people, what is new and what to do. It becomes the text of the GitHub release.
2. Commit and push it, then check:

   ```bash
   bash scripts/release.sh X.Y.Z --check
   ```

3. Release:

   ```bash
   bash scripts/release.sh X.Y.Z
   ```

   The script runs the checks, sets the version, builds, tags, publishes the release with the changelog text, updates the tap, and verifies the result. It stops at the first step that fails and names it. The rules are in `specs/foundation/distribution`, section "Release process".
4. On your machine: `brew update && brew upgrade ionpulse-dev/tap/waymark`, then restart your agent tool.

## Not covered

- Signing and notarization need a developer identity. Without it the formula installs an unsigned binary. Homebrew installs are not quarantined, so no Gatekeeper warning shows.
- Linux and the container image are out of scope for local mode.
