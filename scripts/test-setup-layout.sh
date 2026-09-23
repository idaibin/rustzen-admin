#!/usr/bin/env sh
set -eu

PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
export RUSTZEN_REPORTS_USER="${RUSTZEN_REPORTS_USER:-$(id -un)}"
export RUSTZEN_REPORTS_GROUP="${RUSTZEN_REPORTS_GROUP:-$(id -gn)}"
export RUSTZEN_ADMIN_USER="${RUSTZEN_ADMIN_USER:-$(id -un)}"
export RUSTZEN_ADMIN_GROUP="${RUSTZEN_ADMIN_GROUP:-$(id -gn)}"
export RUSTZEN_LOG_CONTROL_GROUP="${RUSTZEN_LOG_CONTROL_GROUP:-$(id -gn)}"
export RUSTZEN_MONITOR_USER="${RUSTZEN_MONITOR_USER:-$(id -un)}"
export RUSTZEN_INSIGHTS_USER="${RUSTZEN_INSIGHTS_USER:-$(id -un)}"
INSTALLER_TEMPLATE="$PROJECT_ROOT/deploy/setup-layout.sh"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/rz-setup-layout-test.XXXXXX")"
TEST_ROOT="$(CDPATH= cd -- "$TEST_ROOT" && pwd -P)"
TEST_COUNT=0

cleanup() {
    rm -rf "$TEST_ROOT"
}

trap cleanup 0 1 2 15

fail() {
    echo "test-setup-layout: $*" >&2
    exit 1
}

assert_exists() {
    [ -e "$1" ] || fail "expected path to exist: $1"
}

assert_symlink() {
    [ -L "$1" ] || fail "expected symlink: $1"
}

assert_equals() {
    expected="$1"
    actual="$2"
    label="$3"
    [ "$actual" = "$expected" ] || fail "$label: expected '$expected', got '$actual'"
}

assert_file_contains() {
    pattern="$1"
    path="$2"
    label="$3"
    grep -Fq "$pattern" "$path" || fail "$label: expected '$pattern' in $path"
}

config_value() {
    awk -v key="$2" 'index($0, key "=") == 1 { print substr($0, length(key) + 2); exit }' "$1"
}

file_mode() {
    case "$(uname -s)" in
        Darwin|FreeBSD|NetBSD|OpenBSD) stat -f '%Lp' "$1" ;;
        *) stat -c '%a' "$1" ;;
    esac
}

file_special_mode() {
    case "$(uname -s)" in
        Darwin|FreeBSD|NetBSD|OpenBSD) stat -f '%Mp%Lp' "$1" ;;
        *) stat -c '%a' "$1" ;;
    esac
}

file_group() {
    case "$(uname -s)" in
        Darwin|FreeBSD|NetBSD|OpenBSD) stat -f '%Sg' "$1" ;;
        *) stat -c '%G' "$1" ;;
    esac
}

file_owner() {
    case "$(uname -s)" in
        Darwin|FreeBSD|NetBSD|OpenBSD) stat -f '%Su' "$1" ;;
        *) stat -c '%U' "$1" ;;
    esac
}

make_fake_systemctl() {
    destination="$1"
    printf '%s\n' \
        '#!/usr/bin/env sh' \
        'set -eu' \
        ': "${SYSTEMCTL_LOG:?}"' \
        'printf "%s\n" "$*" >> "$SYSTEMCTL_LOG"' \
        'if [ "${SYSTEMCTL_FAIL_ON:-}" = "$*" ]; then exit 1; fi' \
        >"$destination"
    chmod 0755 "$destination"
}

make_racing_install() {
    destination="$1"
    cat >"$destination" <<'EOF'
#!/usr/bin/env sh
set -eu

destination_path=""
for argument in "$@"; do
    destination_path="$argument"
done
"${REAL_INSTALL_BIN:?}" "$@"
case "$destination_path" in
    */bundle.snapshot)
        if [ -f "${RACE_REPLACEMENT:?}" ]; then
            mv "$RACE_REPLACEMENT" "${RACE_SOURCE:?}"
        fi
        ;;
esac
EOF
    chmod 0755 "$destination"
}

make_test_signing_key() {
    destination="$1"
    KEY_DESTINATION="$destination" bun -e '
        const crypto = require("node:crypto");
        const fs = require("node:fs");
        const { privateKey, publicKey } = crypto.generateKeyPairSync("ed25519");
        fs.writeFileSync(
            process.env.KEY_DESTINATION,
            privateKey.export({ format: "pem", type: "pkcs8" }),
            { mode: 0o600 },
        );
        const der = publicKey.export({ format: "der", type: "spki" });
        process.stdout.write(der.subarray(12).toString("hex"));
    '
}

make_packaged_installer() {
    destination="$1"
    verify_key="$2"
    sed "s/__RUSTZEN_DEPLOY_VERIFY_KEY__/$verify_key/g" "$INSTALLER_TEMPLATE" >"$destination"
    chmod 0755 "$destination"
}

