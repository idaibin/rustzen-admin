#!/usr/bin/env sh
set -eu

INSTALL_ROOT="${INSTALL_ROOT:-/opt/rz}"
SYSTEMD_DIR="${SYSTEMD_DIR:-/etc/systemd/system}"
SYSTEMCTL_BIN="${SYSTEMCTL_BIN:-systemctl}"
REPORTS_USER="${RUSTZEN_REPORTS_USER:-rz-reports}"
REPORTS_GROUP="${RUSTZEN_REPORTS_GROUP:-}"
MANAGED_UNITS="rz.target rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service"
REQUIRED_FILES="
bin/rz
bin/rz-admin
bin/rz-monitor
bin/rz-insights
bin/rz-reports
systemd/rz.target
systemd/rz-recovery.service
systemd/rz-admin.service
systemd/rz-monitor.service
systemd/rz-insights.service
systemd/rz-reports.service
config/rz.env
config/rz-reports.env
setup-layout.sh
"

WORK_DIR=""
INSTALL_LOCK=""
CANDIDATE_DIR=""

fail() {
    echo "setup-layout: $*" >&2
    exit 1
}

group_exists() {
    if command -v getent >/dev/null 2>&1; then
        getent group "$1" >/dev/null 2>&1
    elif command -v dscl >/dev/null 2>&1; then
        dscl . -read "/Groups/$1" >/dev/null 2>&1
    else
        return 1
    fi
}

cleanup() {
    if [ -n "$CANDIDATE_DIR" ] && [ -d "$CANDIDATE_DIR" ]; then
        rm -rf "$CANDIDATE_DIR"
    fi
    if [ -n "$INSTALL_LOCK" ] && [ -d "$INSTALL_LOCK" ]; then
        rmdir "$INSTALL_LOCK" 2>/dev/null || true
    fi
    if [ -n "$WORK_DIR" ] && [ -d "$WORK_DIR" ]; then
        rm -rf "$WORK_DIR"
    fi
}

trap cleanup 0 1 2 15

atomic_replace() {
    source_path="$1"
    destination_path="$2"
    case "$(uname -s)" in
        Darwin|FreeBSD|NetBSD|OpenBSD)
            mv -fh "$source_path" "$destination_path"
            ;;
        *)
            mv -fT "$source_path" "$destination_path"
            ;;
    esac
}

hex_to_binary() {
    value="$1"
    if command -v xxd >/dev/null 2>&1; then
        printf '%s' "$value" | xxd -r -p
        return
    fi
    printf '%s\n' "$value" | LC_ALL=C awk '
        function nibble(character) {
            return index("0123456789abcdef", character) - 1
        }
        {
            for (offset = 1; offset <= length($0); offset += 2) {
                printf "%c", nibble(substr($0, offset, 1)) * 16 + nibble(substr($0, offset + 1, 1))
            }
        }
    '
}

