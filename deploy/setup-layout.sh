#!/usr/bin/env sh
set -eu

INSTALL_ROOT="${INSTALL_ROOT:-/opt/rz}"
SYSTEMD_DIR="${SYSTEMD_DIR:-/etc/systemd/system}"
CLI_BIN_DIR="${CLI_BIN_DIR:-/usr/local/bin}"
SYSTEMCTL_BIN="${SYSTEMCTL_BIN:-systemctl}"
REPORTS_USER="${RUSTZEN_REPORTS_USER:-rz-reports}"
REPORTS_GROUP="${RUSTZEN_REPORTS_GROUP:-}"
ADMIN_USER="${RUSTZEN_ADMIN_USER:-rz-admin}"
ADMIN_GROUP="${RUSTZEN_ADMIN_GROUP:-rz-admin}"
LOG_CONTROL_GROUP="${RUSTZEN_LOG_CONTROL_GROUP:-rz-log-control}"
MONITOR_USER="${RUSTZEN_MONITOR_USER:-rz-monitor}"
INSIGHTS_USER="${RUSTZEN_INSIGHTS_USER:-rz-insights}"
MANAGED_UNITS="rz-full.service rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service rz-update.service rz-update.path"
REQUIRED_FILES="
bin/rz
bin/rz-admin
bin/rz-monitor
bin/rz-insights
bin/rz-reports
systemd/rz-full.service
systemd/rz-recovery.service
systemd/rz-admin.service
systemd/rz-monitor.service
systemd/rz-insights.service
systemd/rz-reports.service
systemd/rz-update.service
systemd/rz-update.path
identity/controller.json
config/rz.env
config/rz-reports.env
setup-layout.sh
"

WORK_DIR=""
INSTALL_LOCK=""
CANDIDATE_DIR=""
INSTALL_COMPLETED=false
CREATED_RELEASE_DIR=""
CREATED_STORED_BUNDLE=""
CREATED_MAIN_CONFIG=""
CREATED_REPORTS_CONFIG=""
CREATED_CURRENT_LINK=""
CREATED_UNITS=""
CREATED_BOOTSTRAP_INPUT=""
CREATED_INITIAL_CREDENTIAL=""
CREATED_BOOTSTRAP_TEMP=""
CREATED_INITIAL_CREDENTIAL_TEMP=""
CREATED_CLI_LINK=""

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

ensure_service_account() {
    service_user="$1"
    service_home="$2"
    if id "$service_user" >/dev/null 2>&1; then
        return
    fi
    [ "$(id -u)" -eq 0 ] || fail "service account is missing: $service_user"
    command -v useradd >/dev/null 2>&1 || fail "useradd is required to create $service_user"
    command -v groupadd >/dev/null 2>&1 || fail "groupadd is required to create $service_user"
    group_exists "$service_user" || groupadd --system "$service_user"
    useradd --system --gid "$service_user" --home-dir "$service_home" --shell /usr/sbin/nologin "$service_user"
}

