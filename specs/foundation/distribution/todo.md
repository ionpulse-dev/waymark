# Todo: Distribution

## Bugs

None.

## Deferred

- Signed and notarized prebuilt binary (needs a developer identity of the product owner).
- A prebuilt Homebrew bottle.
- Homebrew core.
- Linux, Intel, and the container image (server mode).
- A download page with checksums.

## Open steps for the product owner

Moved here from `docs/next-steps/release-checklist.md` when that file was removed (2026-11-15).

- [ ] Run the manual tests with real agent tools (`make manual-tests`). Add the MCP entry from `docs/local-install.md` first. One test checks which channel of "WAYMARK NEEDS AN UPDATE" each agent tool shows.
- [ ] Choose a license for the tap repository. It has no LICENSE file.
- [ ] Check the trademark and the domains of the name.
- [ ] Decide how to handle outside contributions.
- [ ] Check the sample vault in Obsidian (`standard/conformance/vault`).
- [ ] Rename `.github/workflows/release.yml.disabled` to `release.yml` when releases should be built by CI. The workflow is not tested.

## Open Questions

- The signing identity (product owner).
- The registry of the container image (server mode).

- Signing and notarization wait for a developer identity (product owner).
- Run `brew audit --strict --new` again after the repository is public.
- `make brew-local` runs `brew install`, and Homebrew cleans old cache files then. Note it in the install test steps.