verify_bundle_signature() {
    bundle="$1"
    version="$2"
    arch="$3"
    verify_key="$(printf '%s' "${RUSTZEN_DEPLOY_VERIFY_KEY:-}" | tr '[:upper:]' '[:lower:]')"
    case "$verify_key" in
        ""|*[!0-9a-f]*)
            fail "RUSTZEN_DEPLOY_VERIFY_KEY must be a trusted 64-character Ed25519 public key"
            ;;
    esac
    if [ "${#verify_key}" -ne 64 ]; then
        fail "RUSTZEN_DEPLOY_VERIFY_KEY must be a trusted 64-character Ed25519 public key"
    fi

    marker_offset="$(LC_ALL=C grep -aob 'RUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN' "$bundle" \
        | tail -n 1 | cut -d: -f1 || true)"
    case "$marker_offset" in
        ""|*[!0-9]*) fail "signed release bundle marker is missing" ;;
    esac
    if [ "$marker_offset" -lt 1 ]; then
        fail "signed release bundle marker is invalid"
    fi
    content_length=$((marker_offset - 1))
    if [ $((content_length % 512)) -ne 0 ]; then
        fail "signed release bundle marker is not aligned after the tar payload"
    fi
    marker_separator="$(dd if="$bundle" bs=1 skip="$content_length" count=1 2>/dev/null \
        | od -An -tu1 | tr -d '[:space:]')"
    if [ "$marker_separator" != "10" ]; then
        fail "signed release bundle marker separator is invalid"
    fi

    actual_trailer="$WORK_DIR/signed-trailer"
    expected_trailer="$WORK_DIR/signed-trailer.expected"
    dd if="$bundle" bs=1 skip="$marker_offset" of="$actual_trailer" 2>/dev/null
    marker_json="$(sed -n '2p' "$actual_trailer")"
    marker_version="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"version":"\([^"]*\)".*$/\1/p')"
    marker_arch="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"arch":"\([^"]*\)".*$/\1/p')"
    marker_hash="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"contentSha256":"\([0-9a-f]*\)".*$/\1/p')"
    marker_signature="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"signature":"\([0-9a-f]*\)".*$/\1/p')"
    expected_json="$(printf '{"schemaVersion":1,"component":"bundle","version":"%s","arch":"%s","contentSha256":"%s","signature":"%s"}' \
        "$marker_version" "$marker_arch" "$marker_hash" "$marker_signature")"
    if [ "$marker_json" != "$expected_json" ] || \
       [ "$marker_version" != "$version" ] || [ "$marker_arch" != "$arch" ] || \
       [ "${#marker_hash}" -ne 64 ] || [ "${#marker_signature}" -ne 128 ]; then
        fail "signed release bundle marker metadata is invalid"
    fi
    printf '%s\n%s\n%s\n' \
        'RUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN' "$marker_json" \
        'RUSTZEN_BUNDLE_SIGNED_MARKER_END' >"$expected_trailer"
    if ! cmp -s "$actual_trailer" "$expected_trailer"; then
        fail "signed release bundle marker must terminate the artifact"
    fi

    content_blocks=$((content_length / 512))
    if command -v openssl >/dev/null 2>&1; then
        actual_hash="$(dd if="$bundle" bs=512 count="$content_blocks" 2>/dev/null \
            | openssl dgst -sha256 | awk '{print $NF}')"
    elif command -v sha256sum >/dev/null 2>&1; then
        actual_hash="$(dd if="$bundle" bs=512 count="$content_blocks" 2>/dev/null \
            | sha256sum | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        actual_hash="$(dd if="$bundle" bs=512 count="$content_blocks" 2>/dev/null \
            | shasum -a 256 | awk '{print $1}')"
    else
        fail "openssl, sha256sum, or shasum is required to hash the release bundle"
    fi
    if [ "$actual_hash" != "$marker_hash" ]; then
        fail "signed release bundle content hash does not match"
    fi

    payload="$WORK_DIR/signature-payload"
    public_key="$WORK_DIR/verify-key.der"
    signature="$WORK_DIR/signature.bin"
    printf 'rustzen-bundle-v1\ncomponent=bundle\nversion=%s\narch=%s\ncontent_sha256=%s\n' \
        "$version" "$arch" "$marker_hash" >"$payload"
    hex_to_binary "302a300506032b6570032100$verify_key" >"$public_key"
    hex_to_binary "$marker_signature" >"$signature"

    if command -v openssl >/dev/null 2>&1 && \
       openssl pkeyutl -verify -pubin -keyform DER -inkey "$public_key" -rawin \
           -in "$payload" -sigfile "$signature" >/dev/null 2>&1; then
        return
    fi

    javascript_runtime=""
    if command -v bun >/dev/null 2>&1; then
        javascript_runtime="bun"
    elif command -v node >/dev/null 2>&1; then
        javascript_runtime="node"
    fi
    if [ -n "$javascript_runtime" ] && \
       RZ_VERIFY_PAYLOAD="$payload" RZ_VERIFY_KEY="$public_key" RZ_VERIFY_SIGNATURE="$signature" \
       "$javascript_runtime" -e '
           const fs = require("node:fs");
           const crypto = require("node:crypto");
           const key = crypto.createPublicKey({
               key: fs.readFileSync(process.env.RZ_VERIFY_KEY),
               format: "der",
               type: "spki",
           });
           const valid = crypto.verify(
               null,
               fs.readFileSync(process.env.RZ_VERIFY_PAYLOAD),
               key,
               fs.readFileSync(process.env.RZ_VERIFY_SIGNATURE),
           );
           process.exit(valid ? 0 : 1);
       ' >/dev/null 2>&1; then
        return
    fi

    fail "release bundle Ed25519 signature verification failed"
}

if [ "$#" -ne 1 ]; then
    fail "usage: RUSTZEN_DEPLOY_VERIFY_KEY=<trusted-key> setup-layout.sh <signed-uncompressed-tar-bundle>"
fi

BUNDLE_PATH="$1"
if [ ! -f "$BUNDLE_PATH" ] || [ ! -r "$BUNDLE_PATH" ]; then
    fail "bundle is not a readable file: $BUNDLE_PATH"
fi
if [ "$(dd if="$BUNDLE_PATH" bs=1 skip=257 count=5 2>/dev/null)" != "ustar" ]; then
    fail "bundle must be an uncompressed tar archive"
fi

umask 077
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rz-setup-layout.XXXXXX")"
ENTRY_LIST="$WORK_DIR/entries"
NORMALIZED_LIST="$WORK_DIR/entries.normalized"
TAR_ERRORS="$WORK_DIR/tar.errors"
VERBOSE_LIST="$WORK_DIR/entries.verbose"
EXTRACT_ROOT="$WORK_DIR/extracted"

if ! tar -tf "$BUNDLE_PATH" >"$ENTRY_LIST" 2>"$TAR_ERRORS"; then
    fail "bundle is not a readable uncompressed tar archive"
fi
if [ -s "$TAR_ERRORS" ]; then
    fail "bundle tar emitted validation warnings"
fi
if [ ! -s "$ENTRY_LIST" ]; then
    fail "bundle contains no entries"
fi

ROOT_NAME=""
VERSION=""
ARCH=""
: >"$NORMALIZED_LIST"

while IFS= read -r entry || [ -n "$entry" ]; do
    [ -n "$entry" ] || fail "bundle contains an empty path"
    case "$entry" in
        /*)
            fail "bundle contains an absolute path: $entry"
            ;;
        */)
            normalized="${entry%/}"
            case "$normalized" in
                */) fail "bundle contains a non-canonical path: $entry" ;;
            esac
            ;;
        *)
            normalized="$entry"
            ;;
    esac
    case "/$normalized/" in
        */../*|*/./*)
            fail "bundle contains a traversal path: $entry"
            ;;
    esac

    case "$normalized" in
        */*) candidate_root="${normalized%%/*}" ;;
        *) candidate_root="$normalized" ;;
    esac
    if [ -z "$ROOT_NAME" ]; then
        ROOT_NAME="$candidate_root"
        case "$ROOT_NAME" in
            rz-*-x86_64)
                ARCH="x86_64"
                VERSION="${ROOT_NAME#rz-}"
                VERSION="${VERSION%-x86_64}"
                ;;
            rz-*-aarch64)
                ARCH="aarch64"
                VERSION="${ROOT_NAME#rz-}"
                VERSION="${VERSION%-aarch64}"
                ;;
            *)
                fail "bundle root must be rz-<version>-<x86_64|aarch64>: $ROOT_NAME"
                ;;
        esac
        case "$VERSION" in
            ""|.|..|*[!A-Za-z0-9._-]*)
                fail "bundle version is invalid: $VERSION"
                ;;
        esac
    elif [ "$candidate_root" != "$ROOT_NAME" ]; then
        fail "bundle contains multiple roots: $candidate_root"
    fi

    case "$normalized" in
        "$ROOT_NAME"|"$ROOT_NAME/bin"|"$ROOT_NAME/systemd"|"$ROOT_NAME/config"|\
        "$ROOT_NAME/bin/rz"|"$ROOT_NAME/bin/rz-admin"|"$ROOT_NAME/bin/rz-monitor"|\
        "$ROOT_NAME/bin/rz-insights"|"$ROOT_NAME/bin/rz-reports"|\
        "$ROOT_NAME/systemd/rz.target"|"$ROOT_NAME/systemd/rz-recovery.service"|\
        "$ROOT_NAME/systemd/rz-admin.service"|\
        "$ROOT_NAME/systemd/rz-monitor.service"|\
        "$ROOT_NAME/systemd/rz-insights.service"|\
        "$ROOT_NAME/systemd/rz-reports.service"|\
        "$ROOT_NAME/config/rz.env"|"$ROOT_NAME/config/rz-reports.env"|"$ROOT_NAME/setup-layout.sh")
            ;;
        *)
            fail "bundle contains an unexpected path: $entry"
            ;;
    esac
    printf '%s\n' "$normalized" >>"$NORMALIZED_LIST"
