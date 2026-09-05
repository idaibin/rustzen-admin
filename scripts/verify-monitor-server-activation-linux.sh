#!/usr/bin/env bash
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
fixtures=target/monitor-server-activation-fixtures
rm -rf "$fixtures"
mkdir -p "$fixtures"
log="$fixtures/evidence.log"
exec > >(tee "$log") 2>&1
docker run --rm --platform linux/arm64 -v "$root:/work" -w /work -v rustzen-monitor-server-cargo:/usr/local/cargo/registry rust:1.95-bookworm bash -euo pipefail -c '
  cargo build --release -p rustzen-cli --target-dir target/monitor-server-linux
  cargo build -p rustzen-cli --target-dir target/monitor-server-linux
  cargo build --release -p rustzen-admin --no-default-features --features monitor-distribution --bin rz-admin --target-dir target/monitor-server-linux
  cargo build --release -p rustzen-monitor --no-default-features --features controller --bin rz-monitor --target-dir target/monitor-server-linux
'
RUSTZEN_INSTALLER_TARGET=aarch64-unknown-linux-gnu RUSTZEN_INSTALLER_OUTPUT="$fixtures/server" RUSTZEN_INSTALLER_ADMIN_BINARY=target/monitor-server-linux/release/rz-admin RUSTZEN_INSTALLER_MONITOR_BINARY=target/monitor-server-linux/release/rz-monitor pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts
name="rz-monitor-server-activation-$$"
image="rz-monitor-server-systemd:bookworm-arm64-v2"
setup="$name-setup"
cleanup() {
  status=$?
  if [ "$status" -ne 0 ] && docker ps -a --format '{{.Names}}' | grep -Fxq "$name"; then
    docker exec "$name" systemctl --no-pager --full status rz-admin.service rz-monitor.service >>"$log" 2>&1 || true
    docker exec "$name" journalctl --no-pager -u rz-admin.service -u rz-monitor.service >>"$log" 2>&1 || true
    docker logs "$name" >>"$log" 2>&1 || true
  fi
  docker rm -f "$name" "$setup" >/dev/null 2>&1 || true
  exit "$status"
}
trap cleanup EXIT
if ! docker image inspect "$image" >/dev/null 2>&1; then
  docker run --name "$setup" --platform linux/arm64 debian:bookworm-slim /bin/bash -euo pipefail -c 'apt-get update >/dev/null; apt-get install -y systemd-sysv curl ca-certificates strace sqlite3 >/dev/null'
  docker commit "$setup" "$image" >/dev/null
  docker rm "$setup" >/dev/null
fi
docker run -d --name "$name" --privileged --cgroupns=host --platform linux/arm64 --tmpfs /run --tmpfs /run/lock -v "$root:/work" -w /work "$image" /sbin/init >/dev/null
for attempt in $(seq 1 30); do docker exec "$name" systemctl is-system-running --wait >/dev/null 2>&1 && break || sleep 1; done
docker exec "$name" /bin/bash -c 'state=$(systemctl is-system-running || true); test "$state" = running -o "$state" = degraded'
docker exec "$name" /bin/bash -euo pipefail -c '
  rz=/work/target/monitor-server-linux/release/rz
  debug_rz=/work/target/monitor-server-linux/debug/rz
  fixture=/work/target/monitor-server-activation-fixtures/server
  "$rz" --json apply --destination /opt/rz --archive "$fixture/archive.tar" --manifest "$fixture/release-manifest.json" --envelope "$fixture/signature-envelope.json" --trusted-public-key "$fixture/public.pem" --key-id installer-test
  install -d -m 0700 /root/rz-activation
  cat > /root/rz-activation/server.env <<EOF
