#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

version=$(awk -F '"' '/^version = / { print $2; exit }' Cargo.toml)
bundle=${RUSTZEN_FULL_BUNDLE:-target/rz/rz-${version}-x86_64.tar}
test -f "$bundle" || { echo "missing full bundle: $bundle; run just build-release" >&2; exit 1; }
private_key="$(node scripts/deploy-sign.mjs ensure-key)"
verify_key="$(node scripts/deploy-sign.mjs public-key)"
fixtures=target/monitor-agent-pairing-fixtures
rm -rf "$fixtures"

docker run --rm --platform linux/amd64 -e RUSTZEN_DEPLOY_VERIFY_KEY="$verify_key" -v "$root:/work" -w /work \
  -v rustzen-agent-pair-cargo:/usr/local/cargo/registry rust:1.95-bookworm bash -euo pipefail -c '
    cargo build -p rustzen-cli --target-dir target/agent-pair-linux
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent --target-dir target/agent-pair-linux
  ' RUSTZEN_DEPLOY_VERIFY_KEY="$verify_key"

RUSTZEN_INSTALLER_TARGET=x86_64-unknown-linux-musl \
RUSTZEN_INSTALLER_OUTPUT="$fixtures/agent" \
RUSTZEN_INSTALLER_ARTIFACT=node-agent \
RUSTZEN_INSTALLER_AGENT_BINARY=target/agent-pair-linux/debug/rz-monitor-agent \
RUSTZEN_INSTALLER_PRIVATE_KEY_FILE="$private_key" \
RUSTZEN_INSTALLER_KEY_ID=rustzen-release \
  pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts

docker run --rm --platform linux/amd64 -v "$root:/work" -w /work \
  -v rustzen-agent-pair-cargo:/usr/local/cargo/registry rust:1.95-bookworm bash -euo pipefail -c '
    rz=target/agent-pair-linux/debug/rz
    cp -R target/monitor-agent-pairing-fixtures/agent /tmp/agent
    cp '"$bundle"' /tmp/controller.tar
    agent=/tmp/agent
    bundle=/tmp/controller.tar
    expected_version='"$version"'
    apply_out=$("$rz" --json install-agent --destination /opt/rz --archive "$agent/archive.tar" --manifest "$agent/release-manifest.json" --envelope "$agent/signature-envelope.json") || { printf "%s\n" "$apply_out" >&2; exit 1; }
    printf "%s\n" "$apply_out" | grep -F node-agent
    groupadd -g 2345 rz-monitor-agent
    useradd -u 2345 -g 2345 -M -s /usr/sbin/nologin rz-monitor-agent
    "$rz" --json prepare-monitor-agent-access | grep -F rz-monitor-agent
    "$rz" --json pin-monitor-controller --bundle "$bundle" --controller-endpoint https://monitor.example | grep -F controller-profile.json
    profile=/opt/rz/controller-profile.json
    test "$(stat -c "%u:%g:%a" "$profile")" = 0:2345:640
    grep -F "\"version\":2" "$profile"
    grep -F "\"controllerVersion\":\"$expected_version\"" "$profile"
    printf "RUSTZEN_ENV=production\nRUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder\nRUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example\nRUSTZEN_MONITOR_NODE_ID=fixture-agent\n" >/root/agent.env
    chmod 0400 /root/agent.env
    RUSTZEN_SYSTEMCTL_RECORDER=/usr/bin/true "$rz" --json activate-monitor-agent --config /root/agent.env | grep -F rz-monitor-agent.service
    runuser -u rz-monitor-agent -- env \
      RUSTZEN_ENV=production \
      RUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder \
      RUSTZEN_MONITOR_NODE_ID=fixture-agent \
      RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example \
      /opt/rz/current/bin/rz-monitor-agent contract pairing
    before=$(sha256sum "$profile")
    cp "$bundle" /tmp/tampered.tar
    printf x >> /tmp/tampered.tar
    if "$rz" --json pin-monitor-controller --bundle /tmp/tampered.tar --controller-endpoint https://monitor.example >/tmp/tampered.out 2>&1; then exit 1; fi
    grep -E "signature must terminate|signature verification failed|metadata does not match" /tmp/tampered.out
    test "$before" = "$(sha256sum "$profile")"
    if "$rz" --json pin-monitor-controller --bundle "$bundle" --controller-endpoint http://monitor.example >/tmp/endpoint.out 2>&1; then exit 1; fi
    grep -F "Controller endpoint is invalid" /tmp/endpoint.out
    test "$before" = "$(sha256sum "$profile")"
  '

echo "full Controller and standalone Agent pairing gate PASS"
