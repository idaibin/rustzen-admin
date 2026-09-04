#!/bin/sh
set -eu

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <version> <x86_64|aarch64> <binary-directory> <output-directory>" >&2
  exit 2
fi

VERSION=$1
ARCH=$2
BIN_DIR=$3
OUTPUT_DIR=$4
ROOT_NAME="rz-${VERSION}-${ARCH}"

case "$VERSION" in
  ''|*[!A-Za-z0-9._-]*) echo "invalid version: $VERSION" >&2; exit 2 ;;
esac
case "$ARCH" in
  x86_64|aarch64) ;;
  *) echo "unsupported architecture: $ARCH" >&2; exit 2 ;;
esac

for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
  if [ ! -f "$BIN_DIR/$binary" ] || [ ! -x "$BIN_DIR/$binary" ]; then
    echo "missing executable bundle member: $BIN_DIR/$binary" >&2
    exit 1
  fi
  if [ "$(dd if="$BIN_DIR/$binary" bs=1 count=4 2>/dev/null | od -An -tx1 | tr -d ' \n')" != "7f454c46" ]; then
    echo "bundle member is not an ELF executable: $BIN_DIR/$binary" >&2
    exit 1
  fi
  MACHINE=$(dd if="$BIN_DIR/$binary" bs=1 skip=18 count=2 2>/dev/null | od -An -tx1 | tr -d ' \n')
  case "$ARCH:$MACHINE" in
    x86_64:3e00|aarch64:b700) ;;
    *) echo "bundle member architecture mismatch: $BIN_DIR/$binary" >&2; exit 1 ;;
  esac
  MARKER=$(printf 'RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=%s\nversion=%s' "$binary" "$VERSION")
  if ! grep -aF "$MARKER" "$BIN_DIR/$binary" >/dev/null; then
    echo "bundle member identity marker mismatch: $BIN_DIR/$binary" >&2
    exit 1
  fi
done

if [ ! -f "$OUTPUT_DIR/config/rz.env" ]; then
  echo "missing generated release config: $OUTPUT_DIR/config/rz.env" >&2
  exit 1
fi
if [ ! -f "$OUTPUT_DIR/config/rz-reports.env" ]; then
  echo "missing generated Reports release config: $OUTPUT_DIR/config/rz-reports.env" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
STAGING=$(mktemp -d "${OUTPUT_DIR}/.bundle.XXXXXX")
trap 'rm -rf "$STAGING"' EXIT HUP INT TERM
ROOT="$STAGING/$ROOT_NAME"
mkdir -p "$ROOT/bin" "$ROOT/systemd" "$ROOT/config"

for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
  install -m 0755 "$BIN_DIR/$binary" "$ROOT/bin/$binary"
done
for unit in rz.target rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
  install -m 0644 "deploy/$unit" "$ROOT/systemd/$unit"
done
install -m 0600 "$OUTPUT_DIR/config/rz.env" "$ROOT/config/rz.env"
install -m 0600 "$OUTPUT_DIR/config/rz-reports.env" "$ROOT/config/rz-reports.env"
install -m 0755 deploy/setup-layout.sh "$ROOT/setup-layout.sh"

BUNDLE="$OUTPUT_DIR/$ROOT_NAME.tar"
rm -f "$BUNDLE"
# Emit the same portable headers on BSD tar and GNU tar. macOS file metadata
# otherwise produces extended headers that the strict Linux installer rejects.
COPYFILE_DISABLE=1 tar --format=ustar -cf "$BUNDLE" -C "$STAGING" "$ROOT_NAME"
printf '%s\n' "$BUNDLE"
