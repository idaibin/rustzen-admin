#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
extra_inputs=("$@")
for input in "${extra_inputs[@]}"; do
  case "$input" in
    ''|/*|.|..|./*|../*|*/./*|*/../*|*//*)
      echo "extra source input must be a repository-relative path without traversal: $input" >&2
      exit 2
      ;;
  esac
  current=$root
  IFS=/ read -r -a components <<<"$input"
  for component in "${components[@]}"; do
    current=$current/$component
    if [ -L "$current" ]; then
      echo "extra source input must not contain symlinks: $input" >&2
      exit 2
    fi
    [ -e "$current" ] || break
  done
done
state=clean
git -C "$root" diff --quiet HEAD -- || state=dirty
test -z "$(git -C "$root" ls-files --others --exclude-standard)" || state=dirty
append_extra_input() {
  relative=$1 absolute=$root/$relative
  printf 'extra\0%s\0' "$relative"
  if [ ! -e "$absolute" ]; then
    printf 'missing\0'
  elif [ -f "$absolute" ]; then
    printf 'file\0%s\0' "$(shasum -a 256 "$absolute" | awk '{print $1}')"
  elif [ -d "$absolute" ]; then
    printf 'directory\0'
    if find "$absolute" -type l -print -quit | grep -q .; then
      echo "extra source input must not contain symlinks: $relative" >&2
      exit 2
    fi
    if find "$absolute" ! -type d ! -type f -print -quit | grep -q .; then
      echo "extra source input must contain only regular files: $relative" >&2
      exit 2
    fi
    while IFS= read -r -d '' file; do
      path=${file#"$root/"}
      printf '%s\0%s\0' "$path" "$(shasum -a 256 "$file" | awk '{print $1}')"
    done < <(find "$absolute" -type f -print0 | LC_ALL=C sort -z)
  else
    echo "extra source input must be a regular file or directory: $relative" >&2
    exit 2
  fi
}
digest=$(
  {
    git -C "$root" status --porcelain=v1 -z
    while IFS= read -r -d '' path; do
      printf '%s\0' "$path"
      if [ -f "$root/$path" ]; then shasum -a 256 "$root/$path"; else printf 'absent\n'; fi
    done < <(git -C "$root" ls-files -co --exclude-standard -z | LC_ALL=C sort -z)
    if [ "${#extra_inputs[@]}" -gt 0 ]; then
      while IFS= read -r -d '' path; do append_extra_input "$path"; done < <(
        printf '%s\0' "${extra_inputs[@]}" | LC_ALL=C sort -zu
      )
    fi
  } | shasum -a 256 | awk '{print $1}'
)
printf '%s\t%s\t%s\n' "$(git -C "$root" rev-parse HEAD)" "$state" "$digest"
