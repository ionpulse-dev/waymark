#!/usr/bin/env bash
# Print the entry of one version from CHANGELOG.md, without its heading.
# Fails if the entry is missing or shorter than 40 characters. Usage: changelog-entry.sh <version>
set -euo pipefail
cd "$(dirname "$0")/.."
v="${1:-}"
[ -n "$v" ] || { echo "usage: changelog-entry.sh <version>" >&2; exit 2; }
entry=$(awk -v v="$v" '
  $0 ~ "^## " v "( |$)" { on = 1; next }
  /^## / { on = 0 }
  on { print }
' CHANGELOG.md | sed -e '/./,$!d' | awk 'NF || p { p = 1; print }' )
entry=$(printf '%s\n' "$entry" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}')
if [ "${#entry}" -lt 40 ]; then
  echo "changelog-entry: CHANGELOG.md has no entry for $v (or it is shorter than 40 characters)." >&2
  exit 1
fi
printf '%s\n' "$entry"
