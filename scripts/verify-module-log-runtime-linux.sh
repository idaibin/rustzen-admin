#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
docker_bin=${RUSTZEN_MODULE_LOG_DOCKER:-docker}
bounded_child_pid=
bounded_watchdog_pid=
bounded_timeout_marker=
cleanup_timeout=${RUSTZEN_MODULE_LOG_CLEANUP_TIMEOUT:-5}
atomic_replace_symlink() {
  case "$(uname -s)" in
    Darwin|FreeBSD) mv -fh "$1" "$2" ;;
    *) mv -Tf "$1" "$2" ;;
  esac
}
process_descendants() {
  local parent=$1 child
  while IFS= read -r child; do
    [ -n "$child" ] || continue
    process_descendants "$child"
    printf '%s\n' "$child"
  done < <(pgrep -P "$parent" 2>/dev/null || true)
}
stop_process_tree() {
  local parent=${1:-} child tree
  [ -n "$parent" ] || return 0
  tree=$(process_descendants "$parent")
  while IFS= read -r child; do [ -z "$child" ] || kill -TERM "$child" 2>/dev/null || true; done <<<"$tree"
  kill -TERM "$parent" 2>/dev/null || true
  for _ in $(seq 1 20); do
    kill -0 "$parent" 2>/dev/null || return 0
    sleep .05
  done
  tree=$(process_descendants "$parent")
  while IFS= read -r child; do [ -z "$child" ] || kill -KILL "$child" 2>/dev/null || true; done <<<"$tree"
  kill -KILL "$parent" 2>/dev/null || true
}
run_bounded() {
  seconds=$1
  shift
  bounded_timeout_marker=$(mktemp "${TMPDIR:-/tmp}/rz-module-log-timeout.XXXXXX")
  rm -f "$bounded_timeout_marker"
  "$@" & bounded_child_pid=$!
  local child=$bounded_child_pid marker=$bounded_timeout_marker
  ( sleep "$seconds"; : >"$marker"; stop_process_tree "$child" ) & bounded_watchdog_pid=$!
  if wait "$bounded_child_pid"; then command_status=0; else command_status=$?; fi
  stop_process_tree "$bounded_watchdog_pid"
  wait "$bounded_watchdog_pid" 2>/dev/null || true
  if [ -e "$bounded_timeout_marker" ]; then command_status=124; fi
  rm -f "$bounded_timeout_marker"
  bounded_child_pid=
  bounded_watchdog_pid=
  bounded_timeout_marker=
  return "$command_status"
}
run_bounded_capture() {
  capture=$(mktemp "${TMPDIR:-/tmp}/rz-module-log-command.XXXXXX")
  if run_bounded "$@" >"$capture"; then status=0; else status=$?; fi
  cat "$capture"
  rm -f "$capture"
  return "$status"
}

cleanup_with_status() {
  result=$1
  trap - EXIT INT TERM
  stop_process_tree "$bounded_watchdog_pid"
  stop_process_tree "$bounded_child_pid"
  [ -z "$bounded_watchdog_pid" ] || wait "$bounded_watchdog_pid" 2>/dev/null || true
  [ -z "$bounded_child_pid" ] || wait "$bounded_child_pid" 2>/dev/null || true
  rm -f "$bounded_timeout_marker"
  bounded_watchdog_pid=
  bounded_child_pid=
  bounded_timeout_marker=
  if [ -L "$current" ] && [ "$(readlink "$current")" = "runs/$run_id" ]; then
    published=1
  fi
  evidence_source=
  if [ -d "$candidate" ]; then evidence_source=$candidate; elif [ "$published" = 0 ] && [ -d "$published_run" ]; then evidence_source=$published_run; fi
  if [ "$result" -ne 0 ] && [ -n "$evidence_source" ]; then
    run_bounded "$cleanup_timeout" "$docker_bin" logs "$container" >"$evidence_source/container.log" 2>&1 || true
    mv "$evidence_source" "$failed_run"
  fi
  rm -rf "$staged" "$publish_link"
  [ -z "$verify_tmp" ] || rm -rf "$verify_tmp"
  run_bounded "$cleanup_timeout" "$docker_bin" rm -f "$container" >/dev/null 2>&1 || true
  rm -rf "$lock"
  exit "$result"
}

