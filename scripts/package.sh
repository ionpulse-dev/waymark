#!/usr/bin/env bash
# Build the release archive and its checksum into dist/. Used by `make brew-local` and the release workflow.
set -euo pipefail
cd "$(dirname "$0")/.."
TARGET=aarch64-apple-darwin
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
cargo build --release --target "$TARGET"
mkdir -p dist
NAME="waymark-$VERSION-$TARGET"
tar -C "target/$TARGET/release" -czf "dist/$NAME.tar.gz" waymark
(cd dist && shasum -a 256 "$NAME.tar.gz" > "$NAME.tar.gz.sha256")
echo "dist/$NAME.tar.gz"
