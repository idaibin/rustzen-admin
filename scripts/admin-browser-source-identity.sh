#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
state=clean
git -C "$root" diff --quiet HEAD -- || state=dirty
test -z "$(git -C "$root" ls-files --others --exclude-standard)" || state=dirty
digest=$(
  {
    git -C "$root" status --porcelain=v1 -z
    while IFS= read -r -d '' path; do
      printf '%s\0' "$path"
      if [ -f "$root/$path" ]; then shasum -a 256 "$root/$path"; else printf 'absent\n'; fi
    done < <(git -C "$root" ls-files -co --exclude-standard -z | LC_ALL=C sort -z)
  } | shasum -a 256 | awk '{print $1}'
)
printf '%s\t%s\t%s\n' "$(git -C "$root" rev-parse HEAD)" "$state" "$digest"
