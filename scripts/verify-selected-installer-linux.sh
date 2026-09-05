#!/usr/bin/env bash
set -euo pipefail
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository"
fixtures=target/installer-fixtures
rm -rf "$fixtures"
RUSTZEN_INSTALLER_OUTPUT="$fixtures/x86" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
! RUSTZEN_INSTALLER_OUTPUT="$fixtures/escaped" RUSTZEN_INSTALLER_KEY_ID=../../escaped pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_OUTPUT="$fixtures/valid" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_OUTPUT="$fixtures/valid-other" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
for mutation in artifact-class unknown-top-level unknown-owner-field; do
  RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_SCHEMA_MUTATION="$mutation" RUSTZEN_INSTALLER_OUTPUT="$fixtures/schema-$mutation" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
done
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_ARTIFACT=node-agent RUSTZEN_INSTALLER_OUTPUT="$fixtures/agent" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
for mutation in duplicate omitted path header checksum-width padding trailing; do
  RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_MUTATION="$mutation" RUSTZEN_INSTALLER_OUTPUT="$fixtures/$mutation" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
done
for mutation in build-id archive-hash manifest-hash key-id; do
  RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_ENVELOPE_MUTATION="$mutation" RUSTZEN_INSTALLER_OUTPUT="$fixtures/envelope-$mutation" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
done
for mutation in capabilities config-owners binary-digest api-digest schema-owner data-owner web-digest native-digest protocol-digest selection-digest; do
  RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_MANIFEST_MUTATION="$mutation" RUSTZEN_INSTALLER_OUTPUT="$fixtures/manifest-$mutation" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
