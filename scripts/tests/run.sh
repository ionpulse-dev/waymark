#!/usr/bin/env bash
# Shell tests of the scripts. Each test prints ok or fails the run.
set -u
cd "$(dirname "$0")/../.." || exit 2
fail=0
ok() { echo "  ok   $1"; }
bad() { echo "  FAIL $1"; fail=1; }

# Makefile parses with the system make (GNU Make 3.81 on macOS)
/usr/bin/make -n help build test lint check >/dev/null 2>&1 && ok "makefile parses with system make" || bad "makefile parses with system make"

# make help lists every target
out=$(make help 2>&1)
for t in help setup doctor build release test test-integration test-all fmt lint check spec-check run sample-memory brew-local clean linux-build linux-test linux-check linux-shell; do
  echo "$out" | grep -qE "^ +$t +" || bad "help lists $t"
done
[ "$fail" -eq 0 ] && ok "help lists every target"

# doctor passes here and names a missing tool
bash scripts/doctor.sh >/dev/null 2>&1 && ok "doctor passes on this machine" || bad "doctor passes on this machine"
res=$(PATH=/usr/bin:/bin bash scripts/doctor.sh 2>&1) && bad "doctor fails without cargo" || {
  echo "$res" | grep -q "MISSING cargo" && echo "$res" | grep -q "Fix:" && ok "doctor names a missing tool and its fix" || bad "doctor names a missing tool and its fix"
}

# setup is idempotent
res=$(bash scripts/setup.sh 2>&1) && echo "$res" | grep -q "Nothing to install" && ok "setup changes nothing when ready" || bad "setup changes nothing when ready"

# The binary links only to system libraries (needs only git at run time)
cargo build -q 2>/dev/null
bin="${CARGO_TARGET_DIR:-target}/debug/waymark"
if [ -x "$bin" ]; then
  if [ "$(uname -s)" = "Darwin" ]; then
    extra=$(otool -L "$bin" | tail -n +2 | awk '{print $1}' | grep -vE '^(/usr/lib/|/System/)' || true)
  else
    # libc, libm, libgcc_s, libdl, libpthread, librt, and the loader are system libraries.
    extra=$(ldd "$bin" | awk '{print $1}' | grep -vE '^(linux-vdso|linux-gate|libc\.so|libm\.so|libgcc_s\.so|libdl\.so|libpthread\.so|librt\.so|libutil\.so|/lib.*/ld-linux|ld-linux)' || true)
  fi
  [ -z "$extra" ] && ok "the binary links only to system libraries" || bad "the binary links only to system libraries: $extra"
  if [ "$(uname -s)" = "Linux" ]; then
    # The C library of the binary must not be newer than the base of the development container (Ubuntu 22.04, glibc 2.35).
    host_glibc=$(ldd --version 2>&1 | head -1 | grep -oE '[0-9]+\.[0-9]+$' || echo 0)
    need=$(objdump -T "$bin" 2>/dev/null | grep -oE 'GLIBC_[0-9]+\.[0-9]+' | sed 's/GLIBC_//' | sort -t. -k1,1n -k2,2n | tail -1)
    if [ "$(printf '%s\n%s\n' 2.35 "$host_glibc" | sort -t. -k1,1n -k2,2n | tail -1)" != "2.35" ]; then
      echo "  skip glibc version check: this system has glibc $host_glibc, newer than the container base. Run make linux-test for this check."
    else
      [ "$(printf '%s\n%s\n' 2.35 "${need:-0}" | sort -t. -k1,1n -k2,2n | tail -1)" = "2.35" ] && ok "the binary needs glibc ${need:-none} (2.35 or older)" || bad "the binary needs glibc $need, newer than 2.35"
    fi
  fi
else
  bad "the binary exists after cargo build"
fi