done <"$ENTRY_LIST"

DUPLICATE_PATH="$(LC_ALL=C sort "$NORMALIZED_LIST" | uniq -d | sed -n '1p')"
if [ -n "$DUPLICATE_PATH" ]; then
    fail "bundle contains a duplicate path: $DUPLICATE_PATH"
fi

for relative_path in $REQUIRED_FILES; do
    if ! grep -Fqx "$ROOT_NAME/$relative_path" "$NORMALIZED_LIST"; then
        fail "bundle is missing required file: $relative_path"
    fi
done

verify_bundle_signature "$BUNDLE_PATH" "$VERSION" "$ARCH"

if ! tar -tvf "$BUNDLE_PATH" >"$VERBOSE_LIST" 2>"$TAR_ERRORS"; then
    fail "bundle member metadata cannot be read"
fi
if [ -s "$TAR_ERRORS" ]; then
    fail "bundle tar emitted metadata warnings"
fi
while IFS= read -r verbose_entry || [ -n "$verbose_entry" ]; do
    entry_type="$(printf '%s' "$verbose_entry" | cut -c 1)"
    case "$entry_type" in
        -|d) ;;
        *) fail "bundle contains a non-regular member" ;;
    esac
done <"$VERBOSE_LIST"

mkdir -p "$EXTRACT_ROOT"
if ! tar -xf "$BUNDLE_PATH" -C "$EXTRACT_ROOT" 2>"$TAR_ERRORS"; then
    fail "bundle extraction failed"
