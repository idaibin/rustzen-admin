#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
architecture=${1:?architecture is required}
bin_dir=${2:-"$root/target/rz/build/$architecture/bin"}
case "$architecture" in
  aarch64) target_triple=aarch64-unknown-linux-musl; platform=linux/arm64 ;;
  x86_64) target_triple=x86_64-unknown-linux-musl; platform=linux/amd64 ;;
  *) echo "unsupported architecture: $architecture" >&2; exit 1 ;;
esac
if [ "$#" -ge 5 ]; then
  head=$3
  state=$4
  source_digest=$5
else
  read -r head state source_digest < <("$root/scripts/admin-browser-source-identity.sh")
fi
output="$bin_dir/build-provenance.txt"
temporary="$output.tmp"
{
  printf 'schemaVersion\t1\n'
  printf 'gitHead\t%s\n' "$head"
  printf 'sourceTreeState\t%s\n' "$state"
  printf 'sourceTreeSha256\t%s\n' "$source_digest"
  printf 'architecture\t%s\n' "$architecture"
  printf 'targetTriple\t%s\n' "$target_triple"
  printf 'platform\t%s\n' "$platform"
  printf 'distribution\tfull\n'
  for name in rz-admin rz-monitor rz-insights rz-reports; do
    test -x "$bin_dir/$name"
    printf '%s\t%s\n' "$name" "$(shasum -a 256 "$bin_dir/$name" | awk '{print $1}')"
  done
} >"$temporary"
mv "$temporary" "$output"