verify_denial_receipts() {
  local evidence=$1 body
  jq -e -s '
    length == 5 and ([.[].endpoint] | sort) == ["backup","cleanup-confirm","cleanup-preview","list","tail"] and
    ([.[].bodyFile] | sort) == ["denied-backup.json","denied-cleanup-confirm.json","denied-cleanup-preview.json","denied-list.json","denied-tail.json"] and
    ([.[] | .status == 403] | all)
  ' "$evidence/non-owner-denials.jsonl" >/dev/null
  while IFS= read -r body; do
    jq -e 'type == "object" and (keys == ["code","data","message"]) and
      .code == 403 and .message == "Permission denied" and .data == null' "$evidence/$body" >/dev/null
    ! grep -Eq 'Server started successfully|Monitor Controller started|Insights service started|Reports service started|/opt/rz/logs|admin\.[0-9]{4}-|monitor\.[0-9]{4}-|insights\.[0-9]{4}-|reports\.[0-9]{4}-|module-log-runtime-(jwt|ipc|agent|reports)' "$evidence/$body"
  done < <(jq -r .bodyFile "$evidence/non-owner-denials.jsonl")
}

verify_identity_receipts() {
  local evidence=$1 manifest="$1/manifest.json" service pid uid gid status_file name status_pid status_uid status_gid
  local key expected_path process_uid process_gid directory_uid directory_gid status_groups log_control_gid
  log_control_gid=$(awk -F '\t' '$1 == "/opt/rz/logs/admin" {print $3}' "$evidence/directory-identities.tsv")
  [ "$(cut -f1 "$evidence/process-identities.tsv" | LC_ALL=C sort | tr '\n' ' ')" = "admin insights monitor reports " ]
  while IFS=$'\t' read -r service pid uid gid status_file; do
    [ "$status_file" = "process-$service.status" ]
    name=$(awk '/^Name:/{print $2}' "$evidence/$status_file")
    status_pid=$(awk '/^Pid:/{print $2}' "$evidence/$status_file")
    status_uid=$(awk '/^Uid:/{print $2}' "$evidence/$status_file")
    status_gid=$(awk '/^Gid:/{print $2}' "$evidence/$status_file")
    status_groups=$(awk '/^Groups:/{sub(/^Groups:[[:space:]]*/, ""); print}' "$evidence/$status_file")
    [ "$name" = "rz-$service" ]
    [ "$status_pid" = "$pid" ] && [ "$status_uid" = "$uid" ] && [ "$status_gid" = "$gid" ]
    case " $status_groups " in
      *" $log_control_gid "*) [ "$service" = admin ] ;;
      *) [ "$service" != admin ] ;;
    esac
    jq -e --arg service "$service" --argjson pid "$pid" --argjson uid "$uid" --argjson gid "$gid" --arg file "$status_file" \
      'any(.processes[]; .service == $service and .pid == $pid and .uid == $uid and .gid == $gid and .statusFile == $file)' "$manifest" >/dev/null
  done <"$evidence/process-identities.tsv"

  [ "$(wc -l <"$evidence/directory-identities.tsv" | tr -d '[:space:]')" = 5 ]
  while IFS=$'\t' read -r path uid gid mode; do
    case "$path" in
      /opt/rz/logs)
        jq -e --arg path "$path" --argjson uid "$uid" --argjson gid "$gid" --arg mode "0$mode" \
          '.directories.root == {path:$path,uid:$uid,gid:$gid,mode:$mode}' "$manifest" >/dev/null
        continue
        ;;
      /opt/rz/logs/admin|/opt/rz/logs/monitor|/opt/rz/logs/insights|/opt/rz/logs/reports)
        key=${path##*/}
        ;;
      *) return 1 ;;
    esac
    jq -e --arg key "$key" --arg path "$path" --argjson uid "$uid" --argjson gid "$gid" --arg mode "0$mode" \
      'any(.directories.modules[]; .module == $key and .path == $path and .uid == $uid and .gid == $gid and .mode == $mode)' "$manifest" >/dev/null
  done <"$evidence/directory-identities.tsv"

  [ "$(cut -f1 "$evidence/current-file-identities.tsv" | LC_ALL=C sort | tr '\n' ' ')" = "admin insights monitor reports " ]
  while IFS=$'\t' read -r service path inode uid gid mode; do
    expected_path="/opt/rz/logs/$service/$service.$(jq -r .utc.startDate "$manifest")"
    [ "$path" = "$expected_path" ] && [ "$mode" = 640 ] && [ "$inode" -gt 0 ]
    jq -e --arg service "$service" --arg path "$path" --argjson inode "$inode" --argjson uid "$uid" --argjson gid "$gid" --arg mode "$mode" \
      'any(.cleanup.currentBefore[]; .module == $service and .path == $path and .inode == $inode and .uid == $uid and .gid == $gid and .mode == $mode)' "$manifest" >/dev/null
    process_uid=$(awk -F '\t' -v service="$service" '$1 == service {print $3}' "$evidence/process-identities.tsv")
    process_gid=$(awk -F '\t' -v service="$service" '$1 == service {print $4}' "$evidence/process-identities.tsv")
    directory_uid=$(awk -F '\t' -v path="/opt/rz/logs/$service" '$1 == path {print $2}' "$evidence/directory-identities.tsv")
    directory_gid=$(awk -F '\t' -v path="/opt/rz/logs/$service" '$1 == path {print $3}' "$evidence/directory-identities.tsv")
    [ "$uid" = "$process_uid" ] && [ "$uid" = "$directory_uid" ] && [ "$gid" = "$directory_gid" ] &&
      [ "$gid" = "$(awk -F '\t' '$1 == "/opt/rz/logs/admin" {print $3}' "$evidence/directory-identities.tsv")" ] &&
      [ "$uid" -gt 0 ] && [ "$gid" -gt 0 ]
  done <"$evidence/current-file-identities.tsv"
}

