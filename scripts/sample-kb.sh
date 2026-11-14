#!/usr/bin/env bash
# `create`: make a sample knowledge base in tmp/sample-kb (a bare remote plus a clone).
# `run`: start the server on it.
set -euo pipefail
cd "$(dirname "$0")/.."
BASE=tmp/sample-kb
create() {
  rm -rf "$BASE" && mkdir -p "$BASE"
  git init -q --bare -b main "$BASE/remote.git"
  git clone -q "$BASE/remote.git" "$BASE/clone" 2>/dev/null
  cp -R standard/conformance/vault/. "$BASE/clone/" 2>/dev/null || mkdir -p "$BASE/clone/knowledge"
  (cd "$BASE/clone" && git add -A && git -c user.name=Sample -c user.email=sample@example.org commit -qm "Sample" --allow-empty && git push -q origin HEAD:main)
  echo "$BASE/clone"
}
case "${1:-create}" in
  create) create ;;
  run)
    [ -d "$BASE/clone" ] || create >/dev/null
    cargo run -q -- --path "$BASE/clone" --author-name Sample --author-email sample@example.org ;;
  *) echo "usage: sample-kb.sh create|run" >&2; exit 2 ;;
esac