RUSTZEN_ENV=production
RUSTZEN_ADMIN_HOST=127.0.0.1
RUSTZEN_ADMIN_PORT=19801
RUSTZEN_MONITOR_PORT=19802
RUSTZEN_ADMIN_RUNTIME_ROOT=/var/lib/rustzen-admin
RUSTZEN_MONITOR_RUNTIME_ROOT=/var/lib/rustzen-monitor
RUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rustzen-admin/admin.db
RUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rustzen-monitor/monitor.db
RUSTZEN_JWT_SECRET=gate-jwt-secret-7f5d29
RUSTZEN_IPC_TOKEN=gate-ipc-token-9ac412
RUSTZEN_MONITOR_AGENT_TOKEN=gate-agent-token-1b73ef
RUSTZEN_BOOTSTRAP_OWNER_PASSWORD=gate-owner-password-4c819a
EOF
  chmod 0600 /root/rz-activation/server.env
  printf "RUSTZEN_ENV=development\n" > /root/rz-activation/invalid.env
  chmod 0600 /root/rz-activation/invalid.env
  echo "ASSERT failed-no-marker"; ! "$rz" --json activate-monitor-server --config /root/rz-activation/invalid.env
  test ! -e /opt/rz/state/monitor-server-activation.json
  test ! -e /opt/rz/state/monitor-server-database-journal.json
  test ! -e /opt/rz/config/rz-admin.env
  test ! -e /etc/systemd/system/rz-admin.service
  test ! -e /var/lib/rustzen-admin
  ! getent passwd rz-admin >/dev/null
  sed "s/RUSTZEN_ADMIN_PORT=19801/RUSTZEN_ADMIN_PORT=0/" /root/rz-activation/server.env > /root/rz-activation/zero-port.env
  sed "s/RUSTZEN_MONITOR_PORT=19802/RUSTZEN_MONITOR_PORT=19801/" /root/rz-activation/server.env > /root/rz-activation/shared-port.env
  sed "s/gate-owner-password-4c819a/short/" /root/rz-activation/server.env > /root/rz-activation/short-owner.env
  chmod 0600 /root/rz-activation/zero-port.env /root/rz-activation/shared-port.env /root/rz-activation/short-owner.env
  for invalid in zero-port shared-port short-owner; do
    echo "ASSERT invalid-$invalid-no-write"
    ! "$rz" --json activate-monitor-server --config "/root/rz-activation/$invalid.env"
    test ! -e /opt/rz/config/rz-admin.env
    test ! -e /etc/systemd/system/rz-admin.service
    test ! -e /var/lib/rustzen-admin
    ! getent passwd rz-admin >/dev/null
  done
  install -d -m 0700 /root/rz-activation/nested
  echo "ASSERT parent-component-rejected"
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/nested/../server.env
  ! getent passwd rz-admin >/dev/null

  echo "ASSERT unselected-service-state-rejected-before-write"
  cat > /etc/systemd/system/rz-reports.service <<EOF
[Unit]
Description=Foreign Reports residue
[Service]
Type=oneshot
ExecStart=/bin/true
[Install]
WantedBy=multi-user.target
EOF
  install -d -m 0750 /var/lib/rustzen-reports /opt/rz/config
  install -m 0640 /dev/null /opt/rz/config/rz-reports.env
  systemctl daemon-reload
  systemctl enable rz-reports.service
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  ! getent passwd rz-admin >/dev/null
  test ! -e /opt/rz/config/rz-admin.env
  test ! -e /var/lib/rustzen-admin
  systemctl disable rz-reports.service
  rm -f /etc/systemd/system/rz-reports.service /opt/rz/config/rz-reports.env
  rm -rf /var/lib/rustzen-reports
  systemctl daemon-reload
  install -d /run/systemd/system
  cat > /run/systemd/system/rz-reports.service <<EOF
[Unit]
Description=Disabled runtime Reports residue
[Service]
Type=oneshot
ExecStart=/bin/true
EOF
  systemctl daemon-reload
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  ! getent passwd rz-admin >/dev/null
  test ! -e /opt/rz/config/rz-admin.env
  test ! -e /var/lib/rustzen-admin
  rm /run/systemd/system/rz-reports.service
  install -d /usr/local/lib/systemd/system
  cat > /usr/local/lib/systemd/system/rz-insights.service <<EOF