make_bundle() {
    version="$1"
    arch="$2"
    destination="$3"
    omitted_path="${4:-}"
    signing_key="${5:-$SIGNING_KEY}"
    signature_mode="${6:-signed}"
    source_dir="$TEST_ROOT/source-$version-$arch-$(basename "$destination")"
    root_name="rz-$version-$arch"
    release_root="$source_dir/$root_name"

    mkdir -p "$release_root/bin" "$release_root/systemd" "$release_root/config" "$release_root/identity"
    for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
        dd if=/dev/zero of="$release_root/bin/$binary" bs=64 count=1 2>/dev/null
        printf '\177ELF' | dd of="$release_root/bin/$binary" bs=1 seek=0 conv=notrunc 2>/dev/null
        case "$arch" in
            x86_64) printf '\076\000' ;;
            aarch64) printf '\267\000' ;;
        esac | dd of="$release_root/bin/$binary" bs=1 seek=18 conv=notrunc 2>/dev/null
        printf 'RUSTZEN_RELEASE_MARKER\nartifact=rz-bundle-member\nbinary=%s\nversion=%s\n' \
            "$binary" "$version" >>"$release_root/bin/$binary"
        if [ "$binary" = "rz-admin" ]; then
            printf 'frontend_sha256=%s\n' \
                aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
                >>"$release_root/bin/$binary"
        fi
        chmod 0755 "$release_root/bin/$binary"
    done
    for unit in rz-full.service rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service rz-update.service rz-update.path; do
        cp "$PROJECT_ROOT/deploy/$unit" "$release_root/systemd/$unit"
    done
    printf 'RUSTZEN_ENV=production\nRUSTZEN_JWT_SECRET=replace-me\nRUSTZEN_IPC_TOKEN=replace-me\nRUSTZEN_MONITOR_AGENT_TOKEN=replace-me\nRUSTZEN_MONITOR_NODE_ID=replace-me\nRUSTZEN_NOTIFICATION_EVENT_KEY=replace-me\nRUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY=replace-me\nRUSTZEN_DEPLOY_VERIFY_KEY=%s\n' \
        "$VERIFY_KEY" \
        >"$release_root/config/rz.env"
    printf 'RUSTZEN_ENV=production\nRUSTZEN_IPC_TOKEN=replace-me\nRUSTZEN_REPORTS_CREDENTIAL_KEY=replace-me\nRUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY=replace-me\n' \
        >"$release_root/config/rz-reports.env"
    printf '{"schemaVersion":1,"artifactClass":"full-controller","version":"%s","arch":"%s","monitorBinarySha256":"%s","agentProtocolContractId":"%s"}\n' \
        "$version" "$arch" \
        "$(openssl dgst -sha256 "$release_root/bin/rz-monitor" | awk '{print $NF}')" \
        aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa \
        >"$release_root/identity/controller.json"
    cp "$INSTALLER_TEMPLATE" "$release_root/setup-layout.sh"
    chmod 0755 "$release_root/setup-layout.sh"

    if [ -n "$omitted_path" ]; then
        rm -f "$release_root/$omitted_path"
    fi

    COPYFILE_DISABLE=1 tar -cf "$destination" -C "$source_dir" "$root_name"
    if [ "$signature_mode" = "signed" ]; then
        RUSTZEN_DEPLOY_SIGN_KEY_FILE="$signing_key" \
            bun "$PROJECT_ROOT/scripts/deploy-sign.mjs" sign-bundle \
                --file "$destination" --version "$version" --arch "$arch" >/dev/null
    fi
}

sign_test_bundle_unchecked() {
    bundle="$1"
    version="$2"
    arch="$3"
    source_root="$4"
    backend_inventory="$TEST_ROOT/unchecked-backend-inventory"
    payload="$TEST_ROOT/unchecked-signature-payload"
    : >"$backend_inventory"
    for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
        binary_path="$source_root/bin/$binary"
        binary_size="$(wc -c <"$binary_path" | tr -d '[:space:]')"
        binary_hash="$(openssl dgst -sha256 "$binary_path" | awk '{print $NF}')"
        printf 'binary=%s\nsize=%s\nsha256=%s\n' \
            "$binary" "$binary_size" "$binary_hash" >>"$backend_inventory"
    done
    content_hash="$(openssl dgst -sha256 "$bundle" | awk '{print $NF}')"
    frontend_hash="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    backend_hash="$(openssl dgst -sha256 "$backend_inventory" | awk '{print $NF}')"
    printf 'rustzen-release-v2\ncomponent=release\nversion=%s\narch=%s\ncontent_sha256=%s\nfrontend_sha256=%s\nbackend_sha256=%s\n' \
        "$version" "$arch" "$content_hash" "$frontend_hash" "$backend_hash" >"$payload"
    signature="$(RZ_TEST_KEY="$SIGNING_KEY" RZ_TEST_PAYLOAD="$payload" bun -e '
        const crypto = require("node:crypto");
        const fs = require("node:fs");
        const key = fs.readFileSync(process.env.RZ_TEST_KEY);
        const payload = fs.readFileSync(process.env.RZ_TEST_PAYLOAD);
        process.stdout.write(crypto.sign(null, payload, key).toString("hex"));
    ')"
    marker="$(printf '{"schemaVersion":2,"component":"release","version":"%s","arch":"%s","contentSha256":"%s","frontendSha256":"%s","backendSha256":"%s","signature":"%s"}' \
        "$version" "$arch" "$content_hash" "$frontend_hash" "$backend_hash" "$signature")"
    printf '\nRUSTZEN_BUNDLE_SIGNED_MARKER_BEGIN\n%s\nRUSTZEN_BUNDLE_SIGNED_MARKER_END\n' \
        "$marker" >>"$bundle"
}

