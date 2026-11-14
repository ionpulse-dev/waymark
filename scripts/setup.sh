#!/usr/bin/env bash
# Install what is missing for waymark development. Safe to run again. No sudo.
set -eu
changed=0
if ! xcode-select -p >/dev/null 2>&1; then
  echo "Xcode command line tools are missing. Run: xcode-select --install, then run make setup again."
  exit 1
fi
if ! command -v brew >/dev/null 2>&1; then
  echo "Homebrew is missing. Install it from https://brew.sh, then run make setup again."
  exit 1
fi
if ! command -v rustup >/dev/null 2>&1; then
  echo "Installing rustup with Homebrew..."
  brew install rustup
  changed=1
fi
# rust-toolchain.toml pins the version. This installs it with its components.
if ! rustup show active-toolchain >/dev/null 2>&1; then
  echo "Installing the pinned Rust toolchain..."
  rustup show >/dev/null
  changed=1
fi
if [ "$changed" -eq 0 ]; then echo "Nothing to install."; fi
bash "$(dirname "$0")/doctor.sh"
