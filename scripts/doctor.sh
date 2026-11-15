#!/usr/bin/env bash
# Check that this machine is ready to build and test waymark (macOS Apple silicon, Linux x86_64 or arm64).
# DOCTOR_OS and DOCTOR_ARCH override uname. They exist for the tests of this script.
set -u
os=${DOCTOR_OS:-$(uname -s)}
arch=${DOCTOR_ARCH:-$(uname -m)}
fail=0

case "$os" in
  Darwin)
    git_fix="xcode-select --install"
    make_fix="xcode-select --install"
    cc_fix="xcode-select --install"
    ;;
  *)
    git_fix="install Git with the package manager of your system, for example: sudo apt install git"
    make_fix="install make with the package manager of your system, for example: sudo apt install build-essential"
    cc_fix="install a C compiler with the package manager of your system, for example: sudo apt install build-essential"
    ;;
esac
rustup_fix="make setup"
[ "$os" = "Darwin" ] || rustup_fix="curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

need() { # name, command to get a version, fix text
  if command -v "$1" >/dev/null 2>&1; then
    printf '  ok   %-10s %s\n' "$1" "$($2 2>&1 | head -1)"
  else
    printf '  MISSING %s. Fix: %s\n' "$1" "$3"
    fail=1
  fi
}

echo "waymark doctor"
case "$os/$arch" in
  Darwin/arm64|Linux/x86_64|Linux/aarch64|Linux/arm64) ;;
  *) echo "  warn $os $arch is not supported. Supported: macOS on Apple silicon, Linux on x86_64 and arm64" ;;
esac

need git "git --version" "$git_fix"
if command -v git >/dev/null 2>&1; then
  # waymark needs Git 2.28 or newer (git init --initial-branch).
  v=$(git --version 2>&1 | sed -E 's/^git version ([0-9]+\.[0-9]+).*/\1/')
  case "$v" in
    [0-9]*.[0-9]*)
      if [ "$(printf '%s\n%s\n' 2.28 "$v" | sort -t. -k1,1n -k2,2n | head -1)" != "2.28" ]; then
        echo "  MISSING git 2.28 or newer (found $v). Fix: $git_fix"
        fail=1
      fi
      ;;
  esac
fi
need make "make --version" "$make_fix"
if [ "$os" = "Darwin" ]; then
  need brew "brew --version" "see https://brew.sh"
else
  need cc "cc --version" "$cc_fix"
fi
need rustup "rustup --version" "$rustup_fix"
need cargo "cargo --version" "$rustup_fix"
need rustc "rustc --version" "$rustup_fix"
if command -v cargo >/dev/null 2>&1; then
  if ! cargo fmt --version >/dev/null 2>&1; then echo "  MISSING rustfmt. Fix: make setup"; fail=1; fi
  if ! cargo clippy --version >/dev/null 2>&1; then echo "  MISSING clippy. Fix: make setup"; fail=1; fi
fi

# The Linux container is an option. A missing runtime is a note, not a failure.
runtime=${CONTAINER_RUNTIME:-docker}
if command -v "$runtime" >/dev/null 2>&1; then
  if "$runtime" info >/dev/null 2>&1; then
    echo "  ok   container  $runtime is running (make linux-test and friends work)"
  else
    echo "  note container  $runtime is installed but not running. make linux-* needs it: start OrbStack (or your container runtime)"
  fi
elif [ "$os" = "Darwin" ]; then
  echo "  note container  no container runtime. make linux-* is not available. To get one, install OrbStack: https://orbstack.dev"
fi

if [ "$fail" -ne 0 ]; then echo "not ready"; exit 1; fi
echo "ready"
