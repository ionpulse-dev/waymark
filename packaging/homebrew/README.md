# Homebrew formula

`waymark.rb` is the template for `Formula/waymark.rb` in the tap `ionpulse-dev/homebrew-tap`. It installs the prebuilt `aarch64-apple-darwin` archive from the GitHub release. `VERSION` and `SHA256` are placeholders.

- `make brew-local` builds the archive, writes the formula into a local tap (`local/waymark`), runs `brew style`, and installs it.
- `bash scripts/brew-local.sh remove` removes the local install and the local tap.
- The steps for a real release are in [docs/release-checklist.md](../../docs/release-checklist.md).
