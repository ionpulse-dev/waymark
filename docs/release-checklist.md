# Release checklist

Everything here is prepared and not done. The product owner runs these steps. Nothing in this list ran.

## Before you start

```bash
make check
bash scripts/public-check.sh
```

Check that `git status` is clean and `main` is pushed.

## 1. Make the repository public

1. Check the trademark and the domains of the name (roadmap, phase 5).
2. Decide how to handle outside contributions.
3. Check the sample vault in Obsidian (`standard/conformance/vault`).
4. In GitHub: Settings, General, Danger Zone, "Change visibility", Public.

## 2. Create the tap repository

```bash
gh repo create ionpulse-dev/homebrew-tap --public --description "Homebrew formulas"
```

The tap holds only `Formula/waymark.rb` and a short README.

## 3. Tag and build the release

Set the version in `Cargo.toml`, commit, then:

```bash
git tag v0.1.0
git push origin v0.1.0
```

To use the workflow, rename `.github/workflows/release.yml.disabled` to `release.yml` first. Or build by hand:

```bash
bash scripts/package.sh
gh release create v0.1.0 dist/*.tar.gz dist/*.sha256 --generate-notes
```

## 4. Update the formula

Copy `packaging/homebrew/waymark.rb` to `Formula/waymark.rb` in the tap. Replace `VERSION` and `SHA256` (the value in `dist/*.sha256`). Commit and push.

## 5. Test the real install (H8)

```bash
brew install ionpulse-dev/tap/waymark
waymark --version
```

Then add the MCP entry from `docs/install.md` and run the open manual tests (`make manual-tests`).

## Not covered

- Signing and notarization need a developer identity. Without it the formula installs an unsigned binary. Homebrew installs are not quarantined, so no Gatekeeper warning shows.
- A test that `brew audit --strict --new` passes needs the public homepage URL (it returned 404 before the repository was public).
- Linux and the container image are out of scope for local mode.