run_installer() {
    bundle="$1"
    install_root="$2"
    systemd_dir="$3"
    systemctl_bin="$4"
    systemctl_log="$5"
    INSTALL_ROOT="$install_root" \
        SYSTEMD_DIR="$systemd_dir" \
        CLI_BIN_DIR="$install_root/command-bin" \
        SYSTEMCTL_BIN="$systemctl_bin" \
        SYSTEMCTL_LOG="$systemctl_log" \
        sh "$INSTALLER" "$bundle" >"$TEST_ROOT/initial-install.stdout"
}

run_installer_with_race() {
    bundle="$1"
    install_root="$2"
    systemd_dir="$3"
    systemctl_bin="$4"
    systemctl_log="$5"
    race_bin_dir="$6"
    pristine_bundle="$7"
    replacement_bundle="$8"
    PATH="$race_bin_dir:$PATH" \
        REAL_INSTALL_BIN="$REAL_INSTALL_BIN" \
        RACE_SOURCE="$bundle" \
        RACE_REPLACEMENT="$replacement_bundle" \
        INSTALL_ROOT="$install_root" \
        SYSTEMD_DIR="$systemd_dir" \
        CLI_BIN_DIR="$install_root/command-bin" \
        SYSTEMCTL_BIN="$systemctl_bin" \
        SYSTEMCTL_LOG="$systemctl_log" \
        sh "$INSTALLER" "$bundle" >"$TEST_ROOT/race-install.stdout"
    cmp -s "$pristine_bundle" "$install_root/data/releases/rz-2.0.0-x86_64.tar" \
        || fail "installer did not preserve the verified private snapshot"
}

expect_install_failure() {
    label="$1"
    bundle="$2"
    install_root="$3"
    systemd_dir="$4"
    systemctl_bin="$5"
    systemctl_log="$6"
    installer="${7-$INSTALLER}"
    if INSTALL_ROOT="$install_root" \
        SYSTEMD_DIR="$systemd_dir" \
        CLI_BIN_DIR="$install_root/command-bin" \
        SYSTEMCTL_BIN="$systemctl_bin" \
        SYSTEMCTL_LOG="$systemctl_log" \
        sh "$installer" "$bundle" >"$TEST_ROOT/$label.stdout" 2>"$TEST_ROOT/$label.stderr"; then
        fail "$label unexpectedly succeeded"
    fi
}

expect_sign_failure() {
    label="$1"
    bundle="$2"
    version="$3"
    arch="$4"
    if RUSTZEN_DEPLOY_SIGN_KEY_FILE="$SIGNING_KEY" \
        bun "$PROJECT_ROOT/scripts/deploy-sign.mjs" sign-bundle \
            --file "$bundle" --version "$version" --arch "$arch" \
            >"$TEST_ROOT/$label.stdout" 2>"$TEST_ROOT/$label.stderr"; then
        fail "$label unexpectedly succeeded"
    fi
}

SYSTEMCTL_BIN_PATH="$TEST_ROOT/systemctl"
SYSTEMCTL_LOG_PATH="$TEST_ROOT/systemctl.log"
make_fake_systemctl "$SYSTEMCTL_BIN_PATH"
: >"$SYSTEMCTL_LOG_PATH"
REAL_INSTALL_BIN="$(command -v install)"
RACE_BIN_DIR="$TEST_ROOT/race-bin"
mkdir -p "$RACE_BIN_DIR"
make_racing_install "$RACE_BIN_DIR/install"

SIGNING_KEY="$TEST_ROOT/signing-key.pem"
OTHER_SIGNING_KEY="$TEST_ROOT/other-signing-key.pem"
VERIFY_KEY="$(make_test_signing_key "$SIGNING_KEY")"
OTHER_VERIFY_KEY="$(make_test_signing_key "$OTHER_SIGNING_KEY")"
INSTALLER="$TEST_ROOT/rz-install"
OTHER_INSTALLER="$TEST_ROOT/rz-install-other"
make_packaged_installer "$INSTALLER" "$VERIFY_KEY"
make_packaged_installer "$OTHER_INSTALLER" "$OTHER_VERIFY_KEY"

BUNDLE_ONE="$TEST_ROOT/rz-1.2.3-x86_64.tar"
BUNDLE_TWO="$TEST_ROOT/rz-2.0.0-x86_64.tar"
make_bundle "1.2.3" "x86_64" "$BUNDLE_ONE"
make_bundle "2.0.0" "x86_64" "$BUNDLE_TWO"

RACE_BUNDLE="$TEST_ROOT/rz-2.0.0-x86_64-race.tar"
RACE_PRISTINE="$TEST_ROOT/rz-2.0.0-x86_64-pristine.tar"
RACE_REPLACEMENT="$TEST_ROOT/rz-2.0.0-x86_64-replacement.tar"
cp "$BUNDLE_TWO" "$RACE_BUNDLE"
cp "$BUNDLE_TWO" "$RACE_PRISTINE"
cp "$BUNDLE_TWO" "$RACE_REPLACEMENT"
perl -0pi -e 's/enable rz-full\.service/enable rx.service/' "$RACE_REPLACEMENT"
cmp -s "$RACE_PRISTINE" "$RACE_REPLACEMENT" \
    && fail "race replacement fixture did not change the signed bundle"
