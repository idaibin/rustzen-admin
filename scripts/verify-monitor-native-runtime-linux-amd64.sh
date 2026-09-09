#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"
usage() {
  echo "usage: $0 --export-root PATH --release-result FILE --certificate FILE --public-key FILE --expected-source-identity ID --output NEW_DIRECTORY [--retain-container-output FILE --retain-owner-token TOKEN]" >&2
  exit 2
}
test "$#" -eq 12 -o "$#" -eq 16 || usage
while test "$#" -gt 0; do
  case "$1" in
    --export-root) test -z "${export_arg:-}" || usage; export_arg="$2" ;;
    --release-result) test -z "${release_arg:-}" || usage; release_arg="$2" ;;
    --certificate) test -z "${certificate_arg:-}" || usage; certificate_arg="$2" ;;
    --public-key) test -z "${key_arg:-}" || usage; key_arg="$2" ;;
    --expected-source-identity) test -z "${source_identity:-}" || usage; source_identity="$2" ;;
    --output) test -z "${output_arg:-}" || usage; output_arg="$2" ;;
    --retain-container-output) test -z "${retain_arg:-}" || usage; retain_arg="$2" ;;
    --retain-owner-token) test -z "${retain_owner:-}" || usage; retain_owner="$2" ;;
    *) usage ;;
  esac
  test -n "$2" || usage
  shift 2
