#!/usr/bin/env bash
# Install waymark from a local Homebrew tap and a local release file. Publishes nothing.
# Use `scripts/brew-local.sh remove` to undo it.
set -euo pipefail
cd "$(dirname "$0")/.."
TAP=local/waymark
if [ "${1:-}" = "remove" ]; then
  brew uninstall waymark 2>/dev/null || true
  brew untap "$TAP" 2>/dev/null || true
  exit 0
fi
ARCHIVE=$(bash scripts/package.sh | tail -1)
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
SUM=$(shasum -a 256 "$ARCHIVE" | cut -d' ' -f1)
brew tap-new --no-git "$TAP" >/dev/null 2>&1 || true
DIR="$(brew --repository)/Library/Taps/local/homebrew-waymark/Formula"
mkdir -p "$DIR"
sed -e "s|^  url .*|  url \"file://$PWD/$ARCHIVE\"|" \
    -e "s|VERSION|$VERSION|g" -e "s|SHA256|$SUM|" \
    packaging/homebrew/waymark.rb > "$DIR/waymark.rb"
brew style "$DIR/waymark.rb"
HOMEBREW_NO_INSTALL_FROM_API=1 brew install "$TAP/waymark"
command -v waymark
waymark --version
