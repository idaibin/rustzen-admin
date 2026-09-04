#!/usr/bin/env sh
set -eu

PROJECT_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
export RUSTZEN_REPORTS_USER="${RUSTZEN_REPORTS_USER:-$(id -un)}"
export RUSTZEN_REPORTS_GROUP="${RUSTZEN_REPORTS_GROUP:-$(id -gn)}"
INSTALLER="$PROJECT_ROOT/deploy/setup-layout.sh"
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

file_mode() {
    case "$(uname -s)" in
        Darwin|FreeBSD|NetBSD|OpenBSD) stat -f '%Lp' "$1" ;;
        *) stat -c '%a' "$1" ;;
    esac
}

make_fake_systemctl() {
    destination="$1"
    printf '%s\n' \
        '#!/usr/bin/env sh' \
        'set -eu' \
        ': "${SYSTEMCTL_LOG:?}"' \
        'printf "%s\n" "$*" >> "$SYSTEMCTL_LOG"' \
        >"$destination"
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

    mkdir -p "$release_root/bin" "$release_root/systemd" "$release_root/config"
    for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
        printf 'fixture %s %s\n' "$binary" "$version" >"$release_root/bin/$binary"
        chmod 0755 "$release_root/bin/$binary"
    done
    for unit in rz.target rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
        cp "$PROJECT_ROOT/deploy/$unit" "$release_root/systemd/$unit"
    done
    printf 'RUSTZEN_ENV=production\nRUSTZEN_JWT_SECRET=%s\nRUSTZEN_DEPLOY_VERIFY_KEY=%s\n' \
        "$version" "$VERIFY_KEY" \
        >"$release_root/config/rz.env"
    printf 'RUSTZEN_ENV=production\nRUSTZEN_IPC_TOKEN=%s\nRUSTZEN_REPORTS_CREDENTIAL_KEY=%s\n' "$version" "$version" \
        >"$release_root/config/rz-reports.env"
    cp "$INSTALLER" "$release_root/setup-layout.sh"
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

run_installer() {
    bundle="$1"
    install_root="$2"
    systemd_dir="$3"
    systemctl_bin="$4"
    systemctl_log="$5"
    INSTALL_ROOT="$install_root" \
        SYSTEMD_DIR="$systemd_dir" \
        SYSTEMCTL_BIN="$systemctl_bin" \
        SYSTEMCTL_LOG="$systemctl_log" \
        RUSTZEN_DEPLOY_VERIFY_KEY="$VERIFY_KEY" \
        sh "$INSTALLER" "$bundle" >"$TEST_ROOT/initial-install.stdout"
}

expect_install_failure() {
    label="$1"
    bundle="$2"
    install_root="$3"
    systemd_dir="$4"
    systemctl_bin="$5"
    systemctl_log="$6"
    verify_key="${7-$VERIFY_KEY}"
    if INSTALL_ROOT="$install_root" \
        SYSTEMD_DIR="$systemd_dir" \
        SYSTEMCTL_BIN="$systemctl_bin" \
        SYSTEMCTL_LOG="$systemctl_log" \
        RUSTZEN_DEPLOY_VERIFY_KEY="$verify_key" \
        sh "$INSTALLER" "$bundle" >"$TEST_ROOT/$label.stdout" 2>"$TEST_ROOT/$label.stderr"; then
        fail "$label unexpectedly succeeded"
    fi
}

SYSTEMCTL_BIN_PATH="$TEST_ROOT/systemctl"
SYSTEMCTL_LOG_PATH="$TEST_ROOT/systemctl.log"
make_fake_systemctl "$SYSTEMCTL_BIN_PATH"
: >"$SYSTEMCTL_LOG_PATH"

SIGNING_KEY="$TEST_ROOT/signing-key.pem"
OTHER_SIGNING_KEY="$TEST_ROOT/other-signing-key.pem"
VERIFY_KEY="$(make_test_signing_key "$SIGNING_KEY")"
OTHER_VERIFY_KEY="$(make_test_signing_key "$OTHER_SIGNING_KEY")"

BUNDLE_ONE="$TEST_ROOT/rz-1.2.3-x86_64.tar"
BUNDLE_TWO="$TEST_ROOT/rz-2.0.0-x86_64.tar"
make_bundle "1.2.3" "x86_64" "$BUNDLE_ONE"
make_bundle "2.0.0" "x86_64" "$BUNDLE_TWO"

INSTALL_ROOT_ONE="$TEST_ROOT/install"
SYSTEMD_DIR_ONE="$TEST_ROOT/systemd"
run_installer \
    "$BUNDLE_ONE" "$INSTALL_ROOT_ONE" "$SYSTEMD_DIR_ONE" \
    "$SYSTEMCTL_BIN_PATH" "$SYSTEMCTL_LOG_PATH"
TEST_COUNT=$((TEST_COUNT + 1))

assert_equals "releases/1.2.3" "$(readlink "$INSTALL_ROOT_ONE/current")" \
    "initial current link"
assert_file_contains "$INSTALL_ROOT_ONE/config/rz.env" \
    "$TEST_ROOT/initial-install.stdout" "main config prompt"
assert_file_contains "$INSTALL_ROOT_ONE/config/rz-reports.env" \
    "$TEST_ROOT/initial-install.stdout" "Reports config prompt"
assert_file_contains "Keep RUSTZEN_IPC_TOKEN identical in both files" \
    "$TEST_ROOT/initial-install.stdout" "IPC token prompt"
assert_file_contains "RUSTZEN_TIMEZONE" \
    "$TEST_ROOT/initial-install.stdout" "timezone prompt"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE")" "install root mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases")" "releases root mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3")" "release mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/bin")" \
    "release bin mode"
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/systemd")" \
    "release systemd mode"