RACE_INSTALL_ROOT="$TEST_ROOT/race-install"
RACE_SYSTEMD_DIR="$TEST_ROOT/race-systemd"
run_installer_with_race \
    "$RACE_BUNDLE" "$RACE_INSTALL_ROOT" "$RACE_SYSTEMD_DIR" \
    "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/race-systemctl.log" "$RACE_BIN_DIR" \
    "$RACE_PRISTINE" "$RACE_REPLACEMENT"
assert_file_contains "enable rz-full.service" \
    "$RACE_INSTALL_ROOT/releases/2.0.0/setup-layout.sh" \
    "verified snapshot extraction"
if grep -Fq "enable rx.service" "$RACE_INSTALL_ROOT/releases/2.0.0/setup-layout.sh"; then
    fail "installer extracted content from the replaced input path"
fi
TEST_COUNT=$((TEST_COUNT + 1))

INSTALL_ROOT_ONE="$TEST_ROOT/install"
SYSTEMD_DIR_ONE="$TEST_ROOT/systemd"
run_installer \
    "$BUNDLE_ONE" "$INSTALL_ROOT_ONE" "$SYSTEMD_DIR_ONE" \
    "$SYSTEMCTL_BIN_PATH" "$SYSTEMCTL_LOG_PATH"
TEST_COUNT=$((TEST_COUNT + 1))

assert_equals "releases/1.2.3" "$(readlink "$INSTALL_ROOT_ONE/current")" \
    "initial current link"
assert_symlink "$INSTALL_ROOT_ONE/command-bin/rz"
assert_equals "$INSTALL_ROOT_ONE/current/bin/rz" \
    "$(readlink "$INSTALL_ROOT_ONE/command-bin/rz")" "rz command link"
assert_file_contains "Runtime secrets were generated locally" \
    "$TEST_ROOT/initial-install.stdout" "generated runtime secret prompt"
if grep -Fq '=replace-me' "$INSTALL_ROOT_ONE/config/rz.env" "$INSTALL_ROOT_ONE/config/rz-reports.env"; then
    fail "installed runtime configuration contains a placeholder"
fi
assert_equals \
    "$(config_value "$INSTALL_ROOT_ONE/config/rz.env" RUSTZEN_IPC_TOKEN)" \
    "$(config_value "$INSTALL_ROOT_ONE/config/rz-reports.env" RUSTZEN_IPC_TOKEN)" \
    "shared IPC token"
assert_equals \
    "$(config_value "$INSTALL_ROOT_ONE/config/rz.env" RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY)" \
    "$(config_value "$INSTALL_ROOT_ONE/config/rz-reports.env" RUSTZEN_REPORTS_NOTIFICATION_EVENT_KEY)" \
    "shared Reports notification key"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE")" "install root mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases")" "releases root mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3")" "release mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/bin")" \
    "release bin mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/systemd")" \
    "release systemd mode"
assert_equals "700" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/config")" \
    "release config directory mode"
assert_equals "$RUSTZEN_ADMIN_USER" \
    "$(file_owner "$INSTALL_ROOT_ONE/releases/1.2.3/config")" \
    "release config directory owner"
assert_equals "$RUSTZEN_ADMIN_GROUP" \
    "$(file_group "$INSTALL_ROOT_ONE/releases/1.2.3/config")" \
    "release config directory group"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/identity")" \
    "release identity directory mode"
assert_equals "700" "$(file_mode "$INSTALL_ROOT_ONE/config")" "shared config directory mode"
assert_equals "711" "$(file_mode "$INSTALL_ROOT_ONE/data")" "data traversal mode"
assert_equals "711" "$(file_mode "$INSTALL_ROOT_ONE/data/db")" "database traversal mode"
assert_equals "750" "$(file_mode "$INSTALL_ROOT_ONE/data/releases")" \
    "release store mode"
assert_equals "$RUSTZEN_ADMIN_GROUP" "$(file_group "$INSTALL_ROOT_ONE/data/releases")" \
    "release store group"
assert_equals "711" "$(file_mode "$INSTALL_ROOT_ONE/logs")" "logs traversal mode"
assert_equals "750" "$(file_mode "$INSTALL_ROOT_ONE/data/reports")" "Reports data mode"
assert_equals "750" "$(file_mode "$INSTALL_ROOT_ONE/data/reports/db")" "Reports database mode"
for module in admin monitor insights reports; do
    assert_equals "2770" "$(file_special_mode "$INSTALL_ROOT_ONE/logs/$module")" "$module logs mode"
    assert_equals "$RUSTZEN_LOG_CONTROL_GROUP" "$(file_group "$INSTALL_ROOT_ONE/logs/$module")" "$module logs group"
done
for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
    path="$INSTALL_ROOT_ONE/releases/1.2.3/bin/$binary"
    assert_exists "$path"
    assert_equals "755" "$(file_mode "$path")" "$binary mode"