cleanup() {
    if [ "$INSTALL_COMPLETED" != true ]; then
        [ -z "$CREATED_CURRENT_LINK" ] || rm -f "$CREATED_CURRENT_LINK"
        for unit in $CREATED_UNITS; do rm -f "$SYSTEMD_DIR/$unit"; done
        [ -z "$CREATED_STORED_BUNDLE" ] || rm -f "$CREATED_STORED_BUNDLE"
        [ -z "$CREATED_RELEASE_DIR" ] || rm -rf "$CREATED_RELEASE_DIR"
        [ -z "$CREATED_MAIN_CONFIG" ] || rm -f "$CREATED_MAIN_CONFIG"
        [ -z "$CREATED_REPORTS_CONFIG" ] || rm -f "$CREATED_REPORTS_CONFIG"
        [ -z "$CREATED_BOOTSTRAP_INPUT" ] || rm -f "$CREATED_BOOTSTRAP_INPUT"
        [ -z "$CREATED_INITIAL_CREDENTIAL" ] || rm -f "$CREATED_INITIAL_CREDENTIAL"
        [ -z "$CREATED_BOOTSTRAP_TEMP" ] || rm -f "$CREATED_BOOTSTRAP_TEMP"
        [ -z "$CREATED_INITIAL_CREDENTIAL_TEMP" ] || rm -f "$CREATED_INITIAL_CREDENTIAL_TEMP"
        [ -z "$CREATED_CLI_LINK" ] || rm -f "$CREATED_CLI_LINK"
    fi
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

generate_secret() {
    if command -v openssl >/dev/null 2>&1; then
        openssl rand -hex 32
    else
        od -An -N32 -tx1 /dev/urandom | tr -d '[:space:]'
    fi
}

replace_placeholder() {
    config_path="$1"
    config_key="$2"
    config_value="$3"
    config_temp="$config_path.new.$$"
    awk -v key="$config_key" -v value="$config_value" '
        index($0, key "=") == 1 {
            if ($0 == key "=replace-me") print key "=" value; else print
            found = 1
            next
        }
        { print }
        END { if (!found) exit 1 }
    ' "$config_path" >"$config_temp" || fail "required configuration key is missing: $config_key"
    chmod 0600 "$config_temp"
    mv "$config_temp" "$config_path"
}

config_value() {
    awk -v key="$2" 'index($0, key "=") == 1 { print substr($0, length(key) + 2); found = 1; exit } END { if (!found) exit 1 }' "$1"
}

shared_config_value() {
    main_value="$(config_value "$1" "$3")" || fail "required configuration key is missing: $3"
    reports_value="$(config_value "$2" "$3")" || fail "required configuration key is missing: $3"
    if [ "$main_value" = replace-me ] && [ "$reports_value" = replace-me ]; then
        generate_secret
    elif [ "$main_value" = replace-me ]; then
        printf '%s\n' "$reports_value"
    elif [ "$reports_value" = replace-me ]; then
        printf '%s\n' "$main_value"
    elif [ "$main_value" = "$reports_value" ]; then
        printf '%s\n' "$main_value"
    else
        fail "shared configuration key differs between service configs: $3"
    fi
}

generate_runtime_config() {
    main_config="$INSTALL_ROOT/config/rz.env"
    reports_config="$INSTALL_ROOT/config/rz-reports.env"
    jwt_secret="$(generate_secret)"
    ipc_token="$(shared_config_value "$main_config" "$reports_config" RUSTZEN_IPC_TOKEN)"
    agent_token="$(generate_secret)"
    node_id="$(generate_secret)"
    notification_key="$(generate_secret)"
    reports_notification_key="$(shared_config_value "$main_config" "$reports_config" RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY)"
    reports_credential_key="$(generate_secret)"

    replace_placeholder "$main_config" RUSTZEN_JWT_SECRET "$jwt_secret"
    replace_placeholder "$main_config" RUSTZEN_IPC_TOKEN "$ipc_token"
    replace_placeholder "$main_config" RUSTZEN_MONITOR_AGENT_TOKEN "$agent_token"
    replace_placeholder "$main_config" RUSTZEN_MONITOR_NODE_ID "$node_id"
    replace_placeholder "$main_config" RUSTZEN_NOTIFICATION_EVENT_KEY "$notification_key"
    replace_placeholder "$main_config" RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY "$reports_notification_key"
    replace_placeholder "$reports_config" RUSTZEN_IPC_TOKEN "$ipc_token"
    replace_placeholder "$reports_config" RUSTZEN_REPORTS_CREDENTIAL_KEY "$reports_credential_key"
    replace_placeholder "$reports_config" RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY "$reports_notification_key"

    if grep -Fq '=replace-me' "$main_config" "$reports_config"; then
        fail "runtime configuration still contains placeholders"
    fi
}

generate_owner_bootstrap() {
    bootstrap_dir="$INSTALL_ROOT/data/db/admin"
    bootstrap_input="$INSTALL_ROOT/data/db/admin/bootstrap-owner-password"
    initial_credential="$INSTALL_ROOT/data/initial-owner-password"
    if [ -e "$bootstrap_input" ] || [ -L "$bootstrap_input" ]; then
        [ -f "$bootstrap_input" ] && [ ! -L "$bootstrap_input" ] && [ -f "$initial_credential" ] && [ ! -L "$initial_credential" ] || fail "incomplete bootstrap owner credential state"
        return
    fi
    [ ! -e "$initial_credential" ] && [ ! -L "$initial_credential" ] || fail "bootstrap owner credential state is invalid"
    chown "$(id -u)" "$bootstrap_dir"
    chmod 0750 "$bootstrap_dir"
    RUSTZEN_BOOTSTRAP_OWNER_PASSWORD="$(generate_secret)"
    CREATED_BOOTSTRAP_TEMP="$(mktemp "$bootstrap_dir/.bootstrap-owner-password.XXXXXX")"
    chmod 0600 "$CREATED_BOOTSTRAP_TEMP"
    printf '%s\n' "$RUSTZEN_BOOTSTRAP_OWNER_PASSWORD" >"$CREATED_BOOTSTRAP_TEMP"
    CREATED_INITIAL_CREDENTIAL_TEMP="$(mktemp "$INSTALL_ROOT/data/.initial-owner-password.XXXXXX")"
    chmod 0600 "$CREATED_INITIAL_CREDENTIAL_TEMP"
    printf '%s\n' "$RUSTZEN_BOOTSTRAP_OWNER_PASSWORD" >"$CREATED_INITIAL_CREDENTIAL_TEMP"
    sync
    atomic_replace "$CREATED_BOOTSTRAP_TEMP" "$bootstrap_input"
    CREATED_BOOTSTRAP_TEMP=""
    CREATED_BOOTSTRAP_INPUT="$bootstrap_input"
    atomic_replace "$CREATED_INITIAL_CREDENTIAL_TEMP" "$initial_credential"
    CREATED_INITIAL_CREDENTIAL_TEMP=""
    CREATED_INITIAL_CREDENTIAL="$initial_credential"
    unset RUSTZEN_BOOTSTRAP_OWNER_PASSWORD
    chown "$ADMIN_USER" "$bootstrap_input"
    chmod 0600 "$bootstrap_input" "$initial_credential"
    chown "$ADMIN_USER" "$bootstrap_dir"
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
    # build-config replaces this token in the separately distributed installer.
    # Do not trust a key supplied by the bundle being verified.
    verify_key="__RUSTZEN_DEPLOY_VERIFY_KEY__"
    case "$verify_key" in
        ""|*[!0-9a-f]*)
            fail "packaged release verification key must be a 64-character Ed25519 public key"
            ;;
    esac
    if [ "${#verify_key}" -ne 64 ]; then
        fail "packaged release verification key must be a 64-character Ed25519 public key"
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
    marker_frontend_hash="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"frontendSha256":"\([0-9a-f]*\)".*$/\1/p')"
    marker_backend_hash="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"backendSha256":"\([0-9a-f]*\)".*$/\1/p')"
    marker_signature="$(printf '%s\n' "$marker_json" | sed -n 's/^.*"signature":"\([0-9a-f]*\)".*$/\1/p')"
    expected_json="$(printf '{"schemaVersion":2,"component":"release","version":"%s","arch":"%s","contentSha256":"%s","frontendSha256":"%s","backendSha256":"%s","signature":"%s"}' \
        "$marker_version" "$marker_arch" "$marker_hash" "$marker_frontend_hash" "$marker_backend_hash" "$marker_signature")"
    if [ "$marker_json" != "$expected_json" ] || \
       [ "$marker_version" != "$version" ] || [ "$marker_arch" != "$arch" ] || \
       [ "${#marker_hash}" -ne 64 ] || [ "${#marker_frontend_hash}" -ne 64 ] || \
       [ "${#marker_backend_hash}" -ne 64 ] || [ "${#marker_signature}" -ne 128 ]; then
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
    printf 'rustzen-release-v2\ncomponent=release\nversion=%s\narch=%s\ncontent_sha256=%s\nfrontend_sha256=%s\nbackend_sha256=%s\n' \
        "$version" "$arch" "$marker_hash" "$marker_frontend_hash" "$marker_backend_hash" >"$payload"
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
    fail "usage: rz-install <signed-uncompressed-tar-bundle>"
fi

SOURCE_BUNDLE_PATH="$1"
if [ ! -f "$SOURCE_BUNDLE_PATH" ] || [ ! -r "$SOURCE_BUNDLE_PATH" ]; then
    fail "bundle is not a readable file: $SOURCE_BUNDLE_PATH"
fi

umask 077
WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/rz-setup-layout.XXXXXX")"
BUNDLE_PATH="$WORK_DIR/bundle.snapshot"
if ! install -m 0600 "$SOURCE_BUNDLE_PATH" "$BUNDLE_PATH"; then
    fail "bundle could not be copied into a private verification snapshot"
fi
if [ "$(dd if="$BUNDLE_PATH" bs=1 skip=257 count=5 2>/dev/null)" != "ustar" ]; then
    fail "bundle must be an uncompressed tar archive"
fi
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
        "$ROOT_NAME"|"$ROOT_NAME/bin"|"$ROOT_NAME/systemd"|"$ROOT_NAME/config"|"$ROOT_NAME/identity"|\
        "$ROOT_NAME/bin/rz"|"$ROOT_NAME/bin/rz-admin"|"$ROOT_NAME/bin/rz-monitor"|\
        "$ROOT_NAME/bin/rz-insights"|"$ROOT_NAME/bin/rz-reports"|\
        "$ROOT_NAME/systemd/rz-full.service"|"$ROOT_NAME/systemd/rz-recovery.service"|\
        "$ROOT_NAME/systemd/rz-admin.service"|\
        "$ROOT_NAME/systemd/rz-monitor.service"|\
        "$ROOT_NAME/systemd/rz-insights.service"|\
        "$ROOT_NAME/systemd/rz-reports.service"|\
        "$ROOT_NAME/systemd/rz-update.service"|"$ROOT_NAME/systemd/rz-update.path"|\
        "$ROOT_NAME/identity/controller.json"|\
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

for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
    binary_path="$SOURCE_ROOT/bin/$binary"
    binary_magic="$(dd if="$binary_path" bs=1 count=4 2>/dev/null | od -An -tx1 | tr -d ' \n')"
    if [ "$binary_magic" != "7f454c46" ]; then
        fail "release bundle member is not an ELF executable: $binary"
    fi
    binary_machine="$(dd if="$binary_path" bs=1 skip=18 count=2 2>/dev/null | od -An -tx1 | tr -d ' \n')"
    case "$ARCH:$binary_machine" in
        x86_64:3e00|aarch64:b700) ;;
        *) fail "release bundle member architecture mismatch: $binary" ;;
    esac
    binary_marker_expected="$WORK_DIR/$binary.identity-marker.expected"
    binary_marker_actual="$WORK_DIR/$binary.identity-marker.actual"
    binary_marker_offsets="$WORK_DIR/$binary.identity-marker.offsets"
    printf 'RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=%s\nversion=%s\n' \
        "$binary" "$VERSION" >"$binary_marker_expected"
    binary_marker_length="$(wc -c <"$binary_marker_expected" | tr -d '[:space:]')"
    LC_ALL=C grep -aob 'RUSTZEN_RELEASE_MARKER' "$binary_path" \
        >"$binary_marker_offsets" || true
    binary_marker_matches=false
    while IFS=: read -r binary_marker_offset ignored_marker || \
        [ -n "$binary_marker_offset" ]; do
        case "$binary_marker_offset" in
            ""|*[!0-9]*) continue ;;
        esac
        dd if="$binary_path" bs=1 skip="$binary_marker_offset" \
            count="$binary_marker_length" of="$binary_marker_actual" 2>/dev/null || true
        if cmp -s "$binary_marker_expected" "$binary_marker_actual"; then
            binary_marker_matches=true
            break
        fi
    done <"$binary_marker_offsets"
    if [ "$binary_marker_matches" != true ]; then
        fail "release bundle member identity marker mismatch: $binary"
    fi
