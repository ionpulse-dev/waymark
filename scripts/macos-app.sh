#!/usr/bin/env bash
# Build the menu bar app (see specs/features/menu-bar-app): swift build, the app bundle, and the signature.
#   bash scripts/macos-app.sh          build apps/macos/build/WaymarkBar.app, signed with a Developer ID
#                                      identity if this Mac has one, else ad hoc (runs on this Mac only)
#   bash scripts/macos-app.sh --dist   also notarize and staple, and make WaymarkBar.zip and its checksum.
#                                      Needs a Developer ID identity and a notarytool keychain profile.
#   bash scripts/macos-app.sh --strings  write Localizable.strings from every L("...") key in the sources.
# Environment: WAYMARK_SIGN_IDENTITY (default: the first "Developer ID Application" identity),
#              WAYMARK_NOTARY_PROFILE (default: waymark), WAYMARK_APP_VERSION (default: Cargo.toml).
set -euo pipefail
cd "$(dirname "$0")/.."
[ "$(uname -s)" = Darwin ] || { echo "macos-app: macOS only" >&2; exit 1; }
dist=0
[ "${1:-}" = "--dist" ] && dist=1
pkg=apps/macos
if [ "${1:-}" = "--strings" ]; then
  f=$pkg/Sources/WaymarkBar/Resources/en.lproj/Localizable.strings
  { echo "/* All window strings of the menu bar app. The key is the English text. */"; echo
    grep -rhoE '\bL\("([^"\\]|\\.)*"' $pkg/Sources/WaymarkBar | sed -E 's/^L\(//' | LC_ALL=C sort -u | while IFS= read -r k; do
      echo "$k = $k;"
    done; } > "$f"
  plutil -lint -s "$f"
  echo "macos-app: wrote $f"
  exit 0
fi
out=$pkg/build
app=$out/WaymarkBar.app
version=${WAYMARK_APP_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}
say() { echo "macos-app: $*"; }

# Every window string must be in the strings file.
strings=$pkg/Sources/WaymarkBar/Resources/en.lproj/Localizable.strings
missing=$(grep -rhoE '\bL\("([^"\\]|\\.)*"' $pkg/Sources/WaymarkBar | sed -E 's/^L\(//' | sort -u | while read -r k; do
  grep -qF -- "$k = " "$strings" || echo "$k"
done)
[ -z "$missing" ] || { echo "macos-app: strings missing in $strings:" >&2; echo "$missing" >&2; exit 1; }
plutil -lint -s "$strings"

say "build $version"
swift build -c release --package-path "$pkg" --product WaymarkBar >/dev/null
bin="$(swift build -c release --package-path "$pkg" --show-bin-path)/WaymarkBar"

rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources/en.lproj"
cp "$bin" "$app/Contents/MacOS/WaymarkBar"
sed "s/VERSION/$version/g" "$pkg/Info.plist" > "$app/Contents/Info.plist"
cp "$strings" "$app/Contents/Resources/en.lproj/"
plutil -lint -s "$app/Contents/Info.plist"

identity=${WAYMARK_SIGN_IDENTITY:-$(security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Developer ID Application:[^"]*\)".*/\1/p' | head -1)}
if [ -n "$identity" ]; then
  say "sign with a Developer ID identity"
  codesign --force --options runtime --timestamp --sign "$identity" "$app"
else
  [ $dist = 0 ] || { echo "macos-app: --dist needs a Developer ID Application identity in the keychain" >&2; exit 1; }
  say "no Developer ID identity: signed ad hoc, for this Mac only"
  codesign --force --sign - "$app"
fi
codesign --verify --strict "$app"

if [ $dist = 1 ]; then
  profile=${WAYMARK_NOTARY_PROFILE:-waymark}
  xcrun notarytool history --keychain-profile "$profile" >/dev/null 2>&1 \
    || { echo "macos-app: no notarytool profile \"$profile\" (xcrun notarytool store-credentials $profile)" >&2; exit 1; }
  rm -f "$out/WaymarkBar.zip"
  ditto -c -k --keepParent "$app" "$out/WaymarkBar.zip"
  say "notarize"
  xcrun notarytool submit "$out/WaymarkBar.zip" --keychain-profile "$profile" --wait
  xcrun stapler staple "$app"
  rm -f "$out/WaymarkBar.zip"
  ditto -c -k --keepParent "$app" "$out/WaymarkBar.zip"
  shasum -a 256 "$out/WaymarkBar.zip" | awk '{print $1}' > "$out/WaymarkBar.zip.sha256"
  spctl --assess --type execute "$app"
  say "$out/WaymarkBar.zip"
fi
say "$app"