done
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/setup-layout.sh")" \
    "installer mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz.env")" \
    "release config mode"
assert_equals "$RUSTZEN_ADMIN_USER" \
    "$(file_owner "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz.env")" \
    "release config owner"
assert_equals "$RUSTZEN_ADMIN_GROUP" \
    "$(file_group "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz.env")" \
    "release config group"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz-reports.env")" \
    "Reports release config mode"
assert_equals "$RUSTZEN_ADMIN_USER" \
    "$(file_owner "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz-reports.env")" \
    "Reports release config owner"
assert_equals "$RUSTZEN_ADMIN_GROUP" \
    "$(file_group "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz-reports.env")" \
    "Reports release config group"
assert_equals "644" \
    "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/identity/controller.json")" \
    "controller identity mode"
for unit in rz-full.service rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service rz-update.service rz-update.path; do
    assert_equals "644" \
        "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/systemd/$unit")" \
        "$unit mode"
done
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/config/rz.env")" \
    "shared config mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/config/rz-reports.env")" \
    "Reports config mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/data/db/admin/bootstrap-owner-password")" \
    "bootstrap owner input mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/data/initial-owner-password")" \
    "initial owner credential mode"
if [ "$(cat "$INSTALL_ROOT_ONE/data/db/admin/bootstrap-owner-password")" != "$(cat "$INSTALL_ROOT_ONE/data/initial-owner-password")" ]; then
    fail "bootstrap and retained owner credentials differ"
fi
for directory in data/db data/releases data/reports data/uploads data/avatars logs; do
    [ -d "$INSTALL_ROOT_ONE/$directory" ] || fail "missing shared directory: $directory"
done
cmp -s "$BUNDLE_ONE" "$INSTALL_ROOT_ONE/data/releases/rz-1.2.3-x86_64.tar" \
    || fail "installed signed bundle was not preserved byte-for-byte"
assert_equals "640" \
    "$(file_mode "$INSTALL_ROOT_ONE/data/releases/rz-1.2.3-x86_64.tar")" \
    "stored release bundle mode"
assert_equals "$RUSTZEN_ADMIN_GROUP" \
    "$(file_group "$INSTALL_ROOT_ONE/data/releases/rz-1.2.3-x86_64.tar")" \
    "stored release bundle group"
for unit in rz-full.service rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service rz-update.service rz-update.path; do
    assert_symlink "$SYSTEMD_DIR_ONE/$unit"
    assert_equals \
        "$INSTALL_ROOT_ONE/current/systemd/$unit" \
        "$(readlink "$SYSTEMD_DIR_ONE/$unit")" \
        "$unit systemd link"
done
EXPECTED_SYSTEMCTL_CALLS="$(printf 'daemon-reload\nenable rz-full.service rz-update.path\n')"
assert_equals "$EXPECTED_SYSTEMCTL_CALLS" "$(cat "$SYSTEMCTL_LOG_PATH")" \
    "initial systemctl calls"

printf 'preserved-config\n' >"$INSTALL_ROOT_ONE/config/rz.env"
chmod 0600 "$INSTALL_ROOT_ONE/config/rz.env"
expect_install_failure \
    "existing-installation" "$BUNDLE_TWO" "$INSTALL_ROOT_ONE" "$SYSTEMD_DIR_ONE" \
    "$SYSTEMCTL_BIN_PATH" "$SYSTEMCTL_LOG_PATH"
assert_file_contains "must be updated through the Admin release worker" \
    "$TEST_ROOT/existing-installation.stderr" "direct upgrade rejection"
TEST_COUNT=$((TEST_COUNT + 1))
assert_equals "releases/1.2.3" "$(readlink "$INSTALL_ROOT_ONE/current")" \
    "current link after direct upgrade rejection"
assert_exists "$INSTALL_ROOT_ONE/releases/1.2.3/bin/rz-admin"
if [ -e "$INSTALL_ROOT_ONE/releases/2.0.0" ]; then
    fail "direct setup upgrade created a second release directory"
fi
assert_equals "preserved-config" "$(cat "$INSTALL_ROOT_ONE/config/rz.env")" \
    "shared config preservation"

for kind in wrong dangling; do
    UNIT_LINK_ROOT="$TEST_ROOT/$kind-unit-link-install"
    UNIT_LINK_SYSTEMD="$TEST_ROOT/$kind-unit-link-systemd"
    mkdir -p "$UNIT_LINK_SYSTEMD"
    if [ "$kind" = wrong ]; then
        UNIT_LINK_TARGET="/unexpected/rz-admin.service"
    else
        UNIT_LINK_TARGET="$TEST_ROOT/missing-rz-admin.service"
    fi
    ln -s "$UNIT_LINK_TARGET" "$UNIT_LINK_SYSTEMD/rz-admin.service"
    expect_install_failure \
        "$kind-unit-link" "$BUNDLE_TWO" "$UNIT_LINK_ROOT" "$UNIT_LINK_SYSTEMD" \
        "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/$kind-unit-link-systemctl.log"
    assert_symlink "$UNIT_LINK_SYSTEMD/rz-admin.service"
    assert_equals "$UNIT_LINK_TARGET" "$(readlink "$UNIT_LINK_SYSTEMD/rz-admin.service")" \
        "$kind unit link preservation"
