#!/usr/bin/env bash
set -euo pipefail
if test "$#" -ne 2 -o "$1" != --owner-token -o -z "$2"; then exit 2; fi
ids="$(docker ps -aq --filter "label=io.rustzen.p8g-owner=$2")"
count="$(printf '%s\n' "$ids" | sed '/^$/d' | wc -l | tr -d ' ')"
test "$count" -le 1 || { echo "multiple retained P8e containers" >&2; exit 1; }
if test "$count" = 1; then docker rm -f "$ids" >/dev/null; fi
test -z "$(docker ps -aq --filter "label=io.rustzen.p8g-owner=$2")"