publish_candidate() {
  mv "$candidate" "$published_run"
  ln -s "runs/$run_id" "$publish_link"
  atomic_replace_symlink "$publish_link" "$current"
  published=1
}

case "$cleanup_timeout" in ''|*[!0-9]*) echo 'RUSTZEN_MODULE_LOG_CLEANUP_TIMEOUT must be a positive integer' >&2; exit 2;; esac
[ "$cleanup_timeout" -gt 0 ] && [ "$cleanup_timeout" -le 30 ] || { echo 'RUSTZEN_MODULE_LOG_CLEANUP_TIMEOUT must be 1..30 seconds' >&2; exit 2; }

if [ -n "${RUSTZEN_MODULE_LOG_TEST_BOUNDED:-}" ]; then
  case "$RUSTZEN_MODULE_LOG_TEST_BOUNDED" in run|logs|rm) ;; *) exit 2 ;; esac
  set +e
  run_bounded 1 "$docker_bin" "$RUSTZEN_MODULE_LOG_TEST_BOUNDED" module-log-test-container
  status=$?
  set -e
  exit "$status"
fi

if [ -n "${RUSTZEN_MODULE_LOG_TEST_VERIFY:-}" ]; then
  test_root=${RUSTZEN_MODULE_LOG_TEST_ROOT:?missing test root}
  case "$RUSTZEN_MODULE_LOG_TEST_VERIFY" in
    denials) verify_denial_receipts "$test_root" ;;
    identity) verify_identity_receipts "$test_root" ;;
    *) exit 2 ;;
  esac
  exit 0
fi

if [ -n "${RUSTZEN_MODULE_LOG_TEST_SIGNAL:-}" ]; then
  test_root=${RUSTZEN_MODULE_LOG_TEST_ROOT:?missing test root}
  run_id=test-signal
  evidence_root=$test_root
  current="$evidence_root/current"
  candidate="$evidence_root/.candidate-$run_id"
  staged="$evidence_root/.binaries-$run_id"
  published_run="$evidence_root/runs/$run_id"
  publish_link="$evidence_root/.current-$run_id"
  failed_run="$evidence_root/failed-runs/$run_id"
  lock="$evidence_root/.verify.lock"
  container=module-log-test-container
  verify_tmp=
  published=0
  mkdir -p "$candidate" "$staged" "$lock" "$evidence_root/runs" "$evidence_root/failed-runs"
  trap 'cleanup_with_status $?' EXIT
  trap 'cleanup_with_status 130' INT
  trap 'cleanup_with_status 143' TERM
  outer_pid=$$
  ( sleep .2; kill -"$RUSTZEN_MODULE_LOG_TEST_SIGNAL" "$outer_pid" ) &
  run_bounded 30 "$docker_bin" run module-log-test-container
  exit 99