done
for value in "${export_arg:-}" "${release_arg:-}" "${certificate_arg:-}" "${key_arg:-}" "${source_identity:-}" "${output_arg:-}"; do test -n "$value" || usage; done
canonical_file() { test -f "$1" && test ! -L "$1" && realpath "$1"; }
canonical_directory() { test -d "$1" && test ! -L "$1" && realpath "$1"; }
export_root="$(canonical_directory "$export_arg")"
release_result="$(canonical_file "$release_arg")"
certificate="$(canonical_file "$certificate_arg")"
public_key="$(canonical_file "$key_arg")"
for path in "$export_root" "$release_result" "$certificate"; do
  case "$path" in "$root"/*) ;; *) echo "runtime input must be beneath repository root: $path" >&2; exit 2 ;; esac
done
output_name="$(basename "$output_arg")"
test "$output_name" != . && test "$output_name" != .. && test -n "$output_name" || usage
output_parent="$(cd "$(dirname "$output_arg")" && pwd -P)"
case "$output_parent" in "$root/target/rz"|"$root/target/rz"/*) ;; *) echo "--output parent must be beneath target/rz" >&2; exit 2 ;; esac
output="$output_parent/$output_name"
test ! -e "$output" && test ! -L "$output" || { echo "--output must not exist: $output" >&2; exit 2; }
if test -n "${retain_arg:-}"; then
  test "${retain_owner:-}" != "" && printf '%s' "$retain_owner" | grep -Eq '^[a-f0-9-]{32,64}$' || usage
  retain_name="$(basename "$retain_arg")"; test -n "$retain_name" && test "$retain_name" != . && test "$retain_name" != .. || usage
  retain_parent="$(cd "$(dirname "$retain_arg")" && pwd -P)"
  case "$retain_parent" in "$root/target/rz"|"$root/target/rz"/*) ;; *) echo "--retain-container-output must be beneath target/rz" >&2; exit 2 ;; esac
  retain_output="$retain_parent/$retain_name"; test ! -e "$retain_output" && test ! -L "$retain_output" || { echo "--retain-container-output must be fresh" >&2; exit 2; }
fi
reject_overlap() {
  case "$output/" in "$1/"*) echo "--output overlaps protected input: $1" >&2; exit 2 ;; esac
  case "$1/" in "$output/"*) echo "--output overlaps protected input: $1" >&2; exit 2 ;; esac
}
for protected in "$export_root" "$release_result" "$(dirname "$certificate")" "$public_key"; do reject_overlap "$protected"; done
release_root="$(pnpm dlx bun@1.3.14 -e 'console.log(JSON.parse(await Bun.file(process.argv[1]).text()).root)' "$release_result")"
release_root="$(canonical_directory "$release_root")"
case "$release_root" in "$root"/*) ;; *) echo "release root must be beneath repository root" >&2; exit 2 ;; esac
reject_overlap "$release_root"
key_id="$(pnpm dlx bun@1.3.14 -e 'console.log(JSON.parse(await Bun.file(process.argv[1]).text()).payload.keyId)' "$release_root/signature-envelope.json")"
archive="$release_root/archive.tar"
manifest="$release_root/release-manifest.json"
envelope="$release_root/signature-envelope.json"
for artifact in "$archive" "$manifest" "$envelope" "$certificate" "$public_key"; do test -f "$artifact"; done
admission="$(pnpm dlx bun@1.3.14 scripts/distribution-verify-published-source-build-certificate.ts \
  --selection distribution/fixtures/monitor.json --export-root "$export_root" \
  --expected-source-identity "$source_identity" --release-root "$release_root" \
  --public-key "$public_key" --key-id "$key_id" --certificate "$certificate")"
mkdir "$output"
printf '%s\n' "$admission" > "$output/published-certificate.json"

test "$(docker version --format '{{.Server.Arch}}')" = amd64
test "$(docker run --rm --platform linux/amd64 debian:bookworm-slim uname -m)" = x86_64
docker run --rm --platform linux/amd64 -v "$root:/work:ro" -w /work \
  -v rustzen-p8e-cli-cargo:/usr/local/cargo/registry -v rustzen-p8e-cli-target:/cargo-target \
  rust:1.95-bookworm cargo build --release -p rustzen-cli --target-dir /cargo-target
docker run --rm --platform linux/amd64 -v rustzen-p8e-cli-target:/cargo-target:ro -v "$output:/output" \
  debian:bookworm-slim cp /cargo-target/release/rz /output/rz
cli="$output/rz"
test -x "$cli"
name="rz-p8e-native-runtime-$$"
setup="$name-setup"
image=rz-p8e-systemd-bookworm-amd64-v1
private=/root/rz-p8e-artifacts
admin_host=127.0.0.1; test -z "${retain_output:-}" || admin_host=0.0.0.0
cleanup() {
  status=$?
  if [ "$status" -ne 0 ] && docker ps -a --format '{{.Names}}' | grep -Fxq "$name"; then
    docker exec "$name" systemctl --no-pager --full status rz-admin.service rz-monitor.service >> "$output/runtime.log" 2>&1 || true
    docker exec "$name" journalctl --no-pager -u rz-admin.service -u rz-monitor.service >> "$output/runtime.log" 2>&1 || true
    docker logs "$name" >> "$output/runtime.log" 2>&1 || true
  fi
  if test "$status" -ne 0 || test -z "${retain_output:-}"; then docker rm -f "$name" >/dev/null 2>&1 || true; fi
  docker rm -f "$setup" >/dev/null 2>&1 || true
  exit "$status"
}
trap cleanup EXIT
if ! docker image inspect "$image" >/dev/null 2>&1; then
  docker run --name "$setup" --platform linux/amd64 debian:bookworm-slim /bin/bash -euo pipefail -c 'apt-get update >/dev/null; apt-get install -y systemd-sysv curl ca-certificates >/dev/null'
  docker commit "$setup" "$image" >/dev/null
  docker rm "$setup" >/dev/null
fi
if test -n "${retain_output:-}"; then
  docker run -d --name "$name" --label "io.rustzen.p8g-owner=${retain_owner:?}" --privileged --cgroupns=private --cpus 4 --memory 512m --pids-limit 256 -p 127.0.0.1::19801 --platform linux/amd64 --tmpfs /run --tmpfs /run/lock "$image" /sbin/init >/dev/null
else
  docker run -d --name "$name" --privileged --cgroupns=host --platform linux/amd64 --tmpfs /run --tmpfs /run/lock "$image" /sbin/init >/dev/null
fi
for attempt in $(seq 1 30); do docker exec "$name" systemctl is-system-running --wait >/dev/null 2>&1 && break || sleep 1; done
docker exec "$name" /bin/bash -c 'state=$(systemctl is-system-running || true); test "$state" = running -o "$state" = degraded'
docker exec "$name" install -d -m 0700 "$private"
docker cp "$archive" "$name:$private/archive.tar"
docker cp "$manifest" "$name:$private/release-manifest.json"
docker cp "$envelope" "$name:$private/signature-envelope.json"
docker cp "$public_key" "$name:$private/public.pem"
docker cp "$certificate" "$name:$private/source-build-manifest.json"
docker cp "$cli" "$name:$private/rz"
docker exec "$name" chmod 0700 "$private"
docker exec "$name" chmod 0600 "$private/archive.tar" "$private/release-manifest.json" "$private/signature-envelope.json" "$private/public.pem" "$private/source-build-manifest.json"
docker exec "$name" chmod 0700 "$private/rz"
for artifact in archive.tar release-manifest.json signature-envelope.json public.pem source-build-manifest.json; do
  source_path="$private/$artifact"
  case "$artifact" in
    archive.tar) host_path="$archive" ;;
    release-manifest.json) host_path="$manifest" ;;
    signature-envelope.json) host_path="$envelope" ;;
    public.pem) host_path="$public_key" ;;
    source-build-manifest.json) host_path="$certificate" ;;
  esac
  test "$(sha256sum "$host_path" | cut -d ' ' -f1)" = "$(docker exec "$name" sha256sum "$source_path" | cut -d ' ' -f1)"
done
docker exec "$name" /bin/bash -euo pipefail -c '
  private=/root/rz-p8e-artifacts
  rz="$private/rz"
  install -d -m 0700 /root/rz-activation
  cat > /root/rz-activation/server.env <<EOF
RUSTZEN_ENV=production
RUSTZEN_ADMIN_HOST='"$admin_host"'
RUSTZEN_ADMIN_PORT=19801
RUSTZEN_MONITOR_PORT=19802
RUSTZEN_ADMIN_RUNTIME_ROOT=/var/lib/rustzen-admin
RUSTZEN_MONITOR_RUNTIME_ROOT=/var/lib/rustzen-monitor
RUSTZEN_ADMIN_SQLITE_PATH=/var/lib/rustzen-admin/admin.db
RUSTZEN_MONITOR_SQLITE_PATH=/var/lib/rustzen-monitor/monitor.db
RUSTZEN_JWT_SECRET=p8e-jwt-secret-7f5d29
RUSTZEN_IPC_TOKEN=p8e-ipc-token-9ac412
RUSTZEN_MONITOR_AGENT_TOKEN=p8e-agent-token-1b73ef
RUSTZEN_BOOTSTRAP_OWNER_PASSWORD=p8e-owner-password-4c819a
EOF
  chmod 0600 /root/rz-activation/server.env
  /usr/bin/true
  "$rz" --json verify --archive "$private/archive.tar" --manifest "$private/release-manifest.json" --envelope "$private/signature-envelope.json" --trusted-public-key "$private/public.pem" --key-id '"$key_id"' > /root/rz-activation/verify.json
  "$rz" --json apply --dry-run --destination /root/rz-p8e-dry --archive "$private/archive.tar" --manifest "$private/release-manifest.json" --envelope "$private/signature-envelope.json" --trusted-public-key "$private/public.pem" --key-id '"$key_id"' > /root/rz-activation/dry-run.json
  test ! -e /root/rz-p8e-dry
  "$rz" --json apply --destination /opt/rz --archive "$private/archive.tar" --manifest "$private/release-manifest.json" --envelope "$private/signature-envelope.json" --trusted-public-key "$private/public.pem" --key-id '"$key_id"' > /root/rz-activation/apply.json
  "$rz" --json install-status --destination /opt/rz > /root/rz-activation/install-status.json
  if ! "$rz" --json activate-monitor-server --config /root/rz-activation/server.env > /root/rz-activation/activate.json; then cat /root/rz-activation/activate.json >&2; exit 1; fi
  test "$(wc -l < /root/rz-activation/activate.json | tr -d " ")" = 1
  build_id=$(sed -n "s/.*\"buildId\":\"\([a-f0-9]\{64\}\)\".*/\1/p" "$private/release-manifest.json")
  composition_id=$(sed -n "s/.*\"compositionId\":\"\([a-f0-9]\{64\}\)\".*/\1/p" "$private/release-manifest.json")
  test -n "$build_id"; test -n "$composition_id"
  binding="\"selectedBinding\":{\"buildId\":\"$build_id\",\"compositionId\":\"$composition_id\"}"
  wait_health() {
    for attempt in $(seq 1 30); do
      curl --fail --silent http://127.0.0.1:19801/health | grep -Fq "$binding" && curl --fail --silent http://127.0.0.1:19802/health | grep -Fq "$binding" && return 0
      sleep 1
    done
    return 1
  }
  systemctl is-enabled --quiet rz.target
  systemctl is-active --quiet rz.target
  for unit in rz-admin.service rz-monitor.service; do systemctl is-active --quiet "$unit"; done
  admin_health=$(curl --fail --silent http://127.0.0.1:19801/health); monitor_health=$(curl --fail --silent http://127.0.0.1:19802/health)
  printf "%s" "$admin_health" | grep -Fq "$binding"; printf "%s" "$monitor_health" | grep -Fq "$binding"
  curl --silent --show-error -o /root/rz-activation/owner-login.json -w "%{http_code}" -H "content-type: application/json" -d "{\"username\":\"owner\",\"password\":\"p8e-owner-password-4c819a\"}" http://127.0.0.1:19801/api/auth/login > /root/rz-activation/owner-login.status
  test "$(cat /root/rz-activation/owner-login.status)" = 200
  for account in owner admin viewer; do
    curl --silent --show-error -o "/root/rz-activation/default-$account.json" -w "%{http_code}" -H "content-type: application/json" -d "{\"username\":\"$account\",\"password\":\"rustzen@123\"}" http://127.0.0.1:19801/api/auth/login > "/root/rz-activation/default-$account.status"
    test "$(cat "/root/rz-activation/default-$account.status")" = 401
  done
  ! grep -R -Fq p8e-owner-password-4c819a /opt/rz
  for absent in /etc/systemd/system/rz-insights.service /etc/systemd/system/rz-reports.service /opt/rz/config/rz-insights.env /opt/rz/config/rz-reports.env /var/lib/rustzen-insights /var/lib/rustzen-reports; do test ! -e "$absent"; done
  systemctl restart rz-admin.service rz-monitor.service
  wait_health
  systemctl stop rz-admin.service rz-monitor.service; systemctl start rz-admin.service; systemctl start rz-monitor.service
  wait_health
  systemctl is-active --quiet rz-admin.service; systemctl is-active --quiet rz-monitor.service
  systemctl stop rz-admin.service rz-monitor.service; systemctl start rz-monitor.service; systemctl start rz-admin.service
  wait_health
  systemctl is-active --quiet rz-admin.service; systemctl is-active --quiet rz-monitor.service
  admin_pid=$(systemctl show -p MainPID --value rz-admin.service); monitor_pid=$(systemctl show -p MainPID --value rz-monitor.service)
  for pair in "rz-admin.service:$admin_pid:rz-admin" "rz-monitor.service:$monitor_pid:rz-monitor"; do
    unit=${pair%%:*}; remaining=${pair#*:}; pid=${remaining%%:*}; binary=${remaining##*:}
    test "$pid" -gt 0
    test "$(stat -Lc "%d:%i" "/opt/rz/current/bin/$binary")" = "$(stat -Lc "%d:%i" "/proc/$pid/exe")"
    test "$(sha256sum "/opt/rz/current/bin/$binary" | cut -d " " -f1)" = "$(sha256sum "/proc/$pid/exe" | cut -d " " -f1)"
  done
  cat > "$private/facts.json" <<EOF
{"keyId":"'"$key_id"'","buildId":"$build_id","compositionId":"$composition_id","certificateSha256":"$(sha256sum "$private/source-build-manifest.json" | cut -d " " -f1)","manifestSha256":"$(sha256sum "$private/release-manifest.json" | cut -d " " -f1)","archiveSha256":"$(sha256sum "$private/archive.tar" | cut -d " " -f1)","envelopeSha256":"$(sha256sum "$private/signature-envelope.json" | cut -d " " -f1)","publicationSha256":"$(sha256sum /opt/rz/state/publication-marker.json | cut -d " " -f1)","activationSha256":"$(sha256sum /opt/rz/state/monitor-server-activation.json | cut -d " " -f1)","adminPid":$admin_pid,"adminDev":"$(stat -Lc "%d" /proc/$admin_pid/exe)","adminIno":"$(stat -Lc "%i" /proc/$admin_pid/exe)","adminSha256":"$(sha256sum /proc/$admin_pid/exe | cut -d " " -f1)","monitorPid":$monitor_pid,"monitorDev":"$(stat -Lc "%d" /proc/$monitor_pid/exe)","monitorIno":"$(stat -Lc "%i" /proc/$monitor_pid/exe)","monitorSha256":"$(sha256sum /proc/$monitor_pid/exe | cut -d " " -f1)"}
EOF
' 2>&1 | tee "$output/runtime.log"
docker cp "$name:$private/facts.json" "$output/facts.json"
for record in verify dry-run apply install-status activate; do docker cp "$name:/root/rz-activation/$record.json" "$output/$record.json"; done
for record in owner-login owner admin viewer; do
  case "$record" in
    owner-login) source=owner-login ;;
    *) source="default-$record" ;;
  esac
  docker cp "$name:/root/rz-activation/$source.json" "$output/$source.json"
  docker cp "$name:/root/rz-activation/$source.status" "$output/$source.status"
