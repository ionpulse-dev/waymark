#!/usr/bin/env bash
# Run a command in the Linux development container. Usage: scripts/linux.sh [command ...]
# With no command, it opens a shell. Used by `make linux-*` and by CI.
#   ARCH=amd64|arm64          run the container as that architecture (default: the host architecture)
#   CONTAINER_RUNTIME=docker  the Docker-compatible command (default: docker; tested with OrbStack)
# The source is mounted at /work. The build output and the Cargo caches live in named volumes, so a
# full run is fast and no file in the project folder is owned by root.
set -euo pipefail
cd "$(dirname "$0")/.."

RUNTIME=${CONTAINER_RUNTIME:-docker}
if ! command -v "$RUNTIME" >/dev/null 2>&1; then
  echo "linux: no container runtime found ('$RUNTIME'). Install OrbStack (https://orbstack.dev) or another Docker-compatible runtime, then try again." >&2
  exit 3
fi
if ! "$RUNTIME" info >/dev/null 2>&1; then
  echo "Start OrbStack (or your container runtime) and try again." >&2
  exit 3
fi

case "${ARCH:-$(uname -m)}" in
  arm64|aarch64) arch=arm64 ;;
  amd64|x86_64) arch=amd64 ;;
  *) echo "linux: unknown ARCH '${ARCH:-}'. Use amd64 or arm64." >&2; exit 2 ;;
esac

rust=$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml | head -1)
[ -n "$rust" ] || { echo "linux: no Rust version in rust-toolchain.toml" >&2; exit 2; }
image="waymark-dev:$rust-$arch"
# One set of volumes per architecture and per checkout, so builds never mix.
key=$(printf '%s' "$PWD" | cksum | cut -d' ' -f1)
target_vol="waymark-target-$arch-$key"
cargo_vol="waymark-cargo-$arch-$key"

"$RUNTIME" build --quiet --platform "linux/$arch" --build-arg "RUST_VERSION=$rust" \
  -t "$image" .devcontainer >/dev/null

# In a Git worktree, `.git` is a file that points to a folder outside the project (an absolute host path).
# Mount that folder at the same path, so that Git works inside the container.
gitmount=()
if [ -f .git ]; then
  common=$(git rev-parse --git-common-dir 2>/dev/null || true)
  if [ -n "$common" ] && [ -d "$common" ]; then
    common=$(cd "$common" && pwd -P)
    gitmount=(-v "$common:$common")
  fi
fi

tty=()
if [ -t 0 ] && [ -t 1 ]; then tty=(-it); fi
[ "$#" -gt 0 ] || set -- bash

exec "$RUNTIME" run --rm "${tty[@]+"${tty[@]}"}" --platform "linux/$arch" \
  --user "$(id -u):$(id -g)" \
  -v "$PWD:/work" \
  ${gitmount[@]+"${gitmount[@]}"} \
  -v "$target_vol:/cache/target" \
  -v "$cargo_vol:/opt/cargo/registry" \
  -e WAYMARK_EXPECT_CASE_SENSITIVE=1 \
  -e GIT_CONFIG_COUNT=1 -e GIT_CONFIG_KEY_0=safe.directory -e GIT_CONFIG_VALUE_0='*' \
  "$image" "$@"