done
TEST_COUNT=$((TEST_COUNT + 1))

PRELINK_ROOT="$TEST_ROOT/prelinked-unit-install"
PRELINK_SYSTEMD="$TEST_ROOT/prelinked-unit-systemd"
mkdir -p "$PRELINK_SYSTEMD"
PRELINK_TARGET="$PRELINK_ROOT/current/systemd/rz-admin.service"
ln -s "$PRELINK_TARGET" "$PRELINK_SYSTEMD/rz-admin.service"
if INSTALL_ROOT="$PRELINK_ROOT" \
    SYSTEMD_DIR="$PRELINK_SYSTEMD" \
    CLI_BIN_DIR="$PRELINK_ROOT/command-bin" \
    SYSTEMCTL_BIN="$SYSTEMCTL_BIN_PATH" \
    SYSTEMCTL_LOG="$TEST_ROOT/prelinked-unit-systemctl.log" \
    SYSTEMCTL_FAIL_ON='enable rz-full.service rz-update.path' \
    sh "$INSTALLER" "$BUNDLE_TWO" >"$TEST_ROOT/prelinked-unit.stdout" 2>"$TEST_ROOT/prelinked-unit.stderr"; then
    fail "prelinked unit install unexpectedly succeeded"
fi
assert_symlink "$PRELINK_SYSTEMD/rz-admin.service"
assert_equals "$PRELINK_TARGET" "$(readlink "$PRELINK_SYSTEMD/rz-admin.service")" \
    "exact pre-existing unit link after systemctl failure"
TEST_COUNT=$((TEST_COUNT + 1))

expect_install_failure \
    "existing-version" "$BUNDLE_ONE" "$INSTALL_ROOT_ONE" "$SYSTEMD_DIR_ONE" \
    "$SYSTEMCTL_BIN_PATH" "$SYSTEMCTL_LOG_PATH"
assert_equals "releases/1.2.3" "$(readlink "$INSTALL_ROOT_ONE/current")" \
    "current link after duplicate setup rejection"
TEST_COUNT=$((TEST_COUNT + 1))

UNSIGNED_BUNDLE="$TEST_ROOT/rz-2.1.0-x86_64-unsigned.tar"
make_bundle "2.1.0" "x86_64" "$UNSIGNED_BUNDLE" "" "$SIGNING_KEY" unsigned
expect_install_failure \
    "unsigned-bundle" "$UNSIGNED_BUNDLE" "$TEST_ROOT/unsigned-install" \
    "$TEST_ROOT/unsigned-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/unsigned-systemctl.log"
assert_file_contains "signed release bundle marker" "$TEST_ROOT/unsigned-bundle.stderr" \
    "unsigned bundle rejection"
TEST_COUNT=$((TEST_COUNT + 1))

INVALID_IDENTITY_BUNDLE="$TEST_ROOT/rz-2.1.1-x86_64-invalid-identity.tar"
make_bundle "2.1.1" "x86_64" "$INVALID_IDENTITY_BUNDLE" "" "$SIGNING_KEY" unsigned
perl -0pi -e 's/binary=rz-monitor/binary=rx-monitor/' "$INVALID_IDENTITY_BUNDLE"
expect_sign_failure \
    "invalid-binary-identity" "$INVALID_IDENTITY_BUNDLE" "2.1.1" "x86_64"
assert_file_contains "identity marker mismatch: rz-monitor" \
    "$TEST_ROOT/invalid-binary-identity.stderr" "signer binary identity rejection"
TEST_COUNT=$((TEST_COUNT + 1))

INVALID_INSTALL_IDENTITY_BUNDLE="$TEST_ROOT/rz-2.1.2-x86_64-invalid-installer-identity.tar"
make_bundle "2.1.2" "x86_64" "$INVALID_INSTALL_IDENTITY_BUNDLE" "" "$SIGNING_KEY" unsigned
INVALID_INSTALL_SOURCE="$TEST_ROOT/source-2.1.2-x86_64-$(basename "$INVALID_INSTALL_IDENTITY_BUNDLE")"
INVALID_INSTALL_ROOT="$INVALID_INSTALL_SOURCE/rz-2.1.2-x86_64"
perl -0pi -e 's/version=2\.1\.2/version=9.9.9/' \
    "$INVALID_INSTALL_ROOT/bin/rz-monitor"
COPYFILE_DISABLE=1 tar -cf "$INVALID_INSTALL_IDENTITY_BUNDLE" \
    -C "$INVALID_INSTALL_SOURCE" rz-2.1.2-x86_64
sign_test_bundle_unchecked \
    "$INVALID_INSTALL_IDENTITY_BUNDLE" "2.1.2" "x86_64" "$INVALID_INSTALL_ROOT"
expect_install_failure \
    "invalid-installer-identity" "$INVALID_INSTALL_IDENTITY_BUNDLE" \
    "$TEST_ROOT/invalid-installer-install" "$TEST_ROOT/invalid-installer-systemd" \
    "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/invalid-installer-systemctl.log"
assert_file_contains "identity marker mismatch: rz-monitor" \
    "$TEST_ROOT/invalid-installer-identity.stderr" "installer binary identity rejection"
TEST_COUNT=$((TEST_COUNT + 1))

