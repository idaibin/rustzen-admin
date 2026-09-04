#!/usr/bin/env bash
set -euo pipefail
repository="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repository"
fixtures=target/installer-fixtures
rm -rf "$fixtures"
RUSTZEN_INSTALLER_OUTPUT="$fixtures/x86" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
! RUSTZEN_INSTALLER_OUTPUT="$fixtures/escaped" RUSTZEN_INSTALLER_KEY_ID=../../escaped pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_OUTPUT="$fixtures/valid" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
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
    test -L "$base/rz/current"
    test "$(stat -c %a "$base/rz/releases" "$base/rz/trust" "$base/rz/state" | grep -Fx 750 | wc -l)" -eq 3
    test "$(stat -c %a "$base/rz/releases"/*/release-manifest.json "$base/rz/releases"/*/signature-envelope.json | grep -Fx 640 | wc -l)" -eq 2
    ! "$rz" --json apply --destination "$base/rz" --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
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
    test -z "$(find "$base" -maxdepth 1 -name ".fault-install-*" -print)"
  '