assert_equals "700" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/config")" \
    "release config directory mode"
assert_equals "700" "$(file_mode "$INSTALL_ROOT_ONE/config")" "shared config directory mode"
assert_equals "711" "$(file_mode "$INSTALL_ROOT_ONE/data")" "data traversal mode"
assert_equals "711" "$(file_mode "$INSTALL_ROOT_ONE/logs")" "logs traversal mode"
assert_equals "750" "$(file_mode "$INSTALL_ROOT_ONE/data/reports")" "Reports data mode"
assert_equals "750" "$(file_mode "$INSTALL_ROOT_ONE/data/reports/db")" "Reports database mode"
assert_equals "750" "$(file_mode "$INSTALL_ROOT_ONE/logs/reports")" "Reports logs mode"
for binary in rz rz-admin rz-monitor rz-insights rz-reports; do
    path="$INSTALL_ROOT_ONE/releases/1.2.3/bin/$binary"
    assert_exists "$path"
    assert_equals "755" "$(file_mode "$path")" "$binary mode"
done
assert_equals "755" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/setup-layout.sh")" \
    "installer mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz.env")" \
    "release config mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/config/rz-reports.env")" \
    "Reports release config mode"
for unit in rz.target rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
    assert_equals "644" \
        "$(file_mode "$INSTALL_ROOT_ONE/releases/1.2.3/systemd/$unit")" \
        "$unit mode"
done
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/config/rz.env")" \
    "shared config mode"
assert_equals "600" "$(file_mode "$INSTALL_ROOT_ONE/config/rz-reports.env")" \
    "Reports config mode"
for directory in data/db data/releases data/reports data/uploads data/avatars logs; do
    [ -d "$INSTALL_ROOT_ONE/$directory" ] || fail "missing shared directory: $directory"
done
cmp -s "$BUNDLE_ONE" "$INSTALL_ROOT_ONE/data/releases/rz-1.2.3-x86_64.tar" \
    || fail "installed signed bundle was not preserved byte-for-byte"
for unit in rz.target rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
    assert_symlink "$SYSTEMD_DIR_ONE/$unit"
    assert_equals \
        "$INSTALL_ROOT_ONE/current/systemd/$unit" \
        "$(readlink "$SYSTEMD_DIR_ONE/$unit")" \
        "$unit systemd link"
done
EXPECTED_SYSTEMCTL_CALLS="$(printf 'daemon-reload\nenable rz.target\n')"
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

expect_install_failure \
    "wrong-signing-key" "$BUNDLE_TWO" "$TEST_ROOT/wrong-key-install" \
    "$TEST_ROOT/wrong-key-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/wrong-key-systemctl.log" \
    "$OTHER_VERIFY_KEY"