expect_install_failure \
    "wrong-signing-key" "$BUNDLE_TWO" "$TEST_ROOT/wrong-key-install" \
    "$TEST_ROOT/wrong-key-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/wrong-key-systemctl.log" \
    "$OTHER_INSTALLER"
assert_file_contains "Ed25519 signature verification failed" \
    "$TEST_ROOT/wrong-signing-key.stderr" "wrong signing key rejection"
TEST_COUNT=$((TEST_COUNT + 1))

TAMPERED_BUNDLE="$TEST_ROOT/rz-1.2.3-x86_64-tampered.tar"
cp "$BUNDLE_ONE" "$TAMPERED_BUNDLE"
perl -0pi -e 's/enable rz-full\.service/enable rx.service/' "$TAMPERED_BUNDLE"
cmp -s "$BUNDLE_ONE" "$TAMPERED_BUNDLE" \
    && fail "tampered bundle fixture did not change the signed bundle"
expect_install_failure \
    "tampered-bundle" "$TAMPERED_BUNDLE" "$TEST_ROOT/tampered-install" \
    "$TEST_ROOT/tampered-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/tampered-systemctl.log"
TEST_COUNT=$((TEST_COUNT + 1))

MISSING_BUNDLE="$TEST_ROOT/rz-3.0.0-x86_64-missing.tar"
make_bundle "3.0.0" "x86_64" "$MISSING_BUNDLE" "bin/rz-reports" "$SIGNING_KEY" unsigned
expect_install_failure \
    "missing-file" "$MISSING_BUNDLE" "$TEST_ROOT/missing-install" \
    "$TEST_ROOT/missing-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/missing-systemctl.log"
TEST_COUNT=$((TEST_COUNT + 1))

COMPRESSED_BUNDLE="$TEST_ROOT/rz-3.1.0-x86_64.tar.gz"
COMPRESSED_SOURCE="$TEST_ROOT/compressed-source"
mkdir -p "$COMPRESSED_SOURCE/rz-3.1.0-x86_64"
printf 'compressed\n' >"$COMPRESSED_SOURCE/rz-3.1.0-x86_64/file"
COPYFILE_DISABLE=1 tar -czf "$COMPRESSED_BUNDLE" -C "$COMPRESSED_SOURCE" rz-3.1.0-x86_64
expect_install_failure \
    "compressed-bundle" "$COMPRESSED_BUNDLE" "$TEST_ROOT/compressed-install" \
    "$TEST_ROOT/compressed-systemd" "$SYSTEMCTL_BIN_PATH" \
    "$TEST_ROOT/compressed-systemctl.log"
TEST_COUNT=$((TEST_COUNT + 1))

TRAVERSAL_PARENT="$TEST_ROOT/traversal-source"
mkdir -p "$TRAVERSAL_PARENT/root"
printf 'escape\n' >"$TRAVERSAL_PARENT/escape"
TRAVERSAL_BUNDLE="$TEST_ROOT/traversal.tar"
COPYFILE_DISABLE=1 tar -cf "$TRAVERSAL_BUNDLE" -C "$TRAVERSAL_PARENT/root" ../escape
expect_install_failure \
    "path-traversal" "$TRAVERSAL_BUNDLE" "$TEST_ROOT/traversal-install" \
    "$TEST_ROOT/traversal-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/traversal-systemctl.log"
TEST_COUNT=$((TEST_COUNT + 1))

NON_SYMLINK_ROOT="$TEST_ROOT/non-symlink-install"
mkdir -p "$NON_SYMLINK_ROOT"
printf 'do-not-replace\n' >"$NON_SYMLINK_ROOT/current"
NON_SYMLINK_BUNDLE="$TEST_ROOT/rz-4.0.0-aarch64.tar"
make_bundle "4.0.0" "aarch64" "$NON_SYMLINK_BUNDLE"
expect_install_failure \
    "non-symlink-current" "$NON_SYMLINK_BUNDLE" "$NON_SYMLINK_ROOT" \
    "$TEST_ROOT/non-symlink-systemd" "$SYSTEMCTL_BIN_PATH" \
    "$TEST_ROOT/non-symlink-systemctl.log"
assert_equals "do-not-replace" "$(cat "$NON_SYMLINK_ROOT/current")" \
    "non-symlink current preservation"
TEST_COUNT=$((TEST_COUNT + 1))

for unit in rz-full.service rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
    if grep -Eq '^Requires=' "$PROJECT_ROOT/deploy/$unit"; then
        fail "$unit must not use Requires="
    fi
done
grep -Eq '^Wants=.*rz-recovery\.service' "$PROJECT_ROOT/deploy/rz-full.service" \
    || fail "rz-full.service does not want rz-recovery.service"
grep -Fqx 'After=network.target rz-recovery.service' "$PROJECT_ROOT/deploy/rz-full.service" \
    || fail "rz-full.service is not ordered after recovery"
grep -Fqx 'Type=oneshot' "$PROJECT_ROOT/deploy/rz-full.service" \
    || fail "rz-full.service is not a oneshot aggregate"
grep -Fqx 'ExecStart=/bin/true' "$PROJECT_ROOT/deploy/rz-full.service" \
    || fail "rz-full.service aggregate command is invalid"