fi
if [ -s "$TAR_ERRORS" ]; then
    fail "bundle tar emitted extraction warnings"
fi

SOURCE_ROOT="$EXTRACT_ROOT/$ROOT_NAME"
for relative_dir in "" bin systemd config; do
    source_dir="$SOURCE_ROOT"
    if [ -n "$relative_dir" ]; then
        source_dir="$SOURCE_ROOT/$relative_dir"
    fi
    if [ ! -d "$source_dir" ] || [ -L "$source_dir" ]; then
        fail "bundle directory is invalid: ${relative_dir:-$ROOT_NAME}"
    fi
done
for relative_path in $REQUIRED_FILES; do
    source_file="$SOURCE_ROOT/$relative_path"
    if [ ! -f "$source_file" ] || [ -L "$source_file" ] || [ ! -s "$source_file" ]; then
        fail "bundle file is not a non-empty regular file: $relative_path"
    fi
done

mkdir -p "$INSTALL_ROOT"
INSTALL_ROOT="$(CDPATH= cd -- "$INSTALL_ROOT" && pwd -P)"
mkdir -p "$SYSTEMD_DIR"
SYSTEMD_DIR="$(CDPATH= cd -- "$SYSTEMD_DIR" && pwd -P)"

if [ -e "$INSTALL_ROOT/current" ] || [ -L "$INSTALL_ROOT/current" ]; then
    fail "an existing installation must be updated through the Admin release worker"
