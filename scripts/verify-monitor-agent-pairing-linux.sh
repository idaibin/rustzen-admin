#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
fixtures=target/monitor-agent-pairing-fixtures
rm -rf "$fixtures"
docker run --rm --platform linux/arm64 -v "$root:/work" -w /work \
  -v rustzen-agent-pair-cargo:/usr/local/cargo/registry rust:1.95-bookworm bash -euo pipefail -c '
    cargo build -p rustzen-cli --target-dir target/agent-pair-linux
    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent --target-dir target/agent-pair-linux
  '
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_OUTPUT="$fixtures/server" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_OUTPUT="$fixtures/agent" RUSTZEN_INSTALLER_ARTIFACT=node-agent RUSTZEN_INSTALLER_AGENT_BINARY=target/agent-pair-linux/debug/rz-monitor-agent pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
docker run --rm --platform linux/arm64 -v "$root:/work" -w /work \
  -v rustzen-agent-pair-cargo:/usr/local/cargo/registry rust:1.95-bookworm bash -euo pipefail -c '
    rz=target/agent-pair-linux/debug/rz
    base=/opt
    agent=target/monitor-agent-pairing-fixtures/agent
    server=target/monitor-agent-pairing-fixtures/server
    "$rz" --json apply --destination "$base/rz" --archive "$agent/archive.tar" --manifest "$agent/release-manifest.json" --envelope "$agent/signature-envelope.json" --trusted-public-key "$agent/public.pem" --key-id installer-test | grep -F node-agent
    groupadd -g 2345 rz-monitor-agent
    useradd -u 2345 -g 2345 -M -s /usr/sbin/nologin rz-monitor-agent
    "$rz" --json prepare-monitor-agent-access | grep -F rz-monitor-agent
    profile="$base/rz/controller-profile.json"
    snapshot_profile() {
      if [ ! -e "$profile" ] && [ ! -L "$profile" ]; then
        printf absent
      elif [ -f "$profile" ] && [ ! -L "$profile" ]; then
        printf "regular:%s:%s" "$(sha256sum "$profile" | cut -d" " -f1)" "$(stat -c "%u:%g:%a:%i" "$profile")"
      else
        printf "nonregular:%s:%s" "$(stat -c "%F" "$profile")" "$(stat -c "%u:%g:%a:%i" "$profile")"
      fi
    }
    pin_with() {
      "$rz" --json pin-monitor-controller --manifest "$1" --envelope "$2" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint "$3"
    }
    negative_absent() {
      name="$1"; expected="$2"; manifest="$3"; envelope="$4"; endpoint="$5"
      rm -f "$profile"
      before="$(snapshot_profile)"
      if pin_with "$manifest" "$envelope" "$endpoint" >"$base/$name.out" 2>&1; then
        echo "negative pairing case unexpectedly succeeded: $name" >&2; exit 1
      fi
      grep -F "$expected" "$base/$name.out"
      after="$(snapshot_profile)"
      test "$before" = "$after"
    }
    for stage in create write fsync rename dirsync; do
      before="$(snapshot_profile)"
      if RUSTZEN_PROFILE_PUBLISH_FAULT="$stage" pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example >"$base/fault-$stage.out" 2>&1; then exit 1; fi
      grep -F "debug profile publication fault at $stage" "$base/fault-$stage.out"
      if [ "$stage" = dirsync ]; then
        test -f "$profile"
        test "$(stat -c "%u:%g:%a" "$profile")" = 0:2345:640
        grep -F https://monitor.example "$profile"
        durable="$(snapshot_profile)"; test "$durable" != absent
        pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example | grep -F true
        test "$durable" = "$(snapshot_profile)"
        rm "$profile"
      else
        test "$before" = "$(snapshot_profile)"
      fi
      test -z "$(find "$base/rz" -maxdepth 1 -name ".rz-publish-*" -print -quit)"
    done
    "$rz" --json pin-monitor-controller --manifest "$server/release-manifest.json" --envelope "$server/signature-envelope.json" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint https://monitor.example >"$base/same-1.out" 2>&1 & first=$!
    "$rz" --json pin-monitor-controller --manifest "$server/release-manifest.json" --envelope "$server/signature-envelope.json" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint https://monitor.example >"$base/same-2.out" 2>&1 & second=$!
    wait "$first"; wait "$second"
    test -f "$profile"
    rm "$profile"
    "$rz" --json pin-monitor-controller --manifest "$server/release-manifest.json" --envelope "$server/signature-envelope.json" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint https://monitor.example >"$base/different-1.out" 2>&1 & first=$!
    "$rz" --json pin-monitor-controller --manifest "$server/release-manifest.json" --envelope "$server/signature-envelope.json" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint https://other.example >"$base/different-2.out" 2>&1 & second=$!
    wait "$first" && first_status=0 || first_status=$?
    wait "$second" && second_status=0 || second_status=$?
    test "$first_status" = 0 || test "$second_status" = 0
    test "$first_status" != 0 || test "$second_status" != 0
    if [ "$first_status" = 0 ]; then winner=https://monitor.example; loser="$base/different-2.out"; else winner=https://other.example; loser="$base/different-1.out"; fi
    grep -E "profile publication conflict|profile differs from requested tuple" "$loser"
    grep -F "\"endpoint\":\"$winner\"" "$profile"
    rm "$profile"
    pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example | grep -F true
    test "$(stat -c "%u:%g:%a" "$profile")" = 0:2345:640
    locked=$(mktemp -d /opt/rz-agent-locked.XXXXXX); chmod 0700 "$locked"
    mv "$base/rz" "$locked/agent"
    ! "$rz" --json pin-monitor-controller --manifest "$server/release-manifest.json" --envelope "$server/signature-envelope.json" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint https://monitor.example
    mv "$locked/agent" "$base/rz"; rmdir "$locked"
    agent_manifest=$(find "$base/rz/releases" -name release-manifest.json -type f -print -quit)
    runuser -u rz-monitor-agent -- cat "$profile" | grep -F controllerBuildId
    ! runuser -u rz-monitor-agent -- sh -c "echo x >> $profile"
    if runuser -u rz-monitor-agent -- env RUSTZEN_MONITOR_NODE_ID=proof RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example target/agent-pair-linux/debug/rz-monitor-agent >"$base/missing.out" 2>&1; then exit 1; fi
    grep -F "Agent executable is not the published current binary" "$base/missing.out"
    runuser -u rz-monitor-agent -- env RUSTZEN_MONITOR_NODE_ID=proof RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example "$base/rz/current/bin/rz-monitor-agent" contract pairing
    ! runuser -u rz-monitor-agent -- env RUSTZEN_MONITOR_NODE_ID=proof RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example target/agent-pair-linux/debug/rz-monitor-agent contract pairing
    cp "$server/signature-envelope.json" "$base/bad-envelope.json"; printf x >> "$base/bad-envelope.json"
    negative_absent bad-envelope "envelope is invalid JSON" "$server/release-manifest.json" "$base/bad-envelope.json" https://monitor.example
    cp "$server/signature-envelope.json" "$base/bad-signature.json"
    signature_replacement() { case "$1" in A) printf B ;; *) printf A ;; esac; }
    test "$(signature_replacement A)" = B
    test "$(signature_replacement B)" = A
    signature_first="$(sed -n "s/.*\"signature\":\"\\([^\"]\\).*/\\1/p" "$base/bad-signature.json")"
    test -n "$signature_first"
    signature_next="$(signature_replacement "$signature_first")"
    if [ "$signature_first" = A ]; then
      sed -i "s/\"signature\":\"A/\"signature\":\"$signature_next/" "$base/bad-signature.json"
    else
      sed -i "s/\"signature\":\"./\"signature\":\"$signature_next/" "$base/bad-signature.json"
    fi
    ! cmp -s "$server/signature-envelope.json" "$base/bad-signature.json"
    negative_absent bad-signature "signature invalid" "$server/release-manifest.json" "$base/bad-signature.json" https://monitor.example
    cp "$server/release-manifest.json" "$base/bad-tuple.json"
    sed -i "0,/\"buildId\":\"[0-9a-f]/{s/\"buildId\":\"[0-9a-f]/\"buildId\":\"f/}" "$base/bad-tuple.json"
    negative_absent bad-tuple "Controller manifest and envelope tuple differs" "$base/bad-tuple.json" "$server/signature-envelope.json" https://monitor.example
    cp "$agent_manifest" "$agent_manifest.bak"
    sed -i "s/80257e2cddb90cca5c857d730d00b05aef1c24e18d27b7cf4fdb9f3225708928/ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff/" "$agent_manifest"
    negative_absent bad-protocol "Agent manifest does not exactly match Controller protocol" "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example
    mv "$agent_manifest.bak" "$agent_manifest"
    pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example | grep -F true
    printf x >> "$profile"
    before_corrupt="$(snapshot_profile)"
    ! runuser -u rz-monitor-agent -- env RUSTZEN_MONITOR_NODE_ID=proof RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example "$base/rz/current/bin/rz-monitor-agent" contract pairing >"$base/corrupt.out" 2>&1
    grep -F "controller profile is invalid JSON" "$base/corrupt.out"
    test "$before_corrupt" = "$(snapshot_profile)"
    negative_absent bad-endpoint "Controller endpoint is invalid" "$server/release-manifest.json" "$server/signature-envelope.json" http://monitor.example
    ln -s nowhere "$profile"
    before_link="$(snapshot_profile)"
    if pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example >"$base/profile-link.out" 2>&1; then exit 1; fi
    grep -F "profile destination is unsafe" "$base/profile-link.out"
    test "$before_link" = "$(snapshot_profile)"
  '