fi

if [ -n "${RUSTZEN_MODULE_LOG_TEST_PUBLISH:-}" ]; then
  test_root=${RUSTZEN_MODULE_LOG_TEST_ROOT:?missing test root}
  run_id=test-publish
  evidence_root=$test_root
  current="$evidence_root/current"
  candidate="$evidence_root/.candidate-$run_id"
  staged="$evidence_root/.binaries-$run_id"
  published_run="$evidence_root/runs/$run_id"
  publish_link="$evidence_root/.current-$run_id"
  failed_run="$evidence_root/failed-runs/$run_id"
  lock="$evidence_root/.verify.lock"
  container=module-log-test-container
  verify_tmp=
  published=0
  mkdir -p "$candidate" "$staged" "$lock" "$evidence_root/runs/old" "$evidence_root/failed-runs"
  printf old >"$evidence_root/runs/old/marker"
  ln -s runs/old "$current"
  printf new >"$candidate/marker"
  trap 'cleanup_with_status $?' EXIT
  trap 'cleanup_with_status 130' INT
  trap 'cleanup_with_status 143' TERM
  publish_candidate
  [ -L "$current" ] && [ "$(readlink "$current")" = "runs/$run_id" ] && [ "$(cat "$current/marker")" = new ]
  exit 0
fi

architecture=$(run_bounded_capture 10 "$docker_bin" info --format '{{.Architecture}}') || {
  echo "Docker architecture discovery failed or exceeded 10 seconds" >&2
  exit 1
}
case "$architecture" in
  aarch64) platform=linux/arm64; target_triple=aarch64-unknown-linux-musl; file_pattern='ELF 64-bit.*ARM aarch64' ;;
  x86_64) platform=linux/amd64; target_triple=x86_64-unknown-linux-musl; file_pattern='ELF 64-bit.*x86-64' ;;
  *) echo "unsupported Colima/Docker architecture: $architecture" >&2; exit 1 ;;
esac
timeout=${RUSTZEN_MODULE_LOG_RUN_TIMEOUT:-240}
case "$timeout" in ''|*[!0-9]*) echo 'RUSTZEN_MODULE_LOG_RUN_TIMEOUT must be a positive integer' >&2; exit 2;; esac
[ "$timeout" -gt 0 ] && [ "$timeout" -le 600 ] || { echo 'RUSTZEN_MODULE_LOG_RUN_TIMEOUT must be 1..600 seconds' >&2; exit 2; }
for unit in rz-admin rz-monitor rz-insights rz-reports; do
  grep -Fqx 'UMask=0027' "$root/deploy/$unit.service" || { echo "$unit.service must declare UMask=0027" >&2; exit 1; }
done

read -r verifier_image verifier_key verifier_provenance_sha < <(RUSTZEN_UI_VERIFIER_DOCKER="$docker_bin" "$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform")
bin_dir=${RUSTZEN_UI_LINUX_BIN_DIR:-"$root/target/rz/build/$architecture/bin"}
evidence_root="$root/target/rz/module-log-runtime"
current="$evidence_root/current"
lock="$evidence_root/.verify.lock"
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$"
candidate="$evidence_root/.candidate-$run_id"
staged="$evidence_root/.binaries-$run_id"
container="rz-module-log-runtime-$run_id"
published_run="$evidence_root/runs/$run_id"
publish_link="$evidence_root/.current-$run_id"
failed_run="$evidence_root/failed-runs/$run_id"
verify_tmp=
published=0
mkdir -p "$evidence_root/runs" "$evidence_root/failed-runs"
mkdir "$lock" 2>/dev/null || { echo "another module-log runtime verification owns $lock" >&2; exit 1; }
mkdir "$candidate" "$staged"

trap 'cleanup_with_status $?' EXIT
trap 'cleanup_with_status 130' INT
trap 'cleanup_with_status 143' TERM