done
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_ARTIFACT=node-agent RUSTZEN_INSTALLER_MANIFEST_MUTATION=agent-capability RUSTZEN_INSTALLER_OUTPUT="$fixtures/agent-bad" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
docker run --rm --platform linux/arm64 \
  -v "$repository:/work" -w /work \
  -v rustzen-installer-cargo:/usr/local/cargo/registry \
  -v rustzen-installer-target:/work/target/installer-linux \
  rust:1.95-bookworm bash -ec '
    cargo build -p rustzen-cli --target-dir target/installer-linux
    rz=target/installer-linux/debug/rz
    fixture=target/installer-fixtures/valid
    other=target/installer-fixtures/valid-other
    x86=target/installer-fixtures/x86
    agent=target/installer-fixtures/agent
    base=$(mktemp -d)
    "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    "$rz" --json apply --dry-run --destination "$base/dry" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    "$rz" --json verify --archive "$agent/archive.tar" --manifest "$agent/release-manifest.json" --envelope "$agent/signature-envelope.json" --trusted-public-key "$agent/public.pem" --key-id installer-test | grep -F node-agent
    test ! -e "$base/dry"
    ! "$rz" --json apply --dry-run --destination "$base/x86" --archive "$x86/archive.tar" --manifest "$x86/release-manifest.json" --envelope "$x86/signature-envelope.json" --trusted-public-key "$x86/public.pem" --key-id installer-test
    "$rz" --json apply --destination "$base/rz" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    "$rz" --json install-status --destination "$base/rz" | grep -F true
    "$rz" --json apply --destination "$base/agent" --archive "$agent/archive.tar" --manifest "$agent/release-manifest.json" --envelope "$agent/signature-envelope.json" --trusted-public-key "$agent/public.pem" --key-id installer-test | grep -F node-agent
    "$rz" --json install-status --destination "$base/agent" | grep -F true
    test -x "$base/agent/current/bin/rz-monitor-agent"
    test ! -e "$base/agent/current/web"
    test ! -e "$base/.agent-install.journal"
    test -L "$base/rz/current"
    test ! -e "$base/rz/state/continuation.json"
    test ! -e "$base/.rz-install.journal"
    test "$(stat -c %a "$base/rz/releases" "$base/rz/trust" "$base/rz/state" | grep -Fx 750 | wc -l)" -eq 3
    test "$(stat -c %a "$base/rz/releases"/*/release-manifest.json "$base/rz/releases"/*/signature-envelope.json | grep -Fx 640 | wc -l)" -eq 2
    "$rz" --json apply --destination "$base/rz" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    reject_terminal() {
      destination="$1"
      before=$(find "$destination" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)
      if "$rz" --json apply --destination "$destination" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test; then exit 1; fi
      test "$before" = "$(find "$destination" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)"
    }
    terminal_build=$(readlink "$base/rz/current" | cut -d/ -f2)
    for mutation in missing extra truncated replaced; do
      terminal="$base/terminal-$mutation"
      cp -a "$base/rz" "$terminal"
      payload=$(find "$terminal/releases" -path "*/payload/*" -type f | head -n1)
      case "$mutation" in
        missing) rm "$payload" ;;
        extra) touch "$terminal/releases/$terminal_build/payload/extra" ;;
        truncated) : > "$payload" ;;
        replaced) printf replacement > "$payload" ;;
      esac
      reject_terminal "$terminal"
    done
    copied="$base/terminal-copied-marker"
    mkdir -m 700 "$copied"
    mkdir -m 750 "$copied/releases" "$copied/state" "$copied/trust"
    cp "$base/rz/state/publication-marker.json" "$copied/state/publication-marker.json"
    chmod 640 "$copied/state/publication-marker.json"
    ln -s "releases/$terminal_build/payload" "$copied/current"
    reject_terminal "$copied"
    touch "$base/occupied"
    ! "$rz" --json apply --destination "$base/occupied" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ln -s nowhere "$base/dangling"
    ! "$rz" --json apply --destination "$base/dangling" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    mkdir "$base/writable"; chmod 777 "$base/writable"
    ! "$rz" --json apply --destination "$base/writable/rz" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    mkdir "$base/normal"
    ! "$rz" --json apply --destination "$base/normal/../normalized" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    for mutation in duplicate omitted path header checksum-width padding trailing; do
      bad="target/installer-fixtures/$mutation"
      ! "$rz" --json verify --archive "$bad/archive.tar" --manifest "$bad/release-manifest.json" --envelope "$bad/signature-envelope.json" --trusted-public-key "$bad/public.pem" --key-id installer-test
    done
    for mutation in build-id archive-hash manifest-hash key-id; do
      bad="target/installer-fixtures/envelope-$mutation"
      ! "$rz" --json verify --archive "$bad/archive.tar" --manifest "$bad/release-manifest.json" --envelope "$bad/signature-envelope.json" --trusted-public-key "$bad/public.pem" --key-id installer-test
    done
    for mutation in capabilities config-owners binary-digest api-digest schema-owner data-owner web-digest native-digest protocol-digest selection-digest; do
      bad="target/installer-fixtures/manifest-$mutation"
      ! "$rz" --json verify --archive "$bad/archive.tar" --manifest "$bad/release-manifest.json" --envelope "$bad/signature-envelope.json" --trusted-public-key "$bad/public.pem" --key-id installer-test
    done
    for mutation in artifact-class unknown-top-level unknown-owner-field; do
      bad="target/installer-fixtures/schema-$mutation"
      ! "$rz" --json verify --archive "$bad/archive.tar" --manifest "$bad/release-manifest.json" --envelope "$bad/signature-envelope.json" --trusted-public-key "$bad/public.pem" --key-id installer-test
    done
    bad=target/installer-fixtures/agent-bad
    ! "$rz" --json verify --archive "$bad/archive.tar" --manifest "$bad/release-manifest.json" --envelope "$bad/signature-envelope.json" --trusted-public-key "$bad/public.pem" --key-id installer-test
    ln -s "$fixture/archive.tar" "$base/link.tar"
    ! "$rz" --json verify --archive "$base/link.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ln -s "$fixture/release-manifest.json" "$base/link-manifest.json"
    ln -s "$fixture/signature-envelope.json" "$base/link-envelope.json"
    ln -s "$fixture/public.pem" "$base/link-key.pem"
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$base/link-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$base/link-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$base/link-key.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$x86/public.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id other-key
    printf invalid > "$base/bad.pem"
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$base/bad.pem" --key-id installer-test
    mkdir "$base/not-a-file"
    ! "$rz" --json verify --archive "$base/not-a-file" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$base/not-a-file" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$base/not-a-file" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    ! "$rz" --json verify --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$base/not-a-file" --key-id installer-test
    ! env RUSTZEN_INSTALLER_FAULT_AFTER_PUBLISH=1 "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    test ! -e "$base/fault"
    journal="$base/.fault-install.journal"
    test -f "$journal"
    cp "$journal" "$base/fault-journal.before"
    nonce=$(grep -o "\"workRootNonce\":\"[0-9a-f]\\{32\\}\"" "$journal" | cut -d\" -f4)
    test -n "$nonce"
    work="$base/.fault-install-$nonce"
    test -f "$work/state/continuation.json"
    mv "$journal" "$base/fault-journal.link-target"
    ln -s "$base/fault-journal.link-target" "$journal"
    ! "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    rm "$journal"
    mv "$base/fault-journal.link-target" "$journal"
    chmod 0644 "$journal"
    ! "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    chmod 0600 "$journal"
    chown 65534:65534 "$journal"
    ! "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    chown 0:0 "$journal"
    ! "$rz" --json apply --destination "$base/fault" --archive "$other/archive.tar" --manifest "$other/release-manifest.json" --envelope "$other/signature-envelope.json" --trusted-public-key "$other/public.pem" --key-id installer-test
    cmp "$journal" "$base/fault-journal.before"
    test ! -e "$base/fault"
    mv "$work" "$base/fault-work-original"
    mkdir -m 700 "$work"
    ! "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    cmp "$journal" "$base/fault-journal.before"
    rmdir "$work"
    mv "$base/fault-work-original" "$work"
    mv "$work" "$base/fault-work-original"
    mkdir -m 700 "$work"
    touch "$work/sentinel"
    ! "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    cmp "$journal" "$base/fault-journal.before"
    rm -rf "$work"
    mv "$base/fault-work-original" "$work"
    mv "$work" "$base/fault-work-original"
    mkdir -m 700 "$work"
    mkdir -m 700 "$work/state"
    printf "{\"nonce\":\"00000000000000000000000000000000\",\"version\":1}" > "$work/state/continuation.json"
    chmod 600 "$work/state/continuation.json"
    ! "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    cmp "$journal" "$base/fault-journal.before"
    rm -rf "$work"
    mv "$base/fault-work-original" "$work"
    "$rz" --json apply --destination "$base/fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    test -L "$base/fault/current"
    test ! -e "$base/fault/state/continuation.json"
    test ! -e "$base/.fault-install.journal"
    ! env RUSTZEN_INSTALLER_FAULT_AFTER_RENAME=1 "$rz" --json apply --destination "$base/rename-fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    test -L "$base/rename-fault/current"
    test -f "$base/rename-fault/state/continuation.json"
    rename_journal="$base/.rename-fault-install.journal"
    test -f "$rename_journal"
    cp "$rename_journal" "$base/rename-fault-journal.before"
    ! "$rz" --json apply --destination "$base/rename-fault" --archive "$other/archive.tar" --manifest "$other/release-manifest.json" --envelope "$other/signature-envelope.json" --trusted-public-key "$other/public.pem" --key-id installer-test
    cmp "$rename_journal" "$base/rename-fault-journal.before"
    state="$base/rename-fault/state"
    mv "$state" "$base/rename-fault-state"
    ln -s "$base/rename-fault-state" "$state"
    ! "$rz" --json apply --destination "$base/rename-fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    rm "$state"
    mv "$base/rename-fault-state" "$state"
    "$rz" --json apply --destination "$base/rename-fault" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    test ! -e "$base/rename-fault/state/continuation.json"
    test ! -e "$rename_journal"
    ! env RUSTZEN_INSTALLER_FAULT_AFTER_WORK_MARKER_CLEANUP=1 "$rz" --json apply --destination "$base/cleanup-marker" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    test -L "$base/cleanup-marker/current"
    test ! -e "$base/cleanup-marker/state/continuation.json"
    cleanup_marker_journal="$base/.cleanup-marker-install.journal"
    test -f "$cleanup_marker_journal"
    cleanup_marker_digest=$(find "$base/cleanup-marker" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)
    ! "$rz" --json apply --destination "$base/cleanup-marker" --archive "$other/archive.tar" --manifest "$other/release-manifest.json" --envelope "$other/signature-envelope.json" --trusted-public-key "$other/public.pem" --key-id installer-test
    test "$cleanup_marker_digest" = "$(find "$base/cleanup-marker" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)"
    "$rz" --json apply --destination "$base/cleanup-marker" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    test "$cleanup_marker_digest" = "$(find "$base/cleanup-marker" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)"
    test ! -e "$base/cleanup-marker/state/continuation.json"
    test ! -e "$cleanup_marker_journal"
    ! env RUSTZEN_INSTALLER_FAULT_AFTER_JOURNAL_CLEANUP=1 "$rz" --json apply --destination "$base/cleanup-journal" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
    test -L "$base/cleanup-journal/current"
    test ! -e "$base/cleanup-journal/state/continuation.json"
    cleanup_journal="$base/.cleanup-journal-install.journal"
    test ! -e "$cleanup_journal"
    cleanup_journal_digest=$(find "$base/cleanup-journal" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)
    ! "$rz" --json apply --destination "$base/cleanup-journal" --archive "$other/archive.tar" --manifest "$other/release-manifest.json" --envelope "$other/signature-envelope.json" --trusted-public-key "$other/public.pem" --key-id installer-test
    test "$cleanup_journal_digest" = "$(find "$base/cleanup-journal" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)"
    "$rz" --json apply --destination "$base/cleanup-journal" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test | grep -F true
    test "$cleanup_journal_digest" = "$(find "$base/cleanup-journal" -type f -exec sha256sum {} + | LC_ALL=C sort | sha256sum | cut -d" " -f1)"
    test ! -e "$base/cleanup-journal/state/continuation.json"
    test ! -e "$cleanup_journal"
  '
