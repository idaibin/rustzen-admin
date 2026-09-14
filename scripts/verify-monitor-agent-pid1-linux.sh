#!/usr/bin/env bash
# Final node-agent gate: one disposable linux/amd64 systemd PID1 container runs
# the signed agent release end to end — apply, pairing, activation, confirmed
# delivery over a trusted TLS front, restart, stop/start and idempotent
# re-activation — and publishes one canonical receipt.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"

usage() { echo "usage: $0 --output NEW_DIRECTORY" >&2; exit 2; }
test "$#" = 2 || usage
test "$1" = "--output" || usage
output=$2
output_name="$(basename "$output")"
test "$output_name" != . && test "$output_name" != .. && test -n "$output_name" || usage
output_parent="$(cd "$(dirname "$output")" && pwd -P)"
case "$output_parent" in "$root/target/rz"|"$root/target/rz"/*) ;; *) echo "--output parent must be beneath target/rz" >&2; exit 2 ;; esac
output="$output_parent/$output_name"
test ! -e "$output" && test ! -L "$output" || { echo "--output must not exist: $output" >&2; exit 2; }

platform=linux/amd64
target_triple=x86_64-unknown-linux-musl
umask 022
work="$(mktemp -d "$root/target/rz/.agent-pid1.XXXXXX")"
bin_dir="$work/bin-stage"
mkdir -p "$bin_dir"
# The server pair comes from the prebuilt source-bound musl build; only the CLI
# is rebuilt so this gate always carries the current installer/pairing code.
prebuilt="$root/target/rz/build/x86_64/bin"
for name in rz-admin rz-monitor; do
  test -x "$prebuilt/$name" || { echo "missing prebuilt binary: $prebuilt/$name; run just build-admin-browser-linux" >&2; exit 1; }
  cp "$prebuilt/$name" "$bin_dir/$name"
done
cli_build_status=0
docker run --name "rz-agent-pid1-cli-$$" --platform linux/amd64 -v "$root:/work" -w /work \
  -v rustzen-agent-pid1-cargo:/usr/local/cargo/registry -v rustzen-agent-pid1-target:/cli-target rust:1.95-bookworm \
  bash -euo pipefail -c "apt-get update >/dev/null && apt-get install -y --no-install-recommends musl-tools >/dev/null && rustup target add $target_triple >/dev/null && CARGO_TARGET_DIR=/cli-target cargo build --release --target $target_triple -p rustzen-cli" || cli_build_status=$?
docker rm -f "rz-agent-pid1-cli-$$" >/dev/null 2>&1 || true
[ "$cli_build_status" -eq 0 ] || { echo "cli build failed" >&2; exit "$cli_build_status"; }
docker run --rm --platform linux/amd64 -v rustzen-agent-pid1-target:/cli-target:ro -v "$bin_dir:/out" debian:bookworm-slim cp "/cli-target/$target_triple/release/rz" /out/rz
test -x "$bin_dir/rz"

evidence="$work/evidence"; fixtures="$work/fixtures"; tls="$work/tls"; credentials="$work/credentials"; bins="$work/bin"
container=""; build_container=""
owner_token="$(pnpm dlx bun@1.3.14 -e 'console.log(crypto.randomUUID())')"
cleanup() {
  status=$?
  trap - EXIT INT TERM
  if [ -n "$container" ] && docker ps -a --format '{{.Names}}' | grep -Fxq "$container"; then
    docker exec "$container" journalctl -u rz-monitor-agent.service --no-pager >> "$evidence/runtime-failure.log" 2>&1 || true
    docker exec "$container" cat /opt/rz-central/admin.log /opt/rz-central/monitor.log >> "$evidence/runtime-failure.log" 2>&1 || true
    docker rm -f "$container" >/dev/null 2>&1 || true
  fi
  [ -z "$build_container" ] || docker rm -f "$build_container" >/dev/null 2>&1 || true
  docker ps -aq --filter "label=io.rustzen.p8g-owner=$owner_token" | xargs -r docker rm -f >/dev/null 2>&1 || true
  if [ "$status" -eq 0 ]; then rm -rf "$work"; else echo "gate failed; work dir kept: $work" >&2; fi
  exit "$status"
}
trap cleanup EXIT INT TERM

mkdir -p "$evidence" "$fixtures" "$tls/creds" "$credentials" "$bins"
read -r head source_state source_tree < <("$root/scripts/admin-browser-source-identity.sh")

# --- agent binary from the same source basis ---------------------------------
agent_bin="$root/target/agent-pid1-cargo/$target_triple/release/rz-monitor-agent"
agent_build_status=0
docker run --name "rz-agent-pid1-agent-$$" --platform "$platform" -v "$root:/work" -w /work \
  -v rustzen-agent-pid1-cargo:/usr/local/cargo/registry rust:1.95-bookworm \
  bash -euo pipefail -c "apt-get update >/dev/null && apt-get install -y --no-install-recommends musl-tools >/dev/null && rustup target add $target_triple >/dev/null && cargo build --release --target $target_triple -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent --target-dir target/agent-pid1-cargo" || agent_build_status=$?
docker rm -f "rz-agent-pid1-agent-$$" >/dev/null 2>&1 || true
[ "$agent_build_status" -eq 0 ] || { echo "agent build failed" >&2; exit "$agent_build_status"; }
test -x "$agent_bin"
file "$agent_bin" | grep -q 'x86-64'

# --- signed fixtures bound to the real agent binary ---------------------------
RUSTZEN_INSTALLER_TARGET=$target_triple RUSTZEN_INSTALLER_OUTPUT="$fixtures/agent" RUSTZEN_INSTALLER_ARTIFACT=node-agent RUSTZEN_INSTALLER_AGENT_BINARY="$agent_bin" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts >/dev/null
RUSTZEN_INSTALLER_TARGET=$target_triple RUSTZEN_INSTALLER_OUTPUT="$fixtures/server" pnpm dlx bun@1.3.14 scripts/distribution-installer-fixture.ts >/dev/null

# --- TLS material: local CA + server cert for monitor.internal ---------------
CA_SUBJECT="/CN=Rustzen Agent PID1 CA"
openssl req -x509 -newkey rsa:2048 -nodes -keyout "$tls/ca.key" -out "$tls/ca.crt" -days 2 -subj "$CA_SUBJECT" >/dev/null 2>&1
openssl req -newkey rsa:2048 -nodes -keyout "$tls/server.key" -out "$tls/server.csr" -subj "/CN=monitor.internal" >/dev/null 2>&1
printf 'subjectAltName=DNS:monitor.internal\nbasicConstraints=CA:FALSE\nkeyUsage=digitalSignature,keyEncipherment\nextendedKeyUsage=serverAuth\n' > "$tls/ext.cnf"
openssl x509 -req -in "$tls/server.csr" -CA "$tls/ca.crt" -CAkey "$tls/ca.key" -CAcreateserial -out "$tls/server.crt" -days 2 -extfile "$tls/ext.cnf" >/dev/null 2>&1

# --- credentials (mode 0600, live only inside the work dir) -------------------
umask 077
printf '%s\n' 'pid1-agent-token-52af80' > "$credentials/agent-token"
printf 'RUSTZEN_ENV=production\nRUSTZEN_MONITOR_AGENT_TOKEN=pid1-agent-token-52af80\nRUSTZEN_MONITOR_CONTROLLER_URL=https://monitor.internal\nRUSTZEN_MONITOR_NODE_ID=pid1-agent-node\n' > "$credentials/agent.env"
umask 022

# --- gather identities --------------------------------------------------------
cli_sha=$(sha256sum "$bin_dir/rz" | cut -d' ' -f1)
admin_sha=$(sha256sum "$bin_dir/rz-admin" | cut -d' ' -f1)
monitor_sha=$(sha256sum "$bin_dir/rz-monitor" | cut -d' ' -f1)
agent_sha=$(sha256sum "$agent_bin" | cut -d' ' -f1)
agent_archive_sha=$(sha256sum "$fixtures/agent/archive.tar" | cut -d' ' -f1)
agent_manifest_sha=$(sha256sum "$fixtures/agent/release-manifest.json" | cut -d' ' -f1)
agent_envelope_sha=$(sha256sum "$fixtures/agent/signature-envelope.json" | cut -d' ' -f1)
server_manifest_sha=$(sha256sum "$fixtures/server/release-manifest.json" | cut -d' ' -f1)
server_envelope_sha=$(sha256sum "$fixtures/server/signature-envelope.json" | cut -d' ' -f1)
# bind-database identity inputs must match the signed server fixture manifest.
build_id=$(jq -er '.buildId // .build_id // empty' "$fixtures/server/release-manifest.json" 2>/dev/null || true)
composition_id=$(jq -er '.compositionId // .selection.compositionId // empty' "$fixtures/server/release-manifest.json" 2>/dev/null || true)
[ -n "$build_id" ] && [ -n "$composition_id" ] || { echo "cannot read fixture identity" >&2; exit 1; }

install -m 0755 "$bin_dir/rz" "$bin_dir/rz-admin" "$bin_dir/rz-monitor" "$bins/"
image=rz-p8e-systemd-bookworm-amd64-v1
if ! docker image inspect "$image" >/dev/null 2>&1; then
  setup="rz-agent-pid1-image-$$"
  docker run --name "$setup" --platform linux/amd64 debian:bookworm-slim /bin/bash -euo pipefail -c 'apt-get update >/dev/null; apt-get install -y systemd-sysv curl ca-certificates >/dev/null'
  docker commit "$setup" "$image" >/dev/null
  docker rm "$setup" >/dev/null
fi

container="rz-agent-pid1-$$"
docker run -d --name "$container" --label "io.rustzen.p8g-owner=$owner_token" --privileged --cgroupns=host \
  --platform linux/amd64 --tmpfs /run --tmpfs /run/lock "$image" /sbin/init >/dev/null
for attempt in $(seq 1 30); do docker exec "$container" systemctl is-system-running --wait >/dev/null 2>&1 && break || sleep 1; done
docker exec "$container" /bin/bash -c 'state=$(systemctl is-system-running || true); test "$state" = running -o "$state" = degraded'

docker exec "$container" mkdir -p /verify/bin /verify/fixtures /verify/tls /verify/credentials /verify/evidence
docker cp "$bins/." "$container:/verify/bin"
docker cp "$fixtures/." "$container:/verify/fixtures"
docker cp "$tls/ca.crt" "$container:/verify/tls/ca.crt"
docker cp "$tls/server.crt" "$container:/verify/tls/server.crt"
docker cp "$tls/server.key" "$container:/verify/tls/server.key"
docker cp "$credentials/." "$container:/verify/credentials"
docker cp "$evidence/." "$container:/verify/evidence"
docker cp scripts/verify-monitor-agent-pid1-linux-inner.sh "$container:/verify/run.sh"

docker exec \
  --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_STATE="$source_state" --env RUSTZEN_VERIFY_TREE="$source_tree" \
  --env RUSTZEN_VERIFY_CLI_SHA256="$cli_sha" --env RUSTZEN_VERIFY_ADMIN_SHA256="$admin_sha" --env RUSTZEN_VERIFY_MONITOR_SHA256="$monitor_sha" --env RUSTZEN_VERIFY_AGENT_SHA256="$agent_sha" \
  --env RUSTZEN_VERIFY_AGENT_ARCHIVE_SHA256="$agent_archive_sha" --env RUSTZEN_VERIFY_AGENT_MANIFEST_SHA256="$agent_manifest_sha" --env RUSTZEN_VERIFY_AGENT_ENVELOPE_SHA256="$agent_envelope_sha" \
  --env RUSTZEN_VERIFY_SERVER_MANIFEST_SHA256="$server_manifest_sha" --env RUSTZEN_VERIFY_SERVER_ENVELOPE_SHA256="$server_envelope_sha" \
  --env RUSTZEN_VERIFY_BUILD_ID="$build_id" --env RUSTZEN_VERIFY_COMPOSITION_ID="$composition_id" \
  --env RUSTZEN_VERIFY_MONITOR_SCHEMA_FINGERPRINT="$server_manifest_sha" \
  --env RUSTZEN_VERIFY_MONITOR_DATA_CONTRACT_ID="$server_envelope_sha" \
  "$container" bash /verify/run.sh 2>&1 | tee "$evidence/inner.log"

docker cp "$container:/verify/evidence/." "$evidence/"
docker rm -f "$container" >/dev/null; container=
# jq writes insertion-ordered JSON; canonicalize before strict parsing.
pnpm dlx bun@1.3.14 -e 'import { canonicalJson } from "./distribution/release-manifest-core.ts"; const value = JSON.parse(await Bun.file(process.argv[1]).text()); await Bun.write(process.argv[1], canonicalJson(value));' "$evidence/manifest.json"

pnpm dlx bun@1.3.14 scripts/monitor-agent-pid1-receipt.ts "$evidence/manifest.json"
read -r final_head final_state final_tree < <("$root/scripts/admin-browser-source-identity.sh")
test "$final_head" = "$head" && test "$final_state" = "$source_state" && test "$final_tree" = "$source_tree"
mv "$evidence" "$output"
pnpm dlx bun@1.3.14 scripts/monitor-agent-pid1-receipt.ts "$output/manifest.json"
echo "node-agent PID1/service-restart gate PASS: $output/manifest.json"