read -r head initial_state initial_sha < <("$root/scripts/admin-browser-source-identity.sh")
test -f "$bin_dir/build-provenance.txt" || { echo "missing Linux build provenance; run just build-admin-browser-linux" >&2; exit 1; }
expected="$candidate/build-provenance.txt"
{
  printf 'schemaVersion\t1\n'
  printf 'gitHead\t%s\n' "$head"
  printf 'sourceTreeState\t%s\n' "$initial_state"
  printf 'sourceTreeSha256\t%s\n' "$initial_sha"
  printf 'architecture\t%s\n' "$architecture"
  printf 'targetTriple\t%s\n' "$target_triple"
  printf 'platform\t%s\n' "$platform"
  printf 'distribution\tfull\n'
  for name in rz-admin rz-monitor rz-insights rz-reports; do
    binary="$bin_dir/$name"
    test -x "$binary" || { echo "missing $architecture Linux binary: $binary" >&2; exit 1; }
    file "$binary" | grep -q "$file_pattern" || { echo "binary does not match $architecture Linux: $binary" >&2; exit 1; }
    printf '%s\t%s\n' "$name" "$(shasum -a 256 "$binary" | awk '{print $1}')"
  done
} >"$expected"
cmp "$expected" "$bin_dir/build-provenance.txt" || { echo "Linux binaries do not match current source-tree provenance" >&2; exit 1; }
binary_hashes='{}'
for name in rz-admin rz-monitor rz-insights rz-reports; do
  cp "$bin_dir/$name" "$staged/$name"
  hash=$(shasum -a 256 "$staged/$name" | awk '{print $1}')
  test "$hash" = "$(awk -F '\t' -v name="$name" '$1 == name {print $2}' "$expected")"
  binary_hashes=$(jq -nc --argjson current "$binary_hashes" --arg name "$name" --arg hash "$hash" '$current + {($name):$hash}')
done

run_bounded "$timeout" "$docker_bin" run --name "$container" --platform "$platform" \
  --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$initial_state" \
  --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$initial_sha" --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" \
  --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" \
  --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_provenance_sha" \
  --env RUSTZEN_VERIFY_BINARY_HASHES="$binary_hashes" \
  --mount "type=bind,src=$staged,dst=/verify/bin,readonly" \
  --mount "type=bind,src=$candidate,dst=/verify/evidence" \
  --mount "type=bind,src=$root/scripts/verify-module-log-runtime-linux-inner.sh,dst=/verify/run.sh,readonly" \
  "$verifier_image" bash /verify/run.sh

read -r final_head final_state final_sha < <("$root/scripts/admin-browser-source-identity.sh")
[ "$final_head" = "$head" ] && [ "$final_state" = "$initial_state" ] && [ "$final_sha" = "$initial_sha" ] || {
  echo "source tree changed during module-log runtime verification" >&2
  exit 1
}
test -f "$candidate/manifest.json"
jq -e --arg head "$head" --arg sourceSha "$initial_sha" --arg architecture "$architecture" '
  .schemaVersion == 1 and .gitHead == $head and .sourceTreeSha256 == $sourceSha and .architecture == $architecture and
  (.binaryHashes | keys | sort) == ["rz-admin","rz-insights","rz-monitor","rz-reports"] and
  (.processes | length) == 4 and ([.processes[] | .uid > 0 and .gid > 0] | all) and
  .directories.root.mode == "0711" and (.directories.modules | length) == 4 and
  ([.directories.modules[] | .uid > 0 and .gid > 0 and .mode == "02770"] | all) and
  (.logFiles | length) == 4 and (.archive.fileCount == 4) and (.archive.manifest | length) == 4 and
  (.cleanup.result.removed | length) == 4 and (.cleanup.result.failures | length) == 0 and
  .utc.startDate == .utc.endDate and .cleanup.currentBefore == .cleanup.currentAfter and
  .umask == "0027" and (.receipts | length) == 25 and ([.receipts[].file] | unique | length) == 25
' "$candidate/manifest.json" >/dev/null
for module in admin monitor insights reports; do
  phrase=$(case "$module" in admin) echo 'Server started successfully';; monitor) echo 'Monitor Controller started';; insights) echo 'Insights service started';; reports) echo 'Reports service started';; esac)
  jq -e --arg module_id "$module" --arg date "$(jq -r .utc.startDate "$candidate/manifest.json")" --arg phrase "$phrase" \
    '.data.module == $module_id and .data.date == $date and (.data.content | contains($phrase))' "$candidate/tail-$module.json" >/dev/null
