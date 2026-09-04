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
    if "$rz" --json pin-monitor-controller --manifest "$server/release-manifest.json" --envelope "$server/signature-envelope.json" --trusted-public-key "$server/public.pem" --key-id installer-test --controller-endpoint https://monitor.example >"$base/locked-root.out" 2>&1; then
      echo "pin unexpectedly accepted an unavailable Agent root" >&2; exit 1
    fi
    grep -F "Agent root is unavailable" "$base/locked-root.out"
    mv "$locked/agent" "$base/rz"; rmdir "$locked"
    agent_manifest=$(find "$base/rz/releases" -name release-manifest.json -type f -print -quit)
    config_source="$base/agent-config.env"
    printf "RUSTZEN_ENV=production\\nRUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder\\nRUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example\\nRUSTZEN_MONITOR_NODE_ID=fixture-agent\\n" > "$config_source"
    chmod 0600 "$config_source"
    recorder="$base/systemctl-recorder"
    recorder_log="$base/systemctl.log"
    cat > "$recorder" <<EOF
#!/bin/sh
echo "\$*" >> $recorder_log
case "\$1" in
  daemon-reload) : ;;
  enable|start) test "\$2" = rz-monitor-agent.service ;;
  --no-block) test "\$2" = start && test "\$3" = rz-monitor-agent.service ;;
  *) exit 1 ;;
esac
stage="\$1"; test "\$stage" != --no-block || stage="\$2"
test "\${RUSTZEN_SYSTEMCTL_FAIL:-}" != "\$stage"
EOF
    chmod 0700 "$recorder"
    if ! RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" >"$base/activate.out" 2>&1; then
      cat "$base/activate.out" >&2; exit 1
    fi
    grep -F true "$base/activate.out"
    ! grep -F fixture-token-is-not-a-placeholder "$base/activate.out"
    cat > "$base/systemctl.expected" <<EOF