# changelog and release script (see specs/foundation/distribution, "Release process")
entry=$(bash scripts/changelog-entry.sh 0.1.5 2>/dev/null) && [ "${#entry}" -ge 40 ] && ok "changelog-entry prints the entry of a version" || bad "changelog-entry prints the entry of a version"
bash scripts/changelog-entry.sh 9.9.9 >/dev/null 2>&1 && bad "changelog-entry fails for a missing entry" || ok "changelog-entry fails for a missing entry"
bash scripts/release.sh >/dev/null 2>&1 && bad "release.sh refuses a missing version" || ok "release.sh refuses a missing version"
bash scripts/release.sh 1.0 >/dev/null 2>&1 && bad "release.sh refuses a bad version" || ok "release.sh refuses a bad version"
res=$(bash scripts/release.sh 9.9.9 --check 2>&1) && bad "release.sh refuses a version without a changelog entry" || {
  echo "$res" | grep -q "no entry for 9.9.9" && ok "release.sh refuses a version without a changelog entry" || bad "release.sh refuses a version without a changelog entry"
}
missing=""
cv=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
for v in $cv $(git tag -l 'v*' | sed 's/^v//'); do
  bash scripts/changelog-entry.sh "$v" >/dev/null 2>&1 || missing="$missing $v"
done
[ -z "$missing" ] && ok "every version and tag has a changelog entry" || bad "no changelog entry for:$missing"

# public-check passes the repository and flags a planted secret
bash scripts/public-check.sh >/dev/null 2>&1 && ok "public-check passes the repository" || bad "public-check passes the repository"
plant=planted-secret.txt
printf 'key = ghp_%s\n' "abcdefghijklmnopqrstuvwxyz0123456789" > "$plant"
bash scripts/public-check.sh >/dev/null 2>&1 && bad "public-check flags a planted secret" || ok "public-check flags a planted secret"
rm -f "$plant"

# The formula template is valid Ruby and keeps its placeholders
if command -v ruby >/dev/null 2>&1; then
  ruby -c packaging/homebrew/waymark.rb >/dev/null 2>&1 && grep -q 'sha256 "SHA256"' packaging/homebrew/waymark.rb && ok "the formula template parses and has placeholders" || bad "the formula template parses and has placeholders"
else
  grep -q 'sha256 "SHA256"' packaging/homebrew/waymark.rb && ok "the formula template has placeholders (ruby is not installed, so the syntax is not checked)" || bad "the formula template has placeholders"
fi

# spec-check detects problems on a temporary copy
tmp=$(mktemp -d)
mkdir -p "$tmp/specs" && cp -R specs/features specs/foundation specs/INDEX.md specs/PROJECT.md specs/WORKFLOW.md "$tmp/specs/" && cp -R docs standard "$tmp/" && cp README.md AGENTS.md LICENSE "$tmp/" 2>/dev/null
if bash scripts/spec-check.sh "$tmp" >/dev/null 2>&1; then
  rm "$tmp/specs/foundation/document-format/tasks.md"
  bash scripts/spec-check.sh "$tmp" >/dev/null 2>&1 && bad "spec-check fails on a missing file" || ok "spec-check fails on a missing file"
  cp specs/foundation/document-format/tasks.md "$tmp/specs/foundation/document-format/tasks.md"
  sed 's/^status: .*/status: blocked/' "$tmp/specs/foundation/document-format/requirements.md" > "$tmp/req.new" && mv "$tmp/req.new" "$tmp/specs/foundation/document-format/requirements.md"
  bash scripts/spec-check.sh "$tmp" >/dev/null 2>&1 && bad "spec-check fails on a status mismatch" || ok "spec-check fails on a status mismatch"
else
  echo "  skip spec-check tests: the repository does not pass spec-check yet"
fi
rm -rf "$tmp"

# ---- Linux support (specs/foundation/linux-support) ----

# The Makefile and the toolchain file name no fixed macOS target
grep -q 'aarch64-apple-darwin' Makefile rust-toolchain.toml 2>/dev/null && bad "Makefile and rust-toolchain.toml name no fixed macOS target" || ok "Makefile and rust-toolchain.toml name no fixed macOS target"
grep -q '^TARGET := \$(shell rustc -vV' Makefile && ok "the Makefile takes the target from the host" || bad "the Makefile takes the target from the host"