done

admin_marker_frontend="$(LC_ALL=C grep -ao 'frontend_sha256=[0-9a-f]\{64\}' \
    "$SOURCE_ROOT/bin/rz-admin" | sed -n 's/^frontend_sha256=//p')"
if [ "$admin_marker_frontend" != "$marker_frontend_hash" ]; then
    fail "signed frontend digest does not match the embedded Admin Web digest"
fi

BACKEND_INVENTORY="$WORK_DIR/backend-inventory"
: >"$BACKEND_INVENTORY"
for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
    binary_path="$SOURCE_ROOT/bin/$binary"
    binary_size="$(wc -c <"$binary_path" | tr -d '[:space:]')"
    if command -v openssl >/dev/null 2>&1; then
        binary_hash="$(openssl dgst -sha256 "$binary_path" | awk '{print $NF}')"
    elif command -v sha256sum >/dev/null 2>&1; then
        binary_hash="$(sha256sum "$binary_path" | awk '{print $1}')"
    else
        binary_hash="$(shasum -a 256 "$binary_path" | awk '{print $1}')"
    fi
    printf 'binary=%s\nsize=%s\nsha256=%s\n' "$binary" "$binary_size" "$binary_hash" \
        >>"$BACKEND_INVENTORY"
done
if command -v openssl >/dev/null 2>&1; then
    installed_backend_hash="$(openssl dgst -sha256 "$BACKEND_INVENTORY" | awk '{print $NF}')"