done
jq -e --arg date "$(jq -r .utc.startDate "$candidate/manifest.json")" '
  [.data[] | select(.date == $date and .active and .readable) | .module] | sort == ["admin","insights","monitor","reports"]
' "$candidate/list-owner.json" >/dev/null
verify_denial_receipts "$candidate"
verify_identity_receipts "$candidate"
for receipt in process-identities.tsv process-admin.status process-monitor.status process-insights.status process-reports.status directory-identities.tsv current-file-identities.tsv; do
  jq -e --arg receipt "$receipt" 'any(.receipts[]; .file == $receipt)' "$candidate/manifest.json" >/dev/null
done

archive="$candidate/rustzen-module-logs.tar"
header_hash=$(awk '/^x-rustzen-archive-sha256:/{gsub("\\r", "", $2); print $2}' "$candidate/archive.headers")
header_count=$(awk '/^x-rustzen-archive-file-count:/{gsub("\\r", "", $2); print $2}' "$candidate/archive.headers")
[ "$header_count" = 4 ]
[ "$header_hash" = "$(shasum -a 256 "$archive" | awk '{print $1}')" ]
[ "$header_hash" = "$(jq -r .archive.sha256 "$candidate/manifest.json")" ]
verify_tmp=$(mktemp -d "${TMPDIR:-/tmp}/rz-module-log-archive.XXXXXX")
tar -tf "$archive" | LC_ALL=C sort >"$verify_tmp/members"
printf '%s\n' admin."$(jq -r .utc.startDate "$candidate/manifest.json")" insights."$(jq -r .utc.startDate "$candidate/manifest.json")" manifest.json monitor."$(jq -r .utc.startDate "$candidate/manifest.json")" reports."$(jq -r .utc.startDate "$candidate/manifest.json")" >"$verify_tmp/expected"
cmp "$verify_tmp/expected" "$verify_tmp/members"
[ "$(wc -l <"$verify_tmp/members")" = "$(sort -u "$verify_tmp/members" | wc -l)" ]
tar -xf "$archive" -C "$verify_tmp"
cmp "$candidate/archive-manifest.json" "$verify_tmp/manifest.json"
archive_date=$(jq -r .utc.startDate "$candidate/manifest.json")
for module in admin monitor insights reports; do
  member="$module.$archive_date"
  cmp "$candidate/source-$module.log" "$verify_tmp/$member"
  jq -e --arg module_id "$module" --arg member "$member" \
    --arg sha "$(shasum -a 256 "$verify_tmp/$member" | awk '{print $1}')" \
    --argjson bytes "$(wc -c <"$verify_tmp/$member")" '
      any(.[]; .module == $module_id and .file_name == $member and .sha256 == $sha and .size_bytes == $bytes)
    ' "$candidate/archive-manifest.json" >/dev/null
done
cmp "$candidate/current-before.json" "$candidate/current-after.json"
jq -e '
  length == 4 and ([.[] | .module] | sort) == ["admin","insights","monitor","reports"] and
  ([.[] | .inode > 0 and (.mode == "640") and .uid > 0 and .gid > 0] | all)
' "$candidate/current-before.json" >/dev/null
jq -e --arg date "$(jq -r .cleanup.oldDate "$candidate/manifest.json")" '
  .data.partial == false and (.data.removed | length) == 4 and (.data.failures | length) == 0 and
  ([.data.removed[] | select(.date == $date) | .module] | sort) == ["admin","insights","monitor","reports"]
' "$candidate/cleanup-result.json" >/dev/null
while IFS=$'\t' read -r file sha bytes; do
  [ "$(shasum -a 256 "$candidate/$file" | awk '{print $1}')" = "$sha" ]
  [ "$(wc -c <"$candidate/$file" | tr -d '[:space:]')" = "$bytes" ]
done < <(jq -r '.receipts[] | [.file,.sha256,.bytes] | @tsv' "$candidate/manifest.json")
for name in rz-admin rz-monitor rz-insights rz-reports; do
  [ "$(shasum -a 256 "$staged/$name" | awk '{print $1}')" = "$(jq -r --arg name "$name" '.binaryHashes[$name]' "$candidate/manifest.json")" ]
done

publish_candidate
echo "module-log runtime evidence published: $current/manifest.json"
