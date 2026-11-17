#!/usr/bin/env bash
# Release waymark: checks, version, archive, tag, GitHub release, tap formula, verification.
# Usage: scripts/release.sh <version> [--check] [--with-app]
#   --check      run only the checks before a release and change nothing
#   --with-app   also build, sign, and notarize the menu bar app, attach WaymarkBar-<version>.zip, and
#                update the cask (see specs/features/menu-bar-app). Needs a Developer ID identity and a
#                notarytool profile (WAYMARK_NOTARY_PROFILE, default waymark).
# It stops at the first step that fails. The notes of the GitHub release are the entry of the
# version in CHANGELOG.md (see specs/foundation/distribution, section "Release process").
set -euo pipefail
cd "$(dirname "$0")/.."

OWNER=ionpulse-dev
REPO=waymark
TAP=homebrew-tap
# The maintainer releases from a Mac. The Linux x86_64 archive is built in the development container.
LINUX_TARGET=x86_64-unknown-linux-gnu
TARGET=${TARGET:-$(rustc -vV | sed -n 's/^host: //p')}

version="${1:-}"
mode=release
with_app=0
for opt in "${@:2}"; do
  case "$opt" in
    --check) mode=--check ;;
    --with-app) with_app=1 ;;
    *) echo "release: unknown option $opt" >&2; exit 2 ;;
  esac
done
step="start"
trap 'echo; echo "release: FAILED in step: $step" >&2' ERR
say() { step="$1"; echo; echo "==> $1"; }

[ -n "$version" ] || { echo "usage: scripts/release.sh <version> [--check] [--with-app]" >&2; exit 2; }
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || { echo "release: the version must look like 0.1.6" >&2; exit 2; }

say "the changelog has an entry for $version"
notes=$(mktemp)
bash scripts/changelog-entry.sh "$version" > "$notes"
head -c 300 "$notes"; echo

say "the repository is ready"
current=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
[ "$(printf '%s\n%s\n' "$current" "$version" | sort -V | tail -1)" = "$version" ] \
  || { echo "release: $version is older than the version $current in Cargo.toml" >&2; exit 1; }
[ "$(git rev-parse --abbrev-ref HEAD)" = "main" ] || { echo "release: not on the branch main" >&2; exit 1; }
dirty=$(git status --porcelain | grep -v '\.DS_Store' || true)
[ -z "$dirty" ] || { echo "release: the working tree is not clean:" >&2; echo "$dirty" >&2; exit 1; }
git fetch -q origin
[ "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" ] || { echo "release: main is not in sync with origin/main" >&2; exit 1; }
! git rev-parse -q --verify "refs/tags/v$version" >/dev/null || { echo "release: the tag v$version exists" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "release: gh is not signed in" >&2; exit 1; }
gh repo view "$OWNER/$TAP" >/dev/null 2>&1 || { echo "release: the tap $OWNER/$TAP is not reachable" >&2; exit 1; }
bash scripts/linux.sh true >/dev/null 2>&1 || { echo "release: the Linux container does not work. Start OrbStack (or your container runtime), then run again. See make doctor." >&2; exit 1; }
if [ $with_app = 1 ]; then
  say "the menu bar app can be signed and notarized"
  security find-identity -v -p codesigning 2>/dev/null | grep -q '"Developer ID Application:' \
    || { echo "release: --with-app needs a Developer ID Application identity in the keychain" >&2; exit 1; }
  xcrun notarytool history --keychain-profile "${WAYMARK_NOTARY_PROFILE:-waymark}" >/dev/null 2>&1 \
    || { echo "release: --with-app needs the notarytool profile ${WAYMARK_NOTARY_PROFILE:-waymark}" >&2; exit 1; }
fi

if [ "$mode" = "--check" ]; then
  echo; echo "release: checks passed for $version. Nothing was changed."
  exit 0
fi

say "make check, spec-check, public-check"
make check >/dev/null
bash scripts/public-check.sh >/dev/null

say "set the version $version"
if [ "$current" != "$version" ]; then
  sed -i.bak "s/^version = \"$current\"/version = \"$version\"/" Cargo.toml && rm -f Cargo.toml.bak
  cargo build -q
  git add Cargo.toml Cargo.lock
  git commit -q -m "Release $version"
  git push -q origin main
fi

say "build the archive"
bash scripts/package.sh >/dev/null
archive="dist/waymark-$version-$TARGET.tar.gz"
sha=$(cut -d' ' -f1 "$archive.sha256")
echo "sha256 $sha"

say "build the Linux archive (x86_64, in the development container)"
ARCH=amd64 bash scripts/linux.sh bash scripts/package.sh >/dev/null
linux_archive="dist/waymark-$version-$LINUX_TARGET.tar.gz"
[ -f "$linux_archive" ] && [ -f "$linux_archive.sha256" ] || { echo "release: the Linux archive was not built" >&2; exit 1; }
linux_sha=$(cut -d' ' -f1 "$linux_archive.sha256")
echo "sha256 $linux_sha"