elif command -v sha256sum >/dev/null 2>&1; then
    installed_backend_hash="$(sha256sum "$BACKEND_INVENTORY" | awk '{print $1}')"
else
    installed_backend_hash="$(shasum -a 256 "$BACKEND_INVENTORY" | awk '{print $1}')"
fi
if [ "$installed_backend_hash" != "$marker_backend_hash" ]; then
    fail "signed backend digest does not match the release executables"
fi

mkdir -p "$INSTALL_ROOT"
INSTALL_ROOT="$(CDPATH= cd -- "$INSTALL_ROOT" && pwd -P)"
mkdir -p "$SYSTEMD_DIR"
SYSTEMD_DIR="$(CDPATH= cd -- "$SYSTEMD_DIR" && pwd -P)"
mkdir -p "$CLI_BIN_DIR"
CLI_BIN_DIR="$(CDPATH= cd -- "$CLI_BIN_DIR" && pwd -P)"
RZ_COMMAND_PATH="$CLI_BIN_DIR/rz"
RZ_COMMAND_TARGET="$INSTALL_ROOT/current/bin/rz"

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
    expected_target="$INSTALL_ROOT/current/systemd/$unit"
    if [ -L "$destination" ]; then
        if [ "$(readlink "$destination")" != "$expected_target" ]; then
            fail "refusing to replace existing systemd unit link: $destination"
        fi
    elif [ -e "$destination" ]; then
        fail "refusing to replace non-symlink systemd unit: $destination"
    fi
