#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
architecture=$(docker info --format '{{.Architecture}}')
case "$architecture" in
  aarch64) target_triple=aarch64-unknown-linux-musl; platform=linux/arm64 ;;
  x86_64) target_triple=x86_64-unknown-linux-musl; platform=linux/amd64 ;;
  *) echo "unsupported Docker architecture: $architecture" >&2; exit 1 ;;
esac
lock_dir="$root/target/rz/build/.admin-browser-$architecture.lock"
mkdir -p "$(dirname -- "$lock_dir")"
if ! mkdir "$lock_dir" 2>/dev/null; then
  echo "another Admin browser build owns $lock_dir" >&2
  exit 1
fi
cleanup() { rmdir "$lock_dir" 2>/dev/null || true; }
trap cleanup EXIT INT TERM

read -r initial_head initial_state initial_digest < <("$root/scripts/admin-browser-source-identity.sh")
just _build-binaries "$architecture" "$target_triple" "$platform"
read -r final_head final_state final_digest < <("$root/scripts/admin-browser-source-identity.sh")
if [ "$final_head" != "$initial_head" ] || [ "$final_state" != "$initial_state" ] || [ "$final_digest" != "$initial_digest" ]; then
  echo "source tree changed during Admin browser binary build" >&2
  exit 1
fi
"$root/scripts/write-admin-browser-build-provenance.sh" \
  "$architecture" "$root/target/rz/build/$architecture/bin" \
  "$initial_head" "$initial_state" "$initial_digest"