assets=("$archive" "$archive.sha256" "$linux_archive" "$linux_archive.sha256")
if [ $with_app = 1 ]; then
  say "build, sign, and notarize the menu bar app"
  WAYMARK_APP_VERSION="$version" bash scripts/macos-app.sh --dist
  app_zip="dist/WaymarkBar-$version.zip"
  cp apps/macos/build/WaymarkBar.zip "$app_zip"
  shasum -a 256 "$app_zip" | awk '{print $1}' > "$app_zip.sha256"
  app_sha=$(cat "$app_zip.sha256")
  assets+=("$app_zip" "$app_zip.sha256")
fi

say "tag and GitHub release"
git tag "v$version"
git push -q origin "v$version"
# The release workflow may have made the release already (it adds the Linux archive). Both paths are safe.
if gh release view "v$version" >/dev/null 2>&1; then
  gh release upload "v$version" "${assets[@]}" --clobber >/dev/null
  gh release edit "v$version" --title "waymark $version" --notes-file "$notes" >/dev/null
else
  gh release create "v$version" "${assets[@]}" --title "waymark $version" --notes-file "$notes" >/dev/null
fi
echo "https://github.com/$OWNER/$REPO/releases/tag/v$version"

say "update the tap"
tmp=$(mktemp -d)
gh repo clone "$OWNER/$TAP" "$tmp/tap" >/dev/null 2>&1
sed -e "s/VERSION/$version/g" -e "s/\"SHA256\"/\"$sha\"/" packaging/homebrew/waymark.rb > "$tmp/tap/Formula/waymark.rb"
if [ $with_app = 1 ]; then
  mkdir -p "$tmp/tap/Casks"
  sed -e "s/\"VERSION\"/\"$version\"/" -e "s/\"SHA256\"/\"$app_sha\"/" packaging/homebrew/waymark-bar.rb > "$tmp/tap/Casks/waymark-bar.rb"
fi
git -C "$tmp/tap" add -A
git -C "$tmp/tap" commit -q -m "waymark $version"
git -C "$tmp/tap" push -q origin HEAD

say "verify the published release"
# Inside the project, so that a container runtime that shares only the home folder (Colima) sees it.
mkdir -p dist
chk=$(mktemp -d "$PWD/dist/.verify.XXXXXX")
curl -sL "https://github.com/$OWNER/$REPO/releases/download/v$version/waymark-$version-$TARGET.tar.gz" -o "$chk/a.tgz"
[ "$(shasum -a 256 "$chk/a.tgz" | cut -d' ' -f1)" = "$sha" ] || { echo "release: the published archive has another checksum" >&2; exit 1; }
tar -xzf "$chk/a.tgz" -C "$chk"
[ "$("$chk/waymark" --version)" = "waymark $version" ] || { echo "release: the published binary reports another version" >&2; exit 1; }
curl -sL "https://github.com/$OWNER/$REPO/releases/download/v$version/waymark-$version-$LINUX_TARGET.tar.gz" -o "$chk/l.tgz"
[ "$(shasum -a 256 "$chk/l.tgz" | cut -d' ' -f1)" = "$linux_sha" ] || { echo "release: the published Linux archive has another checksum" >&2; exit 1; }
mkdir "$chk/linux" && tar -xzf "$chk/l.tgz" -C "$chk/linux"
[ "$("${CONTAINER_RUNTIME:-docker}" run --rm --platform linux/amd64 -v "$chk/linux:/t:ro" ubuntu:22.04 /t/waymark --version)" = "waymark $version" ] || { echo "release: the published Linux binary reports another version" >&2; exit 1; }
if [ $with_app = 1 ]; then
  curl -sL "https://github.com/$OWNER/$REPO/releases/download/v$version/WaymarkBar-$version.zip" -o "$chk/app.zip"
  [ "$(shasum -a 256 "$chk/app.zip" | cut -d' ' -f1)" = "$app_sha" ] || { echo "release: the published app has another checksum" >&2; exit 1; }
fi
brew update >/dev/null 2>&1 || true
brew audit --strict --new "$OWNER/tap/$REPO" >/dev/null 2>&1 || { echo "release: brew audit failed" >&2; exit 1; }
[ "$(git rev-parse HEAD)" = "$(git rev-parse "v$version^{commit}")" ] || { echo "release: the tag does not point at HEAD" >&2; exit 1; }

trap - ERR
echo
echo "release: waymark $version is published and verified."
echo "Update on your machine: brew update && brew upgrade $OWNER/tap/$REPO"
