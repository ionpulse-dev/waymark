#!/usr/bin/env bash
# Checks the container image of the personal server (specs/foundation/personal-server, test A13).
# It builds the image, so it takes a few minutes the first time. It skips when no container runtime runs.
set -u
cd "$(dirname "$0")/../.." || exit 2
runtime="${CONTAINER_RUNTIME:-docker}"
if ! command -v "$runtime" >/dev/null 2>&1 || ! "$runtime" info >/dev/null 2>&1; then
  echo "  skip image checks: no container runtime is running ($runtime)"
  exit 0
fi
fail=0
ok() { echo "  ok   $1"; }
bad() { echo "  FAIL $1"; fail=1; }
tag="waymark-personal-server:test"
"$runtime" build -q -t "$tag" . >/dev/null 2>&1 && ok "the image builds" || { bad "the image builds"; exit 1; }

# It runs as a normal user and starts the binary.
[ "$("$runtime" run --rm --entrypoint id "$tag" -u)" != "0" ] && ok "the image runs as a normal user" || bad "the image runs as a normal user"
"$runtime" run --rm "$tag" --version | grep -q '^waymark' && ok "the image starts waymark" || bad "the image starts waymark"

# With no settings the server stops and names what is missing. It does not listen.
res=$("$runtime" run --rm "$tag" 2>&1) && bad "the image stops without settings" || {
  echo "$res" | grep -q "WAYMARK_OWNER" && ok "the image stops without settings and names them" || bad "the image stops without settings and names them: $res"
}

# The image holds no secret: no private key anywhere in its layers, and none of the secret names with a value.
dir=$(mktemp -d)
"$runtime" save "$tag" -o "$dir/image.tar" && tar -xf "$dir/image.tar" -C "$dir"
if grep -rlaE -e '-----BEGIN [A-Z ]*PRIVATE KEY-----' "$dir" >/dev/null 2>&1; then bad "the image holds a private key"; else ok "the image holds no private key"; fi
env_values=$("$runtime" inspect --format '{{range .Config.Env}}{{println .}}{{end}}' "$tag")
if echo "$env_values" | grep -qE '^WAYMARK_(SIGNING_KEY|GITHUB_APP_KEY|GITHUB_CLIENT_SECRET|OWNER|REPO)='; then bad "the image sets a secret or a setting of the person"; else ok "the image sets no secret and no setting of the person"; fi
rm -rf "$dir"
exit "$fail"
