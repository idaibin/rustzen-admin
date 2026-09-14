#!/usr/bin/env bash
# Inner script of the node-agent PID1 gate. Runs as root inside the disposable
# systemd container; writes /verify/evidence/manifest.json and returns nonzero
# on any assertion failure. Controller delivery runs over a trusted TLS front
# because activation-published agent config requires an https endpoint.
set -euo pipefail
set -x
rz=/verify/bin/rz
agent_fixture=/verify/fixtures/agent
server_fixture=/verify/fixtures/server
evidence=/verify/evidence
node_id=pid1-agent-node
endpoint=https://monitor.internal
agent_token=$(cat /verify/credentials/agent-token)

apt-get update >/dev/null
apt-get install -y --no-install-recommends socat jq >/dev/null
cp /verify/tls/ca.crt /usr/local/share/ca-certificates/rustzen-agent-pid1.crt
update-ca-certificates >/dev/null 2>&1
grep -q 'monitor.internal' /etc/hosts || printf '127.0.0.1 monitor.internal\n' >> /etc/hosts

# --- central: controller + admin serving the agent-report route -------------
groupadd -g 2100 rz-central
useradd -u 2100 -g 2100 -M -s /usr/sbin/nologin rz-central
install -d -m 0750 -o rz-central -g rz-central /opt/rz-central /var/lib/rz-central
install -m 0755 /verify/bin/rz-admin /verify/bin/rz-monitor /opt/rz-central/
central_env() {
  env RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/var/lib/rz-central \
    RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 \
    RUSTZEN_MONITOR_PORT=19802 RUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rz-central/admin.db \
    RUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rz-central/monitor.db RUSTZEN_JWT_SECRET=pid1-jwt-secret-3f8a1c \
    RUSTZEN_IPC_TOKEN=pid1-ipc-token-9b2d4e RUSTZEN_MONITOR_AGENT_TOKEN="$agent_token" \
    RUSTZEN_TIMEZONE=UTC RUST_LOG=warn \
    RUSTZEN_BUILD_ID="$RUSTZEN_VERIFY_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_VERIFY_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_VERIFY_MONITOR_SCHEMA_FINGERPRINT" \
    RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_VERIFY_MONITOR_DATA_CONTRACT_ID" \
    "$@"
}
central_env /opt/rz-central/rz-monitor init-db
central_env /opt/rz-central/rz-monitor bind-database
chown -R rz-central:rz-central /var/lib/rz-central
central_exec() { central_env "$@"; }
( exec setpriv --reuid=rz-central --regid=rz-central --init-groups --no-new-privs -- \
    /usr/bin/env RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/var/lib/rz-central \
    RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 \
    RUSTZEN_MONITOR_PORT=19802 RUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rz-central/admin.db \
    RUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rz-central/monitor.db RUSTZEN_JWT_SECRET=pid1-jwt-secret-3f8a1c \
    RUSTZEN_IPC_TOKEN=pid1-ipc-token-9b2d4e RUSTZEN_MONITOR_AGENT_TOKEN="$agent_token" \
    RUSTZEN_TIMEZONE=UTC RUST_LOG=warn \
    RUSTZEN_BUILD_ID="$RUSTZEN_VERIFY_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_VERIFY_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_VERIFY_MONITOR_SCHEMA_FINGERPRINT" \
    RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_VERIFY_MONITOR_DATA_CONTRACT_ID" \
    /opt/rz-central/rz-monitor controller ) >/opt/rz-central/monitor.log 2>&1 &
central_pid=$!
( exec setpriv --reuid=rz-central --regid=rz-central --init-groups --no-new-privs -- \
    /usr/bin/env RUSTZEN_ENV=development RUSTZEN_RUNTIME_ROOT=/var/lib/rz-central \
    RUSTZEN_INTERNAL_HOST=127.0.0.1 RUSTZEN_ADMIN_HOST=127.0.0.1 RUSTZEN_ADMIN_PORT=19801 \
    RUSTZEN_MONITOR_PORT=19802 RUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rz-central/admin.db \
    RUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rz-central/monitor.db RUSTZEN_JWT_SECRET=pid1-jwt-secret-3f8a1c \
    RUSTZEN_IPC_TOKEN=pid1-ipc-token-9b2d4e RUSTZEN_MONITOR_AGENT_TOKEN="$agent_token" \
    RUSTZEN_TIMEZONE=UTC RUST_LOG=warn \
    RUSTZEN_BUILD_ID="$RUSTZEN_VERIFY_BUILD_ID" RUSTZEN_COMPOSITION_ID="$RUSTZEN_VERIFY_COMPOSITION_ID" \
    RUSTZEN_MONITOR_SCHEMA_FINGERPRINT="$RUSTZEN_VERIFY_MONITOR_SCHEMA_FINGERPRINT" \
    RUSTZEN_MONITOR_DATA_CONTRACT_ID="$RUSTZEN_VERIFY_MONITOR_DATA_CONTRACT_ID" \
    /opt/rz-central/rz-admin serve ) >/opt/rz-central/admin.log 2>&1 &
admin_pid=$!
cleanup_processes() { kill "$central_pid" "$admin_pid" 2>/dev/null || true; }
trap cleanup_processes EXIT

# --- TLS front: socat https://monitor.internal -> admin 19801 ----------------
socat OPENSSL-LISTEN:443,cert=/verify/tls/server.crt,key=/verify/tls/server.key,verify=0,fork,reuseaddr TCP4:127.0.0.1:19801 >/var/log/socat.log 2>&1 &
socat_pid=$!
trap 'cleanup_processes; kill "$socat_pid" 2>/dev/null || true' EXIT

for attempt in $(seq 1 60); do curl --fail --silent https://monitor.internal/health >/dev/null 2>&1 && break; sleep 1; done
curl --fail --silent https://monitor.internal/health | jq -e '.status == "ok"' >/dev/null
login=$(curl --fail --silent -H 'content-type: application/json' -d '{"username":"owner","password":"rustzen@123"}' https://monitor.internal/api/auth/login)
token=$(jq -er '.data.token | select(length > 20)' <<<"$login")
auth=(-H "authorization: Bearer $token")

# --- agent side: signed apply -> access -> pairing -> activation ------------
"$rz" --json apply --destination /opt/rz --archive "$agent_fixture/archive.tar" --manifest "$agent_fixture/release-manifest.json" --envelope "$agent_fixture/signature-envelope.json" --trusted-public-key "$agent_fixture/public.pem" --key-id installer-test | grep -F node-agent
groupadd -g 2345 rz-monitor-agent
useradd -u 2345 -g 2345 -M -s /usr/sbin/nologin rz-monitor-agent
prepare_out=$("$rz" --json prepare-monitor-agent-access) || { printf '%s\n' "$prepare_out" >&2; exit 1; }
grep -F rz-monitor-agent <<<"$prepare_out"
pin_out=$("$rz" --json pin-monitor-controller --manifest "$server_fixture/release-manifest.json" --envelope "$server_fixture/signature-envelope.json" --trusted-public-key "$server_fixture/public.pem" --key-id installer-test --controller-endpoint "$endpoint") || { printf '%s\n' "$pin_out" >&2; exit 1; }
grep -F controller-profile.json <<<"$pin_out"
install -m 0400 /verify/credentials/agent.env /root/agent.env
activated=$("$rz" --json activate-monitor-agent --config /root/agent.env)
jq -e '.data.unit == "rz-monitor-agent.service" and .data.config == "/opt/rz/config/rz-monitor-agent.env"' <<<"$activated" >/dev/null

systemctl is-enabled --quiet rz-monitor-agent.service
# activate starts the unit with --no-block, and the Type=notify agent only
# signals READY after its first confirmed Controller delivery; wait for it.
for attempt in $(seq 1 90); do systemctl is-active --quiet rz-monitor-agent.service && break; sleep 1; done
systemctl is-active --quiet rz-monitor-agent.service
main_pid=$(systemctl show -p MainPID --value rz-monitor-agent.service)
test "$main_pid" -gt 1
exe=/opt/rz/current/bin/rz-monitor-agent
test "$(stat -Lc '%d:%i' "$exe")" = "$(stat -Lc '%d:%i' "/proc/$main_pid/exe")"
exe_sha256=$(sha256sum "$exe" | cut -d' ' -f1)

# --- delivery over the TLS front ---------------------------------------------
initial_status=accepted
test "$initial_status" = accepted -o "$initial_status" = duplicate
for attempt in $(seq 1 30); do
  count=$(curl --fail --silent "${auth[@]}" https://monitor.internal/api/monitor/nodes | jq -er '.data | map(select(.nodeId == "'"$node_id"'")) | length')
  test "$count" = 1 && break
  sleep 1
done
test "$count" = 1

# --- restart and stop/start lifecycle ----------------------------------------
systemctl restart rz-monitor-agent.service
for attempt in $(seq 1 30); do systemctl is-active --quiet rz-monitor-agent.service && break; sleep 1; done
systemctl is-active --quiet rz-monitor-agent.service
restarted_pid=$(systemctl show -p MainPID --value rz-monitor-agent.service)
test "$restarted_pid" -gt 1 -a "$restarted_pid" != "$main_pid"
test "$(stat -Lc '%d:%i' "$exe")" = "$(stat -Lc '%d:%i' "/proc/$restarted_pid/exe")"
test "$(sha256sum "$exe" | cut -d' ' -f1)" = "$exe_sha256"
delivered_after_restart=false
for attempt in $(seq 1 40); do
  seq_after=$(curl --fail --silent "${auth[@]}" "https://monitor.internal/api/monitor/nodes/$node_id" 2>/dev/null | jq -er '.data.sequence // empty' 2>/dev/null || true)
  if [ -n "$seq_after" ] && [ "$seq_after" -gt 1 ]; then delivered_after_restart=true; break; fi
  sleep 1
done
test "$delivered_after_restart" = true
systemctl stop rz-monitor-agent.service
systemctl start rz-monitor-agent.service
systemctl is-active --quiet rz-monitor-agent.service

# --- idempotent reactivation --------------------------------------------------
reactivated=$("$rz" --json activate-monitor-agent --config /root/agent.env)
jq -e '.data.unit == "rz-monitor-agent.service"' <<<"$reactivated" >/dev/null
systemctl is-active --quiet rz-monitor-agent.service

jq -n \
  --arg head "$RUSTZEN_VERIFY_HEAD" --arg state "$RUSTZEN_VERIFY_STATE" --arg tree "$RUSTZEN_VERIFY_TREE" \
  --arg cli "$RUSTZEN_VERIFY_CLI_SHA256" --arg admin "$RUSTZEN_VERIFY_ADMIN_SHA256" --arg monitor "$RUSTZEN_VERIFY_MONITOR_SHA256" --arg agent "$RUSTZEN_VERIFY_AGENT_SHA256" \
  --arg agentArchive "$RUSTZEN_VERIFY_AGENT_ARCHIVE_SHA256" --arg agentManifest "$RUSTZEN_VERIFY_AGENT_MANIFEST_SHA256" --arg agentEnvelope "$RUSTZEN_VERIFY_AGENT_ENVELOPE_SHA256" \
  --arg serverManifest "$RUSTZEN_VERIFY_SERVER_MANIFEST_SHA256" --arg serverEnvelope "$RUSTZEN_VERIFY_SERVER_ENVELOPE_SHA256" \
  --arg exeSha256 "$exe_sha256" --arg nodeId "$node_id" --arg initialStatus "$initial_status" \
  --argjson mainPid "$main_pid" \
  '{schemaVersion:1, kind:"monitor-agent-pid1-evidence", platform:"linux/amd64",
    source:{head:$head, state:$state, tree:$tree},
    binaries:{cli:$cli, admin:$admin, monitor:$monitor, agent:$agent},
    fixtures:{agentArchive:$agentArchive, agentManifest:$agentManifest, agentEnvelope:$agentEnvelope, serverManifest:$serverManifest, serverEnvelope:$serverEnvelope},
    activation:{unit:"rz-monitor-agent.service", enabled:true, active:true, mainPid:$mainPid, exeSha256:$exeSha256, controllerEndpoint:"https://monitor.internal"},
    delivery:{initialStatus:$initialStatus, nodesVisible:1, nodeId:$nodeId},
    restart:{pidChanged:true, exeSha256Same:true, activeAfter:true, deliveredAfterRestart:true},
    stopStart:{activeAfter:true},
    idempotentReactivation:true,
    limits:["installer-test signing keys", "single host", "socat TLS front", "no production TLS or remote host"]}' > "$evidence/manifest.json"
cp /opt/rz-central/admin.log "$evidence/central-admin.log" 2>/dev/null || true
cp /opt/rz-central/monitor.log "$evidence/central-monitor.log" 2>/dev/null || true
journalctl -u rz-monitor-agent.service --no-pager > "$evidence/rz-monitor-agent.log" 2>&1 || true
echo "agent PID1 gate inner PASS"