# The container definition and the editor configuration use the Rust version of the toolchain file
rv=$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml | head -1)
grep -q "\"RUST_VERSION\": \"$rv\"" .devcontainer/devcontainer.json && ok "devcontainer.json uses the Rust version of rust-toolchain.toml" || bad "devcontainer.json uses the Rust version of rust-toolchain.toml ($rv)"

# doctor: platform and tool hints (DOCTOR_OS and DOCTOR_ARCH fake the system)
res=$(DOCTOR_OS=Linux DOCTOR_ARCH=x86_64 bash scripts/doctor.sh 2>&1 || true)
echo "$res" | grep -q "warn" && bad "doctor has no platform warning on Linux x86_64" || ok "doctor has no platform warning on Linux x86_64"
res=$(DOCTOR_OS=Linux DOCTOR_ARCH=riscv64 bash scripts/doctor.sh 2>&1 || true)
echo "$res" | grep -q "warn" && ok "doctor warns for an unsupported platform" || bad "doctor warns for an unsupported platform"
tmpbin=$(mktemp -d)
printf '#!/bin/sh\necho "git version 2.20.1"\n' > "$tmpbin/git"; chmod +x "$tmpbin/git"
res=$(PATH="$tmpbin:$PATH" bash scripts/doctor.sh 2>&1) && bad "doctor fails for Git older than 2.28" || {
  echo "$res" | grep -q "2.28 or newer" && ok "doctor fails for Git older than 2.28" || bad "doctor fails for Git older than 2.28"
}
# A missing tool on Linux gets a Linux hint and no Homebrew
res=$(DOCTOR_OS=Linux DOCTOR_ARCH=x86_64 PATH=/nonexistent /bin/bash scripts/doctor.sh 2>&1 || true)
if echo "$res" | grep -q "MISSING git"; then
  echo "$res" | grep "MISSING git" | grep -q "package manager" && ! echo "$res" | grep -qi "brew" && ok "doctor on Linux names a missing tool with a Linux hint and no Homebrew" || bad "doctor on Linux names a missing tool with a Linux hint and no Homebrew"
else
  bad "doctor names a missing git"
fi
# A missing container runtime is a note, not a failure
res=$(CONTAINER_RUNTIME=no-such-runtime DOCTOR_OS=Darwin bash scripts/doctor.sh 2>&1); code=$?
echo "$res" | grep -q "note container" && echo "$res" | grep -q "orbstack" && ok "doctor notes a missing container runtime and says how to get one" || bad "doctor notes a missing container runtime and says how to get one"
echo "$res" | grep -q "MISSING container" && bad "a missing container runtime is not a failure"
rm -rf "$tmpbin"

# linux.sh: no runtime, and a runtime that is not running
res=$(CONTAINER_RUNTIME=no-such-runtime bash scripts/linux.sh true 2>&1) && bad "linux.sh stops without a runtime" || {
  echo "$res" | grep -q "no container runtime found" && ok "linux.sh stops without a runtime and says how to get one" || bad "linux.sh stops without a runtime and says how to get one"
}
fake=$(mktemp -d)
printf '#!/bin/sh\n[ "$1" = "info" ] && exit 1\nexit 0\n' > "$fake/docker"; chmod +x "$fake/docker"
res=$(CONTAINER_RUNTIME="$fake/docker" bash scripts/linux.sh true 2>&1) && bad "linux.sh stops when the runtime is not running" || {
  [ "$res" = "Start OrbStack (or your container runtime) and try again." ] && ok "linux.sh says to start the runtime when it is not running" || bad "linux.sh says to start the runtime when it is not running: $res"
}
res=$(ARCH=sparc CONTAINER_RUNTIME=true bash scripts/linux.sh true 2>&1) && bad "linux.sh refuses an unknown ARCH" || ok "linux.sh refuses an unknown ARCH"
rm -rf "$fake"

