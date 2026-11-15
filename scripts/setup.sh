#!/usr/bin/env bash
# Check or install what is missing for waymark development. Safe to run again. No sudo.
# On macOS it can install rustup with Homebrew. On Linux it never runs a package manager: it tells you the command.
set -eu
changed=0
if [ "$(uname -s)" = "Darwin" ]; then
  if ! xcode-select -p >/dev/null 2>&1; then
    echo "Xcode command line tools are missing. Run: xcode-select --install, then run make setup again."
    exit 1
  fi
  if ! command -v rustup >/dev/null 2>&1; then
    if ! command -v brew >/dev/null 2>&1; then
      echo "Homebrew is missing. Install it from https://brew.sh, then run make setup again."
      exit 1
    fi
    echo "Installing rustup with Homebrew..."
    brew install rustup
    changed=1
  fi
else
  if ! command -v git >/dev/null 2>&1 || ! command -v cc >/dev/null 2>&1 || ! command -v make >/dev/null 2>&1; then
    echo "git, make, or a C compiler is missing. Install them with the package manager of your system,"
    echo "for example: sudo apt install git build-essential. Then run make setup again."
    exit 1
  fi
  if ! command -v rustup >/dev/null 2>&1; then
    echo "rustup is missing. Install it with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "Then open a new shell and run make setup again."
    exit 1
  fi
fi
# rust-toolchain.toml pins the version. This installs it with its components.
if ! rustup show active-toolchain >/dev/null 2>&1; then
  echo "Installing the pinned Rust toolchain..."
  rustup show >/dev/null
  changed=1
fi
if [ "$changed" -eq 0 ]; then echo "Nothing to install."; fi
bash "$(dirname "$0")/doctor.sh"