assert_file_contains "Ed25519 signature verification failed" \
    "$TEST_ROOT/wrong-signing-key.stderr" "wrong signing key rejection"
TEST_COUNT=$((TEST_COUNT + 1))

TAMPERED_BUNDLE="$TEST_ROOT/rz-1.2.3-x86_64-tampered.tar"
cp "$BUNDLE_ONE" "$TAMPERED_BUNDLE"
perl -0pi -e 's/fixture rz-admin/fixturf rz-admin/' "$TAMPERED_BUNDLE"
expect_install_failure \
    "tampered-bundle" "$TAMPERED_BUNDLE" "$TEST_ROOT/tampered-install" \
    "$TEST_ROOT/tampered-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/tampered-systemctl.log"
assert_file_contains "content hash does not match" "$TEST_ROOT/tampered-bundle.stderr" \
    "tampered bundle rejection"
TEST_COUNT=$((TEST_COUNT + 1))

expect_install_failure \
    "missing-trusted-key" "$BUNDLE_TWO" "$TEST_ROOT/missing-key-install" \
    "$TEST_ROOT/missing-key-systemd" "$SYSTEMCTL_BIN_PATH" "$TEST_ROOT/missing-key-systemctl.log" \
    ""
assert_file_contains "RUSTZEN_DEPLOY_VERIFY_KEY must be a trusted" \
    "$TEST_ROOT/missing-trusted-key.stderr" "missing trusted key rejection"
TEST_COUNT=$((TEST_COUNT + 1))

MISSING_BUNDLE="$TEST_ROOT/rz-3.0.0-x86_64-missing.tar"
make_bundle "3.0.0" "x86_64" "$MISSING_BUNDLE" "bin/rz-reports"
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

for unit in rz.target rz-recovery.service rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
    if grep -Eq '^Requires=' "$PROJECT_ROOT/deploy/$unit"; then
        fail "$unit must not use Requires="
    fi
done
grep -Eq '^Wants=.*rz-recovery\.service' "$PROJECT_ROOT/deploy/rz.target" \
    || fail "rz.target does not want rz-recovery.service"
grep -Fqx 'PartOf=rz.target' "$PROJECT_ROOT/deploy/rz-recovery.service" \
    || fail "recovery does not declare PartOf=rz.target"
grep -Fqx 'Restart=on-failure' "$PROJECT_ROOT/deploy/rz-recovery.service" \
    || fail "recovery does not restart on failure"
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-admin update recover' \
    "$PROJECT_ROOT/deploy/rz-recovery.service" || fail "recovery ExecStart is invalid"
for unit in rz-admin.service rz-monitor.service rz-insights.service rz-reports.service; do
    grep -Eq "^Wants=.*${unit}" "$PROJECT_ROOT/deploy/rz.target" \
        || fail "rz.target does not want $unit"
    grep -Eq "^Before=.*${unit}" "$PROJECT_ROOT/deploy/rz-recovery.service" \
        || fail "recovery is not ordered before $unit"
    grep -Eq '^After=.*rz-recovery\.service' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit is not ordered after recovery"
    grep -Fqx 'ExecCondition=/usr/bin/test ! -e /opt/rz/data/recovery-blocked' \
        "$PROJECT_ROOT/deploy/$unit" || fail "$unit has no recovery failure guard"
    grep -Fqx 'PartOf=rz.target' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit does not declare PartOf=rz.target"
    grep -Fqx 'Restart=on-failure' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit does not restart independently"
    grep -Eq '^StartLimitIntervalSec=' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit has no start-limit interval"
    grep -Eq '^StartLimitBurst=' "$PROJECT_ROOT/deploy/$unit" \
        || fail "$unit has no start-limit burst"
done
grep -Fqx 'ExecStart=/opt/rz/current/bin/rz-admin serve' \
    "$PROJECT_ROOT/deploy/rz-admin.service" || fail "Admin ExecStart is invalid"
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
if grep -Fq 'rz-monitor-agent.service' "$PROJECT_ROOT/deploy/rz.target"; then
    fail "Monitor Agent must not be part of rz.target"
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
