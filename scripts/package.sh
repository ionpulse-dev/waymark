#!/usr/bin/env bash
# Build the release archive and its checksum into dist/. Used by `make brew-local` and the release workflow.
# The target is the host target. TARGET=<triple> builds for another target that is installed.
set -euo pipefail
cd "$(dirname "$0")/.."
TARGET=${TARGET:-$(rustc -vV | sed -n 's/^host: //p')}
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
sha256() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$@"; else shasum -a 256 "$@"; fi; }
cargo build --release --target "$TARGET"
mkdir -p dist
NAME="waymark-$VERSION-$TARGET"
tar -C "${CARGO_TARGET_DIR:-target}/$TARGET/release" -czf "dist/$NAME.tar.gz" waymark
(cd dist && sha256 "$NAME.tar.gz" > "$NAME.tar.gz.sha256")
echo "dist/$NAME.tar.gz"