fi
if [ -L "$INSTALL_ROOT/config/rz.env" ] || \
   { [ -e "$INSTALL_ROOT/config/rz.env" ] && [ ! -f "$INSTALL_ROOT/config/rz.env" ]; }; then
    fail "existing config is not a regular file: $INSTALL_ROOT/config/rz.env"
fi
if [ -L "$INSTALL_ROOT/config/rz-reports.env" ] || \
   { [ -e "$INSTALL_ROOT/config/rz-reports.env" ] && [ ! -f "$INSTALL_ROOT/config/rz-reports.env" ]; }; then
    fail "existing Reports config is not a regular file: $INSTALL_ROOT/config/rz-reports.env"
fi
for unit in $MANAGED_UNITS; do
    destination="$SYSTEMD_DIR/$unit"
    if [ -e "$destination" ] && [ ! -L "$destination" ]; then
        fail "refusing to replace non-symlink systemd unit: $destination"
    fi
done
if [ "${SYSTEMCTL_BIN#*/}" != "$SYSTEMCTL_BIN" ]; then
    [ -x "$SYSTEMCTL_BIN" ] || fail "systemctl command is not executable: $SYSTEMCTL_BIN"
elif ! command -v "$SYSTEMCTL_BIN" >/dev/null 2>&1; then
    fail "systemctl command was not found: $SYSTEMCTL_BIN"
fi

if ! id "$REPORTS_USER" >/dev/null 2>&1; then
    if [ "$(id -u)" -ne 0 ]; then
        fail "reports service account is missing: $REPORTS_USER"
    fi
    command -v useradd >/dev/null 2>&1 || fail "useradd is required to create $REPORTS_USER"
    command -v groupadd >/dev/null 2>&1 || fail "groupadd is required to create rz-reports"
    if ! group_exists rz-reports; then
        groupadd --system rz-reports
    fi
    useradd --system --gid rz-reports --home-dir "$INSTALL_ROOT/data/reports" --shell /usr/sbin/nologin "$REPORTS_USER"
fi
if [ -z "$REPORTS_GROUP" ]; then
    REPORTS_GROUP="$(id -gn "$REPORTS_USER")"
fi
if ! group_exists "$REPORTS_GROUP"; then
    fail "reports service group is missing: $REPORTS_GROUP"
fi

INSTALL_LOCK="$INSTALL_ROOT/.setup-layout.lock"
if ! mkdir "$INSTALL_LOCK" 2>/dev/null; then
    fail "another layout installation is in progress"
fi

mkdir -p \
    "$INSTALL_ROOT/releases" \
    "$INSTALL_ROOT/config" \
    "$INSTALL_ROOT/data/db" \
    "$INSTALL_ROOT/data/releases" \
    "$INSTALL_ROOT/data/reports" \
    "$INSTALL_ROOT/data/uploads" \
    "$INSTALL_ROOT/data/avatars" \
    "$INSTALL_ROOT/logs"
chmod 0755 "$INSTALL_ROOT" "$INSTALL_ROOT/releases"
chmod 0700 "$INSTALL_ROOT/config"
chmod 0711 "$INSTALL_ROOT/data" "$INSTALL_ROOT/logs"
mkdir -p "$INSTALL_ROOT/logs/reports"
mkdir -p "$INSTALL_ROOT/data/reports/db"
chown "$REPORTS_USER:$REPORTS_GROUP" "$INSTALL_ROOT/data/reports" "$INSTALL_ROOT/data/reports/db" "$INSTALL_ROOT/logs/reports"
chmod 0750 "$INSTALL_ROOT/data/reports" "$INSTALL_ROOT/data/reports/db" "$INSTALL_ROOT/logs/reports"

RELEASE_DIR="$INSTALL_ROOT/releases/$VERSION"
if [ -e "$RELEASE_DIR" ] || [ -L "$RELEASE_DIR" ]; then
    fail "release directory already exists: $RELEASE_DIR"