done
if [ -e "$RZ_COMMAND_PATH" ] || [ -L "$RZ_COMMAND_PATH" ]; then
    if [ ! -L "$RZ_COMMAND_PATH" ] || [ "$(readlink "$RZ_COMMAND_PATH")" != "$RZ_COMMAND_TARGET" ]; then
        fail "refusing to replace existing command path: $RZ_COMMAND_PATH"
    fi
fi
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
ensure_service_account "$ADMIN_USER" "$INSTALL_ROOT/data/db/admin"
ensure_service_account "$MONITOR_USER" "$INSTALL_ROOT/data/db/monitor"
ensure_service_account "$INSIGHTS_USER" "$INSTALL_ROOT/data/db/insights"
group_exists "$ADMIN_GROUP" || fail "admin service group is missing: $ADMIN_GROUP"
if ! group_exists "$LOG_CONTROL_GROUP"; then
    [ "$(id -u)" -eq 0 ] || fail "log control group is missing: $LOG_CONTROL_GROUP"
    command -v groupadd >/dev/null 2>&1 || fail "groupadd is required to create $LOG_CONTROL_GROUP"
    groupadd --system "$LOG_CONTROL_GROUP"
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
    "$INSTALL_ROOT/data/db/admin" \
    "$INSTALL_ROOT/data/db/monitor" \
    "$INSTALL_ROOT/data/db/insights" \
    "$INSTALL_ROOT/data/releases" \
    "$INSTALL_ROOT/data/reports" \
    "$INSTALL_ROOT/data/uploads" \
    "$INSTALL_ROOT/data/avatars" \
    "$INSTALL_ROOT/data/update-requests" \
    "$INSTALL_ROOT/logs"