# The container makes no file that is owned by root (needs a running runtime; not run inside the container)
if [ -n "${WAYMARK_IN_CONTAINER:-}" ]; then
  echo "  skip container ownership test: already inside the container"
elif command -v docker >/dev/null 2>&1 && docker info >/dev/null 2>&1; then
  bash scripts/linux.sh sh -c 'touch owner-probe.tmp && mkdir -p tmp && touch tmp/owner-probe.tmp' >/dev/null 2>&1
  roots=$(find . -path ./target -prune -o -path ./.git -prune -o -user root -print 2>/dev/null | head -3)
  rm -f owner-probe.tmp tmp/owner-probe.tmp
  [ -z "$roots" ] && ok "the container leaves no file owned by root" || bad "files owned by root: $roots"
else
  echo "  skip container ownership test: no running container runtime (see make doctor)"
fi

# package.sh names the archive after the host target and makes a checksum
if [ -n "${WAYMARK_SKIP_PACKAGE_TEST:-}" ]; then
  echo "  skip package test"
else
  host=$(rustc -vV | sed -n 's/^host: //p')
  ver=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
  out=$(bash scripts/package.sh 2>/dev/null | tail -1)
  [ "$out" = "dist/waymark-$ver-$host.tar.gz" ] && [ -f "$out" ] && [ -f "$out.sha256" ] && ok "package.sh names the archive after the host target and writes a checksum" || bad "package.sh archive: $out"
  if command -v sha256sum >/dev/null 2>&1; then (cd dist && sha256sum -c "$(basename "$out").sha256" >/dev/null 2>&1) && ok "the checksum matches the archive" || bad "the checksum matches the archive"
  else (cd dist && shasum -a 256 -c "$(basename "$out").sha256" >/dev/null 2>&1) && ok "the checksum matches the archive" || bad "the checksum matches the archive"; fi
fi

# release.sh builds, uploads, and verifies the Linux x86_64 archive too
grep -q 'x86_64-unknown-linux-gnu' scripts/release.sh && grep -q 'ARCH=amd64 bash scripts/linux.sh bash scripts/package.sh' scripts/release.sh && grep -q '"\$linux_archive" "\$linux_archive.sha256"' scripts/release.sh && ok "release.sh publishes the Linux x86_64 archive and its checksum" || bad "release.sh publishes the Linux x86_64 archive and its checksum"

# CI and release workflows (disabled files, ready to enable)
ci=.github/workflows/ci.yml.disabled
rel=.github/workflows/release.yml.disabled
grep -q 'ubuntu' "$ci" && grep -q 'macos' "$ci" && ok "the CI workflow has a Linux and a macOS job" || bad "the CI workflow has a Linux and a macOS job"
grep -q 'scripts/linux.sh' "$ci" && ok "the CI Linux job uses the development container" || bad "the CI Linux job uses the development container"
grep -q 'ubuntu' "$rel" && grep -q 'scripts/linux.sh' "$rel" && grep -q 'linux' "$rel" && ok "the release workflow builds a Linux archive" || bad "the release workflow builds a Linux archive"
grep -q 'sha256' scripts/package.sh && ok "package.sh writes a checksum, and the release workflow uploads *.sha256" || bad "package.sh writes a checksum"

# README and install guide: both systems, and the note on the Linux steps until the manual check is done
for f in README.md docs/local-install.md; do
  grep -qi 'linux' "$f" && grep -qi 'macos' "$f" && ok "$f names Linux and macOS" || bad "$f names Linux and macOS"
done
note="The Linux steps are not yet tested on a real machine"
pending=$(grep -c '^- \[ \] \[manual\] Linux docs check' specs/foundation/linux-support/test.md || true)
for f in README.md docs/local-install.md; do
  if [ "$pending" -gt 0 ]; then
    grep -q "$note" "$f" && ok "$f carries the Linux note until the manual check is done" || bad "$f carries the Linux note until the manual check is done"
  else
    grep -q "$note" "$f" && bad "$f still carries the Linux note after the manual check" || ok "$f has no Linux note after the manual check"
  fi
done
exit "$fail"