grep -Fqx 'RemainAfterExit=yes' "$PROJECT_ROOT/deploy/rz-full.service" \
    || fail "rz-full.service does not remain active"
grep -Fqx 'WantedBy=multi-user.target' "$PROJECT_ROOT/deploy/rz-full.service" \
    || fail "rz-full.service is not enabled for normal boot"
grep -Fqx 'PartOf=rz-full.service' "$PROJECT_ROOT/deploy/rz-recovery.service" \
    || fail "recovery does not declare PartOf=rz-full.service"
grep -Fqx 'Restart=on-failure' "$PROJECT_ROOT/deploy/rz-recovery.service" \
    || fail "recovery does not restart on failure"
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-admin update recover' \
    "$PROJECT_ROOT/deploy/rz-recovery.service" || fail "recovery ExecStart is invalid"
for unit in rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
    grep -Eq "^Wants=.*${unit}" "$PROJECT_ROOT/deploy/rz-full.service" \
        || fail "rz-full.service does not want $unit"
    grep -Eq "^Before=.*${unit}" "$PROJECT_ROOT/deploy/rz-recovery.service" \
        || fail "recovery is not ordered before $unit"
    grep -Eq '^After=.*rz-recovery\.service' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit is not ordered after recovery"
    grep -Fqx 'ExecCondition=/usr/bin/test ! -e /opt/rz/data/recovery-blocked' \
        "$PROJECT_ROOT/deploy/$unit" || fail "$unit has no recovery failure guard"
    grep -Fqx 'PartOf=rz-full.service' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit does not declare PartOf=rz-full.service"
    grep -Fqx 'Restart=on-failure' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit does not restart independently"
    grep -Fqx 'UMask=0027' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit does not preserve group-managed service log files"
    grep -Eq '^StartLimitIntervalSec=' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit has no start-limit interval"
    grep -Eq '^StartLimitBurst=' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit has no start-limit burst"
done
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-admin serve' \
    "$PROJECT_ROOT/deploy/rz-admin.service" || fail "Admin ExecStart is invalid"
grep -Fqx 'SupplementaryGroups=rz-log-control' \
    "$PROJECT_ROOT/deploy/rz-admin.service" || fail "Admin log control group is missing"
grep -Fqx 'ReadWritePaths=/opt/rz/data/db/admin /opt/rz/data/releases /opt/rz/data/uploads /opt/rz/data/avatars /opt/rz/data/update-requests /opt/rz/logs/admin /opt/rz/logs/monitor /opt/rz/logs/insights /opt/rz/logs/reports' \
    "$PROJECT_ROOT/deploy/rz-admin.service" || fail "Admin module log write paths are incomplete"
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-monitor controller' \
    "$PROJECT_ROOT/deploy/rz-monitor.service" || fail "Monitor ExecStart is invalid"
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-insights serve' \
    "$PROJECT_ROOT/deploy/rz-insights.service" || fail "Insights ExecStart is invalid"
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-reports serve' \
    "$PROJECT_ROOT/deploy/rz-reports.service" || fail "Reports ExecStart is invalid"
for directive in \
    'User=rz-reports' \
    'Group=rz-reports' \
    'EnvironmentFile=/opt/rz/config/rz-reports.env' \
    'NoNewPrivileges=yes' \
    'PrivateTmp=yes' \
    'ProtectSystem=strict' \
    'ReadWritePaths=/opt/rz/data/reports /opt/rz/logs/reports'; do
    grep -Fqx "$directive" "$PROJECT_ROOT/deploy/rz-reports.service" \
        || fail "Reports unit is missing: $directive"
done
if grep -Fq 'rz-monitor-agent.service' "$PROJECT_ROOT/deploy/rz-full.service"; then
    fail "Monitor Agent must not be part of rz-full.service"
fi
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-monitor-agent' \
    "$PROJECT_ROOT/deploy/rz-monitor-agent.service" \
    || fail "Monitor Agent does not use the versioned Agent binary"
grep -Fqx 'EnvironmentFile=/opt/rz/config/rz-monitor-agent.env' "$PROJECT_ROOT/deploy/rz-monitor-agent.service" \
    || fail "Monitor Agent must use its selected configuration file"
grep -Fqx 'Type=notify' "$PROJECT_ROOT/deploy/rz-monitor-agent.service" \
    || fail "Monitor Agent must wait for a report-delivery readiness signal"
grep -Fqx 'NotifyAccess=main' "$PROJECT_ROOT/deploy/rz-monitor-agent.service" \
    || fail "Monitor Agent readiness must be emitted by its main process"
grep -Fqx 'TimeoutStartSec=infinity' "$PROJECT_ROOT/deploy/rz-monitor-agent.service" \
    || fail "Monitor Agent must keep retrying reports while unready"
assert_equals "1" "$(grep -c '^ExecStart=' "$PROJECT_ROOT/deploy/rz-monitor-agent.service")" \
    "Monitor Agent ExecStart count"
if grep -Fq 'rz-monitor agent' "$PROJECT_ROOT/deploy/rz-monitor-agent.service"; then
    fail "Monitor Agent must not use the rejected rz-monitor agent subcommand"
fi
TEST_COUNT=$((TEST_COUNT + 1))

echo "setup-layout integration tests passed ($TEST_COUNT groups)"