chmod 0755 "$INSTALL_ROOT" "$INSTALL_ROOT/releases"
chmod 0700 "$INSTALL_ROOT/config"
chmod 0711 "$INSTALL_ROOT/data" "$INSTALL_ROOT/data/db" "$INSTALL_ROOT/logs"
chown "$ADMIN_USER:$ADMIN_GROUP" "$INSTALL_ROOT/data/releases"
chmod 0750 "$INSTALL_ROOT/data/releases"
mkdir -p "$INSTALL_ROOT/logs/admin" "$INSTALL_ROOT/logs/monitor" "$INSTALL_ROOT/logs/insights" "$INSTALL_ROOT/logs/reports"
mkdir -p "$INSTALL_ROOT/data/reports/db"
chown "$REPORTS_USER:$REPORTS_GROUP" "$INSTALL_ROOT/data/reports" "$INSTALL_ROOT/data/reports/db"
chown "$ADMIN_USER" "$INSTALL_ROOT/data/uploads" "$INSTALL_ROOT/data/avatars" "$INSTALL_ROOT/data/update-requests"
chown "$MONITOR_USER" "$INSTALL_ROOT/data/db/monitor"
chown "$INSIGHTS_USER" "$INSTALL_ROOT/data/db/insights"
chown "$ADMIN_USER:$LOG_CONTROL_GROUP" "$INSTALL_ROOT/logs/admin"
chown "$MONITOR_USER:$LOG_CONTROL_GROUP" "$INSTALL_ROOT/logs/monitor"
chown "$INSIGHTS_USER:$LOG_CONTROL_GROUP" "$INSTALL_ROOT/logs/insights"
chown "$REPORTS_USER:$LOG_CONTROL_GROUP" "$INSTALL_ROOT/logs/reports"
chmod 0750 "$INSTALL_ROOT/data/db/admin" "$INSTALL_ROOT/data/db/monitor" "$INSTALL_ROOT/data/db/insights" "$INSTALL_ROOT/data/uploads" "$INSTALL_ROOT/data/avatars" "$INSTALL_ROOT/data/update-requests"
chmod 0750 "$INSTALL_ROOT/data/reports" "$INSTALL_ROOT/data/reports/db"
chmod 2770 "$INSTALL_ROOT/logs/admin" "$INSTALL_ROOT/logs/monitor" "$INSTALL_ROOT/logs/insights" "$INSTALL_ROOT/logs/reports"

RELEASE_DIR="$INSTALL_ROOT/releases/$VERSION"
if [ -e "$RELEASE_DIR" ] || [ -L "$RELEASE_DIR" ]; then
    fail "release directory already exists: $RELEASE_DIR"
fi

CANDIDATE_DIR="$INSTALL_ROOT/releases/.$VERSION.new.$$"
if [ -e "$CANDIDATE_DIR" ] || [ -L "$CANDIDATE_DIR" ]; then
    fail "release staging path already exists: $CANDIDATE_DIR"
fi
mkdir -p "$CANDIDATE_DIR/bin" "$CANDIDATE_DIR/systemd" "$CANDIDATE_DIR/config" "$CANDIDATE_DIR/identity"
for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
    install -m 0755 "$SOURCE_ROOT/bin/$binary" "$CANDIDATE_DIR/bin/$binary"
done
for unit in $MANAGED_UNITS; do
    install -m 0644 "$SOURCE_ROOT/systemd/$unit" "$CANDIDATE_DIR/systemd/$unit"
done
install -m 0600 "$SOURCE_ROOT/config/rz.env" "$CANDIDATE_DIR/config/rz.env"
install -m 0600 "$SOURCE_ROOT/config/rz-reports.env" "$CANDIDATE_DIR/config/rz-reports.env"
install -m 0644 "$SOURCE_ROOT/identity/controller.json" "$CANDIDATE_DIR/identity/controller.json"
install -m 0755 "$SOURCE_ROOT/setup-layout.sh" "$CANDIDATE_DIR/setup-layout.sh"
mv "$CANDIDATE_DIR" "$RELEASE_DIR"
CANDIDATE_DIR=""
CREATED_RELEASE_DIR="$RELEASE_DIR"
chmod 0755 "$RELEASE_DIR" "$RELEASE_DIR/bin" "$RELEASE_DIR/systemd"
chmod 0700 "$RELEASE_DIR/config"
chmod 0755 "$RELEASE_DIR/identity"
chown "$ADMIN_USER:$ADMIN_GROUP" \
    "$RELEASE_DIR/config" \
    "$RELEASE_DIR/config/rz.env" \
    "$RELEASE_DIR/config/rz-reports.env"

