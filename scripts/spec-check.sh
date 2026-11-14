#!/usr/bin/env bash
# Check the spec folders. Usage: spec-check.sh [repo-root]
set -u
root="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$root" || exit 2
fail=0
err() { echo "spec-check: $1"; fail=1; }
status_of() { sed -n 's/^status: *//p' "$1" | head -1; }
index_status() { # name -> status from INDEX.md table row
  grep -E "^\| \[$1\]" specs/INDEX.md | head -1 | sed -E 's/.*`([a-z-]+)`.*/\1/'
}
for dir in specs/features/*/ specs/foundation/*/; do
  name=$(basename "$dir")
  req="$dir/requirements.md"
  [ -f "$req" ] || { err "$dir has no requirements.md"; continue; }
  st=$(status_of "$req")
  [ "$st" = "superseded" ] && continue
  for f in requirements.md plan.md tasks.md test.md decisions.md todo.md; do
    # draft specs need only requirements.md and decisions.md
    if [ "$st" = "draft" ] && [ "$f" != "requirements.md" ] && [ "$f" != "decisions.md" ]; then continue; fi
    [ -f "$dir/$f" ] || err "$dir is missing $f"
  done
  ist=$(index_status "$name")
  if [ -z "$ist" ]; then err "$name is not in specs/INDEX.md"
  elif [ "$ist" != "$st" ]; then err "$name: status is $st but INDEX.md says $ist"; fi
  # requires entries must name existing specs
  for r in $(awk '/^requires:/{f=1;next} f&&/^  - /{print $2;next} f{exit}' "$req"); do
    [ -d "specs/features/$r" ] || [ -d "specs/foundation/$r" ] || err "$name requires $r, which does not exist"
  done
done
# relative links in specs and docs
for f in $(find specs docs -name '*.md' -not -path 'specs/templates/*'); do
  d=$(dirname "$f")
  for link in $(sed 's/`[^`]*`//g' "$f" | grep -o '](\([^)#:]*\.md\)' | sed 's/](//'); do
    case "$link" in http*|/*) continue ;; esac
    [ -e "$d/$link" ] || err "$f links to missing $link"
  done
done
[ "$fail" -eq 0 ] && echo "spec-check: ok"
exit "$fail"