[Unit]
Description=Disabled local Insights residue
[Service]
Type=oneshot
ExecStart=/bin/true
EOF
  systemctl daemon-reload
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  ! getent passwd rz-admin >/dev/null
  test ! -e /opt/rz/config/rz-admin.env
  test ! -e /var/lib/rustzen-admin
  rm /usr/local/lib/systemd/system/rz-insights.service
  systemctl daemon-reload
  install -d -m 0750 /var/lib/rustzen-insights
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  ! getent passwd rz-admin >/dev/null
  test ! -e /opt/rz/config/rz-admin.env
  rm -rf /var/lib/rustzen-insights

  echo "ASSERT shared-service-group-rejected"
  useradd --system --no-create-home --home-dir /nonexistent --shell /usr/sbin/nologin intruder
  groupadd --system rz-admin
  usermod --append --groups rz-admin intruder
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  ! getent passwd rz-admin >/dev/null
  test ! -e /opt/rz/config/rz-admin.env
  test ! -e /var/lib/rustzen-admin
  userdel intruder
  groupdel rz-admin

  echo "ASSERT symlink-runtime-rejected"
  ln -s /tmp /var/lib/rustzen-admin
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  test ! -e /opt/rz/state/monitor-server-activation.json
  rm /var/lib/rustzen-admin

  echo "ASSERT existing-database-rejected"
  install -d -o rz-admin -g rz-admin -m 0750 /var/lib/rustzen-admin
  install -d -o rz-monitor -g rz-monitor -m 0750 /var/lib/rustzen-monitor
  install -o rz-admin -g rz-admin -m 0600 /dev/null /var/lib/rustzen-admin/admin.db
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  test ! -e /opt/rz/state/monitor-server-activation.json
  rm -rf /var/lib/rustzen-admin /var/lib/rustzen-monitor

  reset_activation() {
    systemctl stop rz.target >/dev/null 2>&1 || true
    systemctl disable rz.target >/dev/null 2>&1 || true
    rm -f /etc/systemd/system/rz-admin.service /etc/systemd/system/rz-monitor.service /etc/systemd/system/rz.target
    rm -f /opt/rz/config/rz-admin.env /opt/rz/config/rz-monitor.env /opt/rz/config/rz-release.env
    rm -f /opt/rz/state/monitor-server-activation.json /opt/rz/state/monitor-server-database-journal.json
    rm -rf /var/lib/rustzen-admin /var/lib/rustzen-monitor
    systemctl daemon-reload
  }
  for stage in rename-admin rename-monitor daemon-reload start readiness enable marker; do
    reset_activation
    echo "ASSERT fault-$stage"
    ! RUSTZEN_MONITOR_SERVER_ACTIVATION_FAULT="$stage" "$debug_rz" --json activate-monitor-server --config /root/rz-activation/server.env
    test ! -e /opt/rz/state/monitor-server-activation.json
    ! systemctl is-enabled --quiet rz.target
    ! systemctl is-active --quiet rz.target
    "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
    test ! -e /opt/rz/state/monitor-server-database-journal.json
    systemctl is-enabled --quiet rz.target
  done
  reset_activation
  echo "ASSERT marker-dirsync-reconciled"
  RUSTZEN_MONITOR_SERVER_ACTIVATION_FAULT=marker-dirsync "$debug_rz" --json activate-monitor-server --config /root/rz-activation/server.env
  test -e /opt/rz/state/monitor-server-activation.json
  systemctl is-enabled --quiet rz.target
  reset_activation
  echo "ASSERT journal-remove-retry"
  ! RUSTZEN_MONITOR_SERVER_ACTIVATION_FAULT=journal-remove "$debug_rz" --json activate-monitor-server --config /root/rz-activation/server.env
  test -e /opt/rz/state/monitor-server-activation.json
  test -e /opt/rz/state/monitor-server-database-journal.json
  systemctl is-enabled --quiet rz.target
  systemctl is-active --quiet rz.target
  "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  test ! -e /opt/rz/state/monitor-server-database-journal.json
  echo "ASSERT exact-retry"; "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  sed "s/gate-owner-password-4c819a/gate-owner-password-6b7d21/" /root/rz-activation/server.env > /root/rz-activation/different.env
  chmod 0600 /root/rz-activation/different.env
  echo "ASSERT differing-tuple-rejected"; ! "$rz" --json activate-monitor-server --config /root/rz-activation/different.env
  build_id=$(sed -n "s/.*\"buildId\":\"\([a-f0-9]\{64\\}\)\".*/\1/p" "$fixture/release-manifest.json")
  composition_id=$(sed -n "s/.*\"compositionId\":\"\([a-f0-9]\{64\\}\)\".*/\1/p" "$fixture/release-manifest.json")
  expected_binding="\"selectedBinding\":{\"buildId\":\"$build_id\",\"compositionId\":\"$composition_id\"}"
  echo "ASSERT health-admin"; curl --fail --silent http://127.0.0.1:19801/health | grep -F "$expected_binding"
  echo "ASSERT health-monitor"; curl --fail --silent http://127.0.0.1:19802/health | grep -F "$expected_binding"
  echo "ASSERT all-default-passwords-rejected"
  for account in owner admin viewer; do ! curl --silent --show-error -H "content-type: application/json" -d "{\"username\":\"$account\",\"password\":\"rustzen@123\"}" http://127.0.0.1:19801/api/auth/login | grep -F "token"; done
  echo "ASSERT owner-login"; curl --fail --silent --show-error -H "content-type: application/json" -d "{\"username\":\"owner\",\"password\":\"gate-owner-password-4c819a\"}" http://127.0.0.1:19801/api/auth/login | grep -F "token"
  echo "ASSERT default-password-rejected"; ! curl --silent --show-error -H "content-type: application/json" -d "{\"username\":\"owner\",\"password\":\"rustzen@123\"}" http://127.0.0.1:19801/api/auth/login | grep -F "token"
  echo "ASSERT owner-secret-not-persisted"; ! grep -R -F "gate-owner-password-4c819a" /opt/rz
  echo "ASSERT completed-state-mutation-rejected"
  chmod 0666 /etc/systemd/system/rz-admin.service
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  chmod 0644 /etc/systemd/system/rz-admin.service
  mv /opt/rz/config/rz-release.env /opt/rz/config/rz-release.env.saved
  ln -s rz-release.env.saved /opt/rz/config/rz-release.env
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  rm /opt/rz/config/rz-release.env
  mv /opt/rz/config/rz-release.env.saved /opt/rz/config/rz-release.env
  "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  systemctl is-active --quiet rz-admin.service
  systemctl is-active --quiet rz-monitor.service
  echo "ASSERT active"; systemctl is-active --quiet rz-admin.service; systemctl is-active --quiet rz-monitor.service
  echo "ASSERT restart"; systemctl restart rz-admin.service rz-monitor.service
  for attempt in $(seq 1 30); do
    admin_health=$(curl --fail --silent http://127.0.0.1:19801/health 2>/dev/null || true)
    monitor_health=$(curl --fail --silent http://127.0.0.1:19802/health 2>/dev/null || true)
    if printf "%s" "$admin_health" | grep -Fq "$expected_binding" && printf "%s" "$monitor_health" | grep -Fq "$expected_binding"; then break; fi
    sleep 1
  done
  printf "%s" "$admin_health" | grep -F "$expected_binding"
  printf "%s" "$monitor_health" | grep -F "$expected_binding"
  systemctl is-active --quiet rz-admin.service
  systemctl is-active --quiet rz-monitor.service
  echo "ASSERT independent-start-orders"
  systemctl stop rz-admin.service rz-monitor.service
  systemctl start rz-admin.service
  systemctl start rz-monitor.service
  for attempt in $(seq 1 30); do curl --fail --silent http://127.0.0.1:19801/health | grep -Fq "$expected_binding" && curl --fail --silent http://127.0.0.1:19802/health | grep -Fq "$expected_binding" && break || sleep 1; done
  systemctl is-active --quiet rz-admin.service; systemctl is-active --quiet rz-monitor.service
  curl --fail --silent http://127.0.0.1:19801/health | grep -Fq "$expected_binding"; curl --fail --silent http://127.0.0.1:19802/health | grep -Fq "$expected_binding"
  echo "ASSERT changed-database-schema-rejected"
  systemctl stop rz-admin.service
  cp --preserve=mode,ownership,timestamps /var/lib/rustzen-admin/admin.db /var/lib/rustzen-admin/admin.db.saved
  printf garbage > /var/lib/rustzen-admin/admin.db
  chown rz-admin:rz-admin /var/lib/rustzen-admin/admin.db
  chmod 0600 /var/lib/rustzen-admin/admin.db
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  mv /var/lib/rustzen-admin/admin.db.saved /var/lib/rustzen-admin/admin.db
  systemctl start rz-admin.service
  "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  echo "ASSERT foreign-same-schema-database-rejected-without-mutation"
  systemctl stop rz-monitor.service
  set -a
  . /opt/rz/config/rz-monitor.env
  . /opt/rz/config/rz-release.env
  set +a
  export RUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rustzen-monitor/foreign.db
  runuser -u rz-monitor -- /opt/rz/current/bin/rz-monitor init-db
  foreign_build=$(printf "%064d" 0 | tr 0 f)
  RUSTZEN_BUILD_ID="$foreign_build" runuser -u rz-monitor -- /opt/rz/current/bin/rz-monitor bind-database
  mv /var/lib/rustzen-monitor/monitor.db /var/lib/rustzen-monitor/monitor.db.saved
  mv /var/lib/rustzen-monitor/foreign.db /var/lib/rustzen-monitor/monitor.db
  foreign_ledger=$(sqlite3 -readonly /var/lib/rustzen-monitor/monitor.db "SELECT version || char(58) || hex(checksum) || char(58) || success FROM _sqlx_migrations ORDER BY version")
  foreign_hash=$(sha256sum /var/lib/rustzen-monitor/monitor.db | cut -d " " -f1)
  ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  systemctl start rz-monitor.service || true
  sleep 2
  ! systemctl is-active --quiet rz-monitor.service
  systemctl stop rz-monitor.service
  observed_ledger=$(sqlite3 -readonly /var/lib/rustzen-monitor/monitor.db "SELECT version || char(58) || hex(checksum) || char(58) || success FROM _sqlx_migrations ORDER BY version")
  observed_hash=$(sha256sum /var/lib/rustzen-monitor/monitor.db | cut -d " " -f1)
  test "$foreign_hash" = "$observed_hash"
  test "$foreign_ledger" = "$observed_ledger"
  rm /var/lib/rustzen-monitor/monitor.db
  mv /var/lib/rustzen-monitor/monitor.db.saved /var/lib/rustzen-monitor/monitor.db
  unset RUSTZEN_MONITOR_SQLITE_PATH RUSTZEN_BUILD_ID RUSTZEN_COMPOSITION_ID RUSTZEN_MONITOR_SCHEMA_FINGERPRINT RUSTZEN_MONITOR_DATA_CONTRACT_ID
  systemctl start rz-monitor.service
  "$rz" --json activate-monitor-server --config /root/rz-activation/server.env
  systemctl stop rz-admin.service rz-monitor.service
  systemctl start rz-monitor.service
  systemctl start rz-admin.service
  for attempt in $(seq 1 30); do curl --fail --silent http://127.0.0.1:19801/health | grep -Fq "$expected_binding" && curl --fail --silent http://127.0.0.1:19802/health | grep -Fq "$expected_binding" && break || sleep 1; done
  systemctl is-active --quiet rz-admin.service; systemctl is-active --quiet rz-monitor.service
  curl --fail --silent http://127.0.0.1:19801/health | grep -Fq "$expected_binding"; curl --fail --silent http://127.0.0.1:19802/health | grep -Fq "$expected_binding"
  echo "ASSERT absence"; test ! -e /etc/systemd/system/rz-insights.service
  test ! -e /etc/systemd/system/rz-reports.service
  test ! -e /opt/rz/config/rz-reports.env
  echo "ASSERT cross-read"; ! runuser -u rz-admin -- cat /var/lib/rustzen-monitor/monitor.db >/dev/null
  ! runuser -u rz-monitor -- cat /var/lib/rustzen-admin/admin.db >/dev/null
'
echo "monitor server PID1 activation gate PASS"
