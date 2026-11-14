# Release checklist

Release 0.1.0 is published (2026-11-14). The steps below show what is done and what is left.

## Done

- [x] Repository `ionpulse-dev/waymark` is public.
- [x] Tap `ionpulse-dev/homebrew-tap` exists, with `Formula/waymark.rb` and a README.
- [x] Tag `v0.1.0` is pushed. The GitHub release holds the archive and its `.sha256`, built by hand with `scripts/package.sh`.
- [x] The formula has the real version and checksum.
- [x] `brew audit --strict --new ionpulse-dev/tap/waymark` exits with 0.
- [x] Real install (H8): `brew install ionpulse-dev/tap/waymark` works, and `waymark --version` prints `0.1.0`.

## Left

- [ ] Run the manual tests with real agent tools (`make manual-tests`). Add the MCP entry from `docs/install.md` first.
- [ ] Choose a license for the tap repository. It has no LICENSE file.
- [ ] Check the trademark and the domains of the name (roadmap, phase 5).
- [ ] Decide how to handle outside contributions.
- [ ] Check the sample vault in Obsidian (`standard/conformance/vault`).
- [ ] Rename `.github/workflows/release.yml.disabled` to `release.yml` once you want releases built by CI. The workflow is untested.

## For the next release

1. Set the version in `Cargo.toml`, run `make check`, commit, push.
2. Build and publish:

   ```bash
   bash scripts/package.sh
   git tag vX.Y.Z && git push origin vX.Y.Z
   gh release create vX.Y.Z dist/*.tar.gz dist/*.sha256 --generate-notes
   ```

3. In the tap, copy `packaging/homebrew/waymark.rb` to `Formula/waymark.rb`, replace `VERSION` and `SHA256`, commit, push.
4. Check with `brew upgrade waymark` and `waymark --version`.

## Not covered

- Signing and notarization need a developer identity. Without it the formula installs an unsigned binary. Homebrew installs are not quarantined, so no Gatekeeper warning shows.
- Linux and the container image are out of scope for local mode.