done
pnpm dlx bun@1.3.14 -e 'const root=process.argv[1]; const read=async(name)=>({status:(await Bun.file(root+"/"+name+".status").text()).trim(),body:JSON.parse(await Bun.file(root+"/"+name+".json").text())}); const owner=await read("owner-login"); const defaults=await Promise.all(["owner","admin","viewer"].map(async(account)=>({account,...await read("default-"+account)}))); await Bun.write(root+"/login-evidence.json", JSON.stringify({owner,defaults}));' "$output"
docker cp "$name:/opt/rz/state/publication-marker.json" "$output/publication-marker.json"
docker cp "$name:/opt/rz/state/monitor-server-activation.json" "$output/activation-marker.json"
docker exec "$name" /bin/bash -euo pipefail -c 'true' >> "$output/runtime.log" 2>&1
pnpm dlx bun@1.3.14 -e 'import { monitorNativeRuntimeEvidenceBytes } from "./distribution/monitor-native-runtime-evidence.ts"; const read=(p)=>Bun.file(p).json(); const [f,admission,release,verify,dry,apply,status,activation,publication,activationMarker]=await Promise.all(process.argv.slice(1,11).map(read)); const fail=(m)=>{throw new Error(m)}; if(admission.selection.target!=="x86_64-unknown-linux-musl"||admission.selection.artifactClass!=="server") fail("admission selection is invalid"); if(admission.buildId!==release.buildId||admission.manifestSha256!==release.manifestSha256||admission.archiveSha256!==release.archiveSha256||admission.envelopeSha256!==release.envelopeSha256) fail("admission release tuple differs"); if(admission.selection.compositionId!==f.compositionId||admission.buildId!==f.buildId||verify.data.build_id!==f.buildId||verify.data.target!==admission.selection.target||apply.data.release.build_id!==f.buildId||dry.data.release.build_id!==f.buildId||status.data.markerPresent!==true||status.data.runnable!==false||activation.data.unit!=="rz.target") fail("CLI tuple differs"); const marker=publication.journal; if(marker.buildId!==f.buildId||marker.target!==admission.selection.target||marker.artifactClass!==admission.selection.artifactClass||marker.compositionId!==f.compositionId||marker.keyId!==f.keyId||marker.archiveSha256!==admission.archiveSha256||marker.manifestSha256!==admission.manifestSha256||marker.envelopeSha256!==admission.envelopeSha256||publication.state!=="payload-published"||activationMarker.buildId!==f.buildId||activationMarker.state!=="ready") fail("marker tuple differs"); const binary=admission.binaryDigests; if(JSON.stringify(binary)!==JSON.stringify([{path:"bin/rz-admin",sha256:f.adminSha256},{path:"bin/rz-monitor",sha256:f.monitorSha256}])) fail("certificate binary digest differs"); const evidence={schemaVersion:1,kind:"monitor-native-runtime-evidence",platform:"linux/amd64",selection:{preset:"monitor",target:admission.selection.target,artifactClass:admission.selection.artifactClass,compositionId:f.compositionId,buildId:f.buildId},release:{keyId:f.keyId,certificateSha256:admission.certificateSha256,manifestSha256:admission.manifestSha256,archiveSha256:admission.archiveSha256,envelopeSha256:admission.envelopeSha256,binaryDigests:binary},installation:{verify:true,dryRun:true,apply:true,installStatus:true},markers:{publicationSha256:f.publicationSha256,activationSha256:f.activationSha256},services:[{unit:"rz-admin.service",mainPid:f.adminPid,executable:{dev:f.adminDev,ino:f.adminIno,sha256:f.adminSha256}},{unit:"rz-monitor.service",mainPid:f.monitorPid,executable:{dev:f.monitorDev,ino:f.monitorIno,sha256:f.monitorSha256}}],health:[{service:"admin",buildId:f.buildId,compositionId:f.compositionId},{service:"monitor",buildId:f.buildId,compositionId:f.compositionId}],checks:{ownerLogin:true,defaultPasswordsRejected:true,insightsAbsent:true,reportsAbsent:true,restart:true,adminThenMonitor:true,monitorThenAdmin:true},runtime:true,browser:false,load:false,releaseReady:false}; await Bun.write(process.argv[11],monitorNativeRuntimeEvidenceBytes(evidence));' "$output/facts.json" "$output/published-certificate.json" "$release_result" "$output/verify.json" "$output/dry-run.json" "$output/apply.json" "$output/install-status.json" "$output/activate.json" "$output/publication-marker.json" "$output/activation-marker.json" "$output/monitor-native-runtime-evidence.json"
pnpm dlx bun@1.3.14 -e 'import { revalidateMonitorNativeRuntime } from "./distribution/monitor-native-runtime-revalidator.ts"; await revalidateMonitorNativeRuntime({evidencePath:process.argv[1],admissionPath:process.argv[2],releaseResultPath:process.argv[3],factsPath:process.argv[4],loginEvidencePath:process.argv[5],verifyPath:process.argv[6],dryRunPath:process.argv[7],applyPath:process.argv[8],statusPath:process.argv[9],activatePath:process.argv[10],publicationMarkerPath:process.argv[11],activationMarkerPath:process.argv[12]});' "$output/monitor-native-runtime-evidence.json" "$output/published-certificate.json" "$release_result" "$output/facts.json" "$output/login-evidence.json" "$output/verify.json" "$output/dry-run.json" "$output/apply.json" "$output/install-status.json" "$output/activate.json" "$output/publication-marker.json" "$output/activation-marker.json"
if test -n "${retain_output:-}"; then
  host_port="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "19801/tcp") 0).HostPort}}' "$name")"
  host_ip="$(docker inspect --format '{{(index (index .NetworkSettings.Ports "19801/tcp") 0).HostIp}}' "$name")"
  test "$host_ip" = 127.0.0.1 && test "$host_port" -gt 0
  pnpm dlx bun@1.3.14 -e 'import { canonicalJson } from "./distribution/release-manifest-core.ts"; const [out,name,id,image,port,evidence,ownerToken]=process.argv.slice(1); await Bun.write(out,canonicalJson({containerId:id,containerName:name,containerPort:19801,hostPort:Number(port),imageId:image,nativeEvidence:evidence,ownerToken}));' "$retain_output" "$name" "$(docker inspect --format '{{.Id}}' "$name")" "$(docker inspect --format '{{.Image}}' "$name")" "$host_port" "$output/monitor-native-runtime-evidence.json" "$retain_owner"
fi
echo "P8e exact-artifact Linux/amd64 PID1 runtime gate PASS: $output/monitor-native-runtime-evidence.json"