fi

CANDIDATE_DIR="$INSTALL_ROOT/releases/.$VERSION.new.$$"
if [ -e "$CANDIDATE_DIR" ] || [ -L "$CANDIDATE_DIR" ]; then
    fail "release staging path already exists: $CANDIDATE_DIR"
fi
mkdir -p "$CANDIDATE_DIR/bin" "$CANDIDATE_DIR/systemd" "$CANDIDATE_DIR/config"
for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
    install -m 0755 "$SOURCE_ROOT/bin/$binary" "$CANDIDATE_DIR/bin/$binary"
done
for unit in $MANAGED_UNITS; do
    install -m 0644 "$SOURCE_ROOT/systemd/$unit" "$CANDIDATE_DIR/systemd/$unit"
done
install -m 0600 "$SOURCE_ROOT/config/rz.env" "$CANDIDATE_DIR/config/rz.env"
install -m 0600 "$SOURCE_ROOT/config/rz-reports.env" "$CANDIDATE_DIR/config/rz-reports.env"
install -m 0755 "$SOURCE_ROOT/setup-layout.sh" "$CANDIDATE_DIR/setup-layout.sh"
mv "$CANDIDATE_DIR" "$RELEASE_DIR"
CANDIDATE_DIR=""
chmod 0755 "$RELEASE_DIR" "$RELEASE_DIR/bin" "$RELEASE_DIR/systemd"
chmod 0700 "$RELEASE_DIR/config"

if [ ! -e "$INSTALL_ROOT/config/rz.env" ]; then
    install -m 0600 "$SOURCE_ROOT/config/rz.env" "$INSTALL_ROOT/config/rz.env"
fi
if [ ! -e "$INSTALL_ROOT/config/rz-reports.env" ]; then
    install -m 0600 "$SOURCE_ROOT/config/rz-reports.env" "$INSTALL_ROOT/config/rz-reports.env"
fi

STORED_BUNDLE="$INSTALL_ROOT/data/releases/rz-$VERSION-$ARCH.tar"
if [ -e "$STORED_BUNDLE" ]; then
    if [ ! -f "$STORED_BUNDLE" ] || [ -L "$STORED_BUNDLE" ] || ! cmp -s "$BUNDLE_PATH" "$STORED_BUNDLE"; then
        fail "stored release bundle conflicts with the installed version: $STORED_BUNDLE"
    fi
else
    STORED_BUNDLE_TEMP="$INSTALL_ROOT/data/releases/.rz-$VERSION-$ARCH.tar.new.$$"
    install -m 0600 "$BUNDLE_PATH" "$STORED_BUNDLE_TEMP"
    atomic_replace "$STORED_BUNDLE_TEMP" "$STORED_BUNDLE"
fi

CURRENT_TEMP="$INSTALL_ROOT/.current.new.$$"
rm -f "$CURRENT_TEMP"
ln -s "releases/$VERSION" "$CURRENT_TEMP"
atomic_replace "$CURRENT_TEMP" "$INSTALL_ROOT/current"

for unit in $MANAGED_UNITS; do
    destination="$SYSTEMD_DIR/$unit"
    temporary="$SYSTEMD_DIR/.$unit.new.$$"
    rm -f "$temporary"
    ln -s "$INSTALL_ROOT/current/systemd/$unit" "$temporary"
    atomic_replace "$temporary" "$destination"
done

"$SYSTEMCTL_BIN" daemon-reload
"$SYSTEMCTL_BIN" enable rz.target

echo "Installed Rustzen $VERSION ($ARCH) at $RELEASE_DIR"
echo "Set production secrets and RUSTZEN_TIMEZONE in $INSTALL_ROOT/config/rz.env and $INSTALL_ROOT/config/rz-reports.env. Keep RUSTZEN_IPC_TOKEN identical in both files, then run: systemctl enable --now rz.target"
