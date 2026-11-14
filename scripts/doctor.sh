#!/usr/bin/env bash
# Check that this machine is ready to build and test waymark.
set -u
fail=0
need() { # name, command to get a version, fix text
  if command -v "$1" >/dev/null 2>&1; then
    printf '  ok   %-10s %s\n' "$1" "$($2 2>&1 | head -1)"
  else
    printf '  MISSING %s. Fix: %s\n' "$1" "$3"
    fail=1
  fi
}
echo "waymark doctor"
if [ "$(uname -s)" != "Darwin" ] || [ "$(uname -m)" != "arm64" ]; then
  echo "  warn this project supports macOS on Apple silicon only"
fi
need git "git --version" "xcode-select --install"
need make "make --version" "xcode-select --install"
need brew "brew --version" "see https://brew.sh"
need rustup "rustup --version" "make setup"
need cargo "cargo --version" "make setup"
need rustc "rustc --version" "make setup"
if command -v cargo >/dev/null 2>&1; then
  if ! cargo fmt --version >/dev/null 2>&1; then echo "  MISSING rustfmt. Fix: make setup"; fail=1; fi
  if ! cargo clippy --version >/dev/null 2>&1; then echo "  MISSING clippy. Fix: make setup"; fail=1; fi
fi
if [ "$fail" -ne 0 ]; then echo "not ready"; exit 1; fi
echo "ready"