if [ ! -e "$INSTALL_ROOT/config/rz.env" ]; then
    install -m 0600 "$SOURCE_ROOT/config/rz.env" "$INSTALL_ROOT/config/rz.env"
    CREATED_MAIN_CONFIG="$INSTALL_ROOT/config/rz.env"
fi
if [ ! -e "$INSTALL_ROOT/config/rz-reports.env" ]; then
    install -m 0600 "$SOURCE_ROOT/config/rz-reports.env" "$INSTALL_ROOT/config/rz-reports.env"
    CREATED_REPORTS_CONFIG="$INSTALL_ROOT/config/rz-reports.env"
fi
for setting in \
    "RUSTZEN_ADMIN_SQLITE_PATH=$INSTALL_ROOT/data/db/admin/admin.db" \
    "RUSTZEN_MONITOR_SQLITE_PATH=$INSTALL_ROOT/data/db/monitor/monitor.db" \
    "RUSTZEN_INSIGHTS_SQLITE_PATH=$INSTALL_ROOT/data/db/insights/insights.db"; do
    key=${setting%%=*}
    grep -Eq "^${key}=" "$INSTALL_ROOT/config/rz.env" || printf '%s\n' "$setting" >>"$INSTALL_ROOT/config/rz.env"
done
generate_runtime_config
generate_owner_bootstrap

STORED_BUNDLE="$INSTALL_ROOT/data/releases/rz-$VERSION-$ARCH.tar"
if [ -e "$STORED_BUNDLE" ]; then
    if [ ! -f "$STORED_BUNDLE" ] || [ -L "$STORED_BUNDLE" ] || ! cmp -s "$BUNDLE_PATH" "$STORED_BUNDLE"; then
        fail "stored release bundle conflicts with the installed version: $STORED_BUNDLE"
    fi
else
    STORED_BUNDLE_TEMP="$INSTALL_ROOT/data/releases/.rz-$VERSION-$ARCH.tar.new.$$"
    install -m 0640 "$BUNDLE_PATH" "$STORED_BUNDLE_TEMP"
    atomic_replace "$STORED_BUNDLE_TEMP" "$STORED_BUNDLE"
    CREATED_STORED_BUNDLE="$STORED_BUNDLE"
fi
    chown "$(id -u):$ADMIN_GROUP" "$STORED_BUNDLE"
chmod 0640 "$STORED_BUNDLE"

CURRENT_TEMP="$INSTALL_ROOT/.current.new.$$"
rm -f "$CURRENT_TEMP"
ln -s "releases/$VERSION" "$CURRENT_TEMP"
atomic_replace "$CURRENT_TEMP" "$INSTALL_ROOT/current"
CREATED_CURRENT_LINK="$INSTALL_ROOT/current"

if [ ! -L "$RZ_COMMAND_PATH" ]; then
    CLI_LINK_TEMP="$CLI_BIN_DIR/.rz.new.$$"
    rm -f "$CLI_LINK_TEMP"
    ln -s "$RZ_COMMAND_TARGET" "$CLI_LINK_TEMP"
    atomic_replace "$CLI_LINK_TEMP" "$RZ_COMMAND_PATH"
    CREATED_CLI_LINK="$RZ_COMMAND_PATH"
fi

for unit in $MANAGED_UNITS; do
    destination="$SYSTEMD_DIR/$unit"
    if [ ! -L "$destination" ]; then
        temporary="$SYSTEMD_DIR/.$unit.new.$$"
        rm -f "$temporary"
        ln -s "$INSTALL_ROOT/current/systemd/$unit" "$temporary"
        atomic_replace "$temporary" "$destination"
        CREATED_UNITS="$CREATED_UNITS $unit"
    fi
done

"$SYSTEMCTL_BIN" daemon-reload
"$SYSTEMCTL_BIN" enable rz-full.service rz-update.path
INSTALL_COMPLETED=true

echo "Installed Rustzen $VERSION ($ARCH) at $RELEASE_DIR"
echo "Runtime secrets were generated locally. Read the one-time owner credential from $INSTALL_ROOT/data/initial-owner-password as root, then run: rz start"