daemon-reload
enable rz-monitor-agent.service
--no-block start rz-monitor-agent.service
EOF
    cmp "$base/systemctl.expected" "$recorder_log"
    test "$(stat -c "%u:%g:%a" "$base/rz/config/rz-monitor-agent.env")" = 0:2345:640
    cmp "$base/rz/current/systemd/rz-monitor-agent.service" /etc/systemd/system/rz-monitor-agent.service
    marker="$base/rz/state/monitor-agent-activation.json"
    test "$(stat -c "%u:%g:%a" "$marker")" = 0:0:600
    grep -F "\"state\":\"published_and_start_queued\"" "$marker"
    snapshot_path() {
      value="$1"
      if [ ! -e "$value" ] && [ ! -L "$value" ]; then
        printf absent
      elif [ -f "$value" ] && [ ! -L "$value" ]; then
        printf "regular:%s:%s" "$(sha256sum "$value" | cut -d" " -f1)" "$(stat -c "%u:%g:%a:%i" "$value")"
      else
        printf "nonregular:%s:%s" "$(stat -c "%F" "$value")" "$(stat -c "%u:%g:%a:%i" "$value")"
      fi
    }
    activation_snapshot() {
      printf "%s|%s|%s|%s|%s" "$(snapshot_path "$profile")" "$(snapshot_path "$base/rz/config")" "$(snapshot_path "$base/rz/config/rz-monitor-agent.env")" "$(snapshot_path /etc/systemd/system/rz-monitor-agent.service)" "$(snapshot_path "$marker")"
    }
    activation_negative() {
      case_name="$1"; expected="$2"; source="$3"
      before="$(activation_snapshot)"
      log_before="$(snapshot_path "$recorder_log")"
      if RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$source" >"$base/activation-$case_name.out" 2>&1; then
        echo "negative activation case unexpectedly succeeded: $case_name" >&2; exit 1
      fi
      grep -F "$expected" "$base/activation-$case_name.out"
      ! grep -F fixture-token "$base/activation-$case_name.out"
      test "$before" = "$(activation_snapshot)"
      test "$log_before" = "$(snapshot_path "$recorder_log")"
      for temp_parent in "$base/rz/config" "$base/rz/state" /etc/systemd/system; do
        [ ! -d "$temp_parent" ] || test -z "$(find "$temp_parent" -maxdepth 1 -name ".rz-publish-*" -print -quit)"
      done
    }
    write_source() {
      destination="$1"; token="$2"; endpoint="$3"; node="$4"
      printf "RUSTZEN_ENV=production\\nRUSTZEN_MONITOR_AGENT_TOKEN=%s\\nRUSTZEN_MONITOR_CONTROLLER_URL=%s\\nRUSTZEN_MONITOR_NODE_ID=%s\\n" "$token" "$endpoint" "$node" > "$destination"
      chmod 0600 "$destination"
    }
    chmod 0644 "$config_source"
    activation_negative source-0644 "Agent config source is unsafe" "$config_source"
    chmod 0600 "$config_source"
    ln -s "$config_source" "$base/source-link.env"
    activation_negative source-link "Agent config source is unavailable" "$base/source-link.env"
    mkdir "$base/unsafe-source"; chmod 0777 "$base/unsafe-source"
    cp "$config_source" "$base/unsafe-source/config.env"; chmod 0600 "$base/unsafe-source/config.env"
    activation_negative source-parent "Agent config source parent is unsafe" "$base/unsafe-source/config.env"
    activation_negative source-relative "Agent config source must be absolute" "opt/agent-config.env"
    write_source "$base/bad-token.env" placeholder https://monitor.example fixture-agent
    activation_negative token-placeholder "Agent config source is invalid" "$base/bad-token.env"
    write_source "$base/bad-token-chars.env" "fixture token with spaces" https://monitor.example fixture-agent
    activation_negative token-characters "Agent config source is invalid" "$base/bad-token-chars.env"
    write_source "$base/bad-node.env" fixture-token-is-not-a-placeholder https://monitor.example "bad/node"
    activation_negative node "Agent config source is invalid" "$base/bad-node.env"
    write_source "$base/http-loopback.env" fixture-token-is-not-a-placeholder http://127.0.0.1:9801 fixture-agent
    activation_negative http-loopback "Agent config source is invalid" "$base/http-loopback.env"
    write_source "$base/endpoint-mismatch.env" fixture-token-is-not-a-placeholder https://other.example fixture-agent
    activation_negative endpoint-mismatch "Agent config endpoint differs from Controller profile" "$base/endpoint-mismatch.env"
    cp "$profile" "$base/profile.good"
    printf x >> "$profile"
    activation_negative profile-json "controller profile is invalid JSON" "$config_source"
    cp "$base/profile.good" "$profile"; chown 0:2345 "$profile"; chmod 0640 "$profile"
    mutate_profile_hash() {
      field="$1"; replacement="$2"
      sed -i "s/\"$field\":\"[0-9a-f]\{64\}\"/\"$field\":\"$replacement\"/" "$profile"
    }
    mutate_profile_hash agentBuildId "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    activation_negative profile-agent-build "Agent profile differs from retained Agent binding" "$config_source"
    cp "$base/profile.good" "$profile"; chown 0:2345 "$profile"; chmod 0640 "$profile"
    mutate_profile_hash agentManifestSha256 "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    activation_negative profile-agent-manifest "Agent profile differs from retained Agent binding" "$config_source"
    cp "$base/profile.good" "$profile"; chown 0:2345 "$profile"; chmod 0640 "$profile"
    mutate_profile_hash protocolId "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    activation_negative profile-protocol "Agent profile differs from retained Agent binding" "$config_source"
    cp "$base/profile.good" "$profile"; chown 0:2345 "$profile"; chmod 0640 "$profile"
    mutate_profile_hash controllerBuildId x
    activation_negative profile-field "controller profile fields are invalid" "$config_source"
    cp "$base/profile.good" "$profile"; chown 0:2345 "$profile"; chmod 0640 "$profile"
    agent_binary="$base/rz/current/bin/rz-monitor-agent"
    cp "$agent_binary" "$base/agent-binary.good"; printf x >> "$agent_binary"
    activation_negative binary-digest "Agent binary digest differs from retained manifest" "$config_source"
    cp "$base/agent-binary.good" "$agent_binary"; chown 0:2345 "$agent_binary"; chmod 0755 "$agent_binary"
    source_unit="$base/rz/current/systemd/rz-monitor-agent.service"
    cp "$source_unit" "$base/source-unit.good"; printf x >> "$source_unit"
    activation_negative unit-digest "Agent native unit differs from manifest" "$config_source"
    cp "$base/source-unit.good" "$source_unit"; chown 0:0 "$source_unit"; chmod 0644 "$source_unit"
    installed_config="$base/rz/config/rz-monitor-agent.env"
    cp "$installed_config" "$base/installed-config.good"; printf x >> "$installed_config"
    activation_negative config-conflict "activation destination differs from requested tuple" "$config_source"
    cp "$base/installed-config.good" "$installed_config"; chown 0:2345 "$installed_config"; chmod 0640 "$installed_config"
    rm "$installed_config"; ln -s nowhere "$installed_config"
    activation_negative config-symlink "activation destination is unsafe" "$config_source"
    rm "$installed_config"; cp "$base/installed-config.good" "$installed_config"; chown 0:2345 "$installed_config"; chmod 0640 "$installed_config"
    cp /etc/systemd/system/rz-monitor-agent.service "$base/installed-unit.good"; printf x >> /etc/systemd/system/rz-monitor-agent.service
    activation_negative unit-conflict "activation destination differs from requested tuple" "$config_source"
    cp "$base/installed-unit.good" /etc/systemd/system/rz-monitor-agent.service; chown 0:0 /etc/systemd/system/rz-monitor-agent.service; chmod 0644 /etc/systemd/system/rz-monitor-agent.service
    rm /etc/systemd/system/rz-monitor-agent.service; ln -s nowhere /etc/systemd/system/rz-monitor-agent.service
    activation_negative unit-symlink "activation destination is unsafe" "$config_source"
    rm /etc/systemd/system/rz-monitor-agent.service; cp "$base/installed-unit.good" /etc/systemd/system/rz-monitor-agent.service; chown 0:0 /etc/systemd/system/rz-monitor-agent.service; chmod 0644 /etc/systemd/system/rz-monitor-agent.service
    chown 0:0 "$base/rz/config"; chmod 0700 "$base/rz/config"
    activation_negative config-directory "Agent config directory ownership or mode is invalid" "$config_source"
    chown 0:2345 "$base/rz/config"; chmod 0750 "$base/rz/config"
    runuser -u rz-monitor-agent -- cat "$installed_config" >/dev/null
    rm "$installed_config"
    activation_negative missing-config "activated Agent destination is unavailable" "$config_source"
    cp "$base/installed-config.good" "$installed_config"; chown 0:2345 "$installed_config"; chmod 0640 "$installed_config"
    rm "$installed_config"; rmdir "$base/rz/config"
    activation_negative missing-config-directory "activated Agent config directory is unavailable" "$config_source"
    mkdir "$base/rz/config"; chown 0:2345 "$base/rz/config"; chmod 0750 "$base/rz/config"
    cp "$base/installed-config.good" "$installed_config"; chown 0:2345 "$installed_config"; chmod 0640 "$installed_config"
    rm /etc/systemd/system/rz-monitor-agent.service
    activation_negative missing-unit "activated Agent destination is unavailable" "$config_source"
    cp "$base/installed-unit.good" /etc/systemd/system/rz-monitor-agent.service; chown 0:0 /etc/systemd/system/rz-monitor-agent.service; chmod 0644 /etc/systemd/system/rz-monitor-agent.service
    rm "$marker"
    for failed_stage in daemon-reload enable start; do
      : > "$recorder_log"
      before="$(activation_snapshot)"
      if RUSTZEN_SYSTEMCTL_FAIL="$failed_stage" RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" >"$base/fail-$failed_stage.out" 2>&1; then exit 1; fi
      grep -F "systemctl activation failed" "$base/fail-$failed_stage.out"
      ! grep -F fixture-token "$base/fail-$failed_stage.out"
      test ! -e "$marker"
      test "$before" = "$(activation_snapshot)"
      : > "$recorder_log"
      RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" | grep -F true
      cmp "$base/systemctl.expected" "$recorder_log"
      test -f "$marker"
      rm "$marker"
    done
    : > "$recorder_log"
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" >"$base/same-activation-1.out" 2>&1 & first=$!
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" >"$base/same-activation-2.out" 2>&1 & second=$!
    wait "$first"; wait "$second"
    cmp "$base/systemctl.expected" "$recorder_log"
    grep -F true "$base/same-activation-1.out"; grep -F true "$base/same-activation-2.out"
    : > "$recorder_log"
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" | grep -F true
    test ! -s "$recorder_log"
    write_source "$base/alternate-config.env" fixture-token-is-another-valid-value https://monitor.example alternate-agent
    rm "$marker" "$installed_config"
    : > "$recorder_log"
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" >"$base/different-activation-1.out" 2>&1 & first=$!
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$base/alternate-config.env" >"$base/different-activation-2.out" 2>&1 & second=$!
    wait "$first" && first_status=0 || first_status=$?
    wait "$second" && second_status=0 || second_status=$?
    if ! { { test "$first_status" = 0 && test "$second_status" != 0; } || { test "$first_status" != 0 && test "$second_status" = 0; }; }; then
      cat "$base/different-activation-1.out" "$base/different-activation-2.out" "$recorder_log" >&2; exit 1
    fi
    if [ "$first_status" = 0 ]; then winner_source="$config_source"; loser_output="$base/different-activation-2.out"; else winner_source="$base/alternate-config.env"; loser_output="$base/different-activation-1.out"; fi
    grep -E "activation destination differs from requested tuple|activation marker differs from requested tuple" "$loser_output" || { cat "$loser_output" >&2; exit 1; }
    ! grep -F fixture-token "$base/different-activation-1.out" "$base/different-activation-2.out"
    cmp "$winner_source" "$installed_config" || { cat "$winner_source" "$installed_config" >&2; exit 1; }
    cmp "$base/systemctl.expected" "$recorder_log" || { cat "$recorder_log" >&2; exit 1; }
    rm "$marker" "$installed_config"
    : > "$recorder_log"
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" | grep -F true
    cmp "$base/systemctl.expected" "$recorder_log"
    mkdir -p /var/lib/rustzen-monitor-agent
    chown 2345:2345 /var/lib/rustzen-monitor-agent
    chmod 0750 /var/lib/rustzen-monitor-agent
    setsid runuser -u rz-monitor-agent -- env RUSTZEN_ENV=production RUSTZEN_RUNTIME_ROOT=/var/lib/rustzen-monitor-agent RUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder RUSTZEN_MONITOR_NODE_ID=fixture-agent RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example "$agent_binary" >"$base/runtime.out" 2>&1 & runtime_pid=$!
    sleep 10
    if ! kill -0 "$runtime_pid"; then cat "$base/runtime.out" >&2; exit 1; fi
    kill -TERM -- "-$runtime_pid" 2>/dev/null || true
    sleep 1
    kill -KILL -- "-$runtime_pid" 2>/dev/null || true
    wait "$runtime_pid" 2>/dev/null || true
    if [ ! -d /var/lib/rustzen-monitor-agent/logs ] || [ -z "$(find /var/lib/rustzen-monitor-agent/logs -type f -print -quit)" ]; then
      cat "$base/runtime.out" >&2; find /var/lib/rustzen-monitor-agent -maxdepth 3 -ls >&2; exit 1
    fi
    rm "$profile" "$marker" "$installed_config"
    pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://localhost:4443 | grep -F true
    write_source "$base/d55c-config.env" fixture-token-is-not-a-placeholder https://localhost:4443 fixture-agent
    : > "$recorder_log"
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$base/d55c-config.env" | grep -F true
    cmp "$base/systemctl.expected" "$recorder_log"
    openssl req -x509 -newkey rsa:2048 -nodes -days 1 -subj "/CN=RustZen D55c CA" -addext basicConstraints=critical,CA:TRUE -addext keyUsage=critical,keyCertSign,cRLSign -keyout "$base/d55c-ca.key" -out "$base/d55c-ca.crt" >/dev/null 2>&1
    openssl req -newkey rsa:2048 -nodes -subj /CN=localhost -keyout "$base/d55c.key" -out "$base/d55c.csr" >/dev/null 2>&1
    printf "basicConstraints=critical,CA:FALSE\nkeyUsage=critical,digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\nsubjectAltName=DNS:localhost\n" > "$base/d55c.ext"
    openssl x509 -req -in "$base/d55c.csr" -CA "$base/d55c-ca.crt" -CAkey "$base/d55c-ca.key" -CAcreateserial -days 1 -extfile "$base/d55c.ext" -out "$base/d55c.crt" >/dev/null 2>&1
    cp "$base/d55c-ca.crt" /usr/local/share/ca-certificates/rz-d55c.crt
    update-ca-certificates >/dev/null
    PYTHONDONTWRITEBYTECODE=1 python3 scripts/monitor-agent-pairing-fixture.py --agent "$agent_binary" --certificate "$base/d55c.crt" --private-key "$base/d55c.key" --runtime-root /var/lib/rustzen-monitor-agent --evidence "$base/d55c.jsonl" --endpoint https://localhost:4443 --port 4443
    test "$(wc -l < "$base/d55c.jsonl")" = 12
    ! grep -F fixture-token-is-not-a-placeholder "$base/d55c.jsonl"
    grep -F "\"case\": \"accepted\", \"event\": \"readiness\", \"payload\": \"READY=1\"" "$base/d55c.jsonl"
    grep -F "\"case\": \"duplicate\", \"event\": \"readiness\", \"payload\": \"READY=1\"" "$base/d55c.jsonl"
    grep -F "\"case\": \"unauthorized\", \"event\": \"readiness\", \"payload\": null" "$base/d55c.jsonl"
    grep -F "\"case\": \"drop\", \"event\": \"readiness\", \"payload\": null" "$base/d55c.jsonl"
    rm "$profile" "$marker" "$installed_config"
    pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example | grep -F true
    : > "$recorder_log"
    RUSTZEN_SYSTEMCTL_RECORDER="$recorder" "$rz" --json activate-monitor-agent --config "$config_source" | grep -F true
    cmp "$base/systemctl.expected" "$recorder_log"
    runuser -u rz-monitor-agent -- cat "$profile" | grep -F controllerBuildId
    ! runuser -u rz-monitor-agent -- sh -c "echo x >> $profile"
    if runuser -u rz-monitor-agent -- env RUSTZEN_ENV=production RUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder RUSTZEN_MONITOR_NODE_ID=fixture-agent RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example target/agent-pair-linux/debug/rz-monitor-agent >"$base/missing.out" 2>&1; then exit 1; fi
    grep -F "Agent executable is not the published current binary" "$base/missing.out"
    runuser -u rz-monitor-agent -- env RUSTZEN_ENV=production RUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder RUSTZEN_MONITOR_NODE_ID=fixture-agent RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example "$base/rz/current/bin/rz-monitor-agent" contract pairing
    ! runuser -u rz-monitor-agent -- env RUSTZEN_ENV=production RUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder RUSTZEN_MONITOR_NODE_ID=fixture-agent RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example target/agent-pair-linux/debug/rz-monitor-agent contract pairing
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
    ! runuser -u rz-monitor-agent -- env RUSTZEN_ENV=production RUSTZEN_MONITOR_AGENT_TOKEN=fixture-token-is-not-a-placeholder RUSTZEN_MONITOR_NODE_ID=fixture-agent RUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.example "$base/rz/current/bin/rz-monitor-agent" contract pairing >"$base/corrupt.out" 2>&1
    grep -F "controller profile is invalid JSON" "$base/corrupt.out"
    test "$before_corrupt" = "$(snapshot_profile)"
    negative_absent bad-endpoint "Controller endpoint is invalid" "$server/release-manifest.json" "$server/signature-envelope.json" http://monitor.example
    ln -s nowhere "$profile"
    before_link="$(snapshot_profile)"
    if pin_with "$server/release-manifest.json" "$server/signature-envelope.json" https://monitor.example >"$base/profile-link.out" 2>&1; then exit 1; fi
    grep -F "profile destination is unsafe" "$base/profile-link.out"
    test "$before_link" = "$(snapshot_profile)"
  '
