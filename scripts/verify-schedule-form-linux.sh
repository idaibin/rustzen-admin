#!/usr/bin/env bash
set -euo pipefail
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
. "$root/scripts/schedule-form-evidence-lib.sh"
docker_bin=${RUSTZEN_SCHEDULE_FORM_DOCKER:-docker};
timeout=${RUSTZEN_SCHEDULE_FORM_TIMEOUT:-480};
info_timeout=${RUSTZEN_SCHEDULE_FORM_DOCKER_INFO_TIMEOUT:-10}
for value in "$timeout" "$info_timeout"
do case "$value" in ''|*[!0-9]*) echo 'schedule form timeouts must be positive integers' >&2;
exit 2;;
esac
done
[ "$timeout" -gt 0 ] && [ "$timeout" -le 900 ] || { echo 'RUSTZEN_SCHEDULE_FORM_TIMEOUT must be 1..900' >&2;
exit 2;
}
[ "$info_timeout" -gt 0 ] && [ "$info_timeout" -le 60 ] || { echo 'RUSTZEN_SCHEDULE_FORM_DOCKER_INFO_TIMEOUT must be 1..60' >&2;
exit 2;
}
run_bounded() { seconds=$1;
shift;
"$@" & command_pid=$!;
( sleep "$seconds";
kill -TERM "$command_pid" 2>/dev/null || true;
sleep 10;
kill -KILL "$command_pid" 2>/dev/null || true ) >/dev/null 2>&1 & watchdog_pid=$!;
if wait "$command_pid"
then command_status=0
else command_status=$?
fi;
kill "$watchdog_pid" 2>/dev/null || true;
wait "$watchdog_pid" 2>/dev/null || true;
return "$command_status";
}
run_bounded_capture() { capture=$(mktemp "${TMPDIR:-/tmp}/rz-schedule-form.XXXXXX") || return 1;
if run_bounded "$@" >"$capture"
then status=0
else status=$?
fi;
cat "$capture";
rm -f "$capture";
return "$status";
}
atomic_replace_symlink() { case "$(uname -s)" in Darwin|FreeBSD) mv -fh "$1" "$2";;
*) mv -Tf "$1" "$2";;
esac;
}
on_interrupt() { exit 130;
}
on_terminate() { exit 143;
}
seam_root() { printf '%s\n' "${RUSTZEN_SCHEDULE_FORM_TEST_ROOT:-$(mktemp -d "${TMPDIR:-/tmp}/rz-schedule-form-seam.XXXXXX")}";
}
if [ "${RUSTZEN_SCHEDULE_FORM_TEST_SETUP_FAILURE:-}" = 1 ]
then
  test_root=$(seam_root);
evidence_root="$test_root";
current="$test_root/current";
lock_dir="$test_root/.verify.lock";
candidate="$test_root/.candidate"
  cleanup_setup() { result=$?;
trap - EXIT INT TERM;
rm -rf "$candidate" "$lock_dir";
failed=0;
if [ "$result" -eq 0 ]
then echo 'setup unexpectedly passed' >&2;
failed=1
fi;
if ! test "$(readlink "$current")" = runs/old
then echo 'setup changed current' >&2;
failed=1
fi;
if test -e "$lock_dir" || test -e "$candidate"
then echo 'setup leaked state' >&2;
failed=1
fi;
[ -n "${RUSTZEN_SCHEDULE_FORM_TEST_ROOT:-}" ] || rm -rf "$test_root";
[ "$failed" -eq 0 ] || exit 99;
echo 'Schedule form setup failure seam passed' >&2;
exit "$result";
}
  mkdir -p "$test_root/runs/old" "$candidate";
printf old >"$test_root/runs/old/manifest.json";
ln -s runs/old "$current";
mkdir "$lock_dir";
trap cleanup_setup EXIT;
trap on_interrupt INT;
trap on_terminate TERM;
exit 1
fi
if [ "${RUSTZEN_SCHEDULE_FORM_TEST_SIGNAL:-}" = INT ] || [ "${RUSTZEN_SCHEDULE_FORM_TEST_SIGNAL:-}" = TERM ]
then
  test_root=$(seam_root);
test_signal=$RUSTZEN_SCHEDULE_FORM_TEST_SIGNAL;
current="$test_root/current";
lock_dir="$test_root/.verify.lock";
candidate="$test_root/.candidate"
  cleanup_signal() { result=$?;
trap - EXIT INT TERM;
rm -rf "$candidate" "$lock_dir";
expected=130;
[ "$test_signal" = TERM ] && expected=143;
failed=0;
if ! test "$result" = "$expected"
then echo 'signal exit code wrong' >&2;
failed=1
fi;
if ! test "$(readlink "$current")" = runs/old
then echo 'signal changed current' >&2;
failed=1
fi;
if test -e "$lock_dir" || test -e "$candidate"
then echo 'signal leaked state' >&2;
failed=1
fi;
[ -n "${RUSTZEN_SCHEDULE_FORM_TEST_ROOT:-}" ] || rm -rf "$test_root";
[ "$failed" -eq 0 ] || exit 99;
echo "Schedule form $test_signal signal seam passed" >&2;
exit "$result";
}
  mkdir -p "$test_root/runs/old" "$candidate";
printf old >"$test_root/runs/old/manifest.json";
ln -s runs/old "$current";
mkdir "$lock_dir";
trap cleanup_signal EXIT;
trap on_interrupt INT;
trap on_terminate TERM;
kill -"$test_signal" "$$"
fi
if [ "${RUSTZEN_SCHEDULE_FORM_TEST_PUBLISH_FAILURE:-}" = 1 ]
then
  test_root=$(mktemp -d "${TMPDIR:-/tmp}/rz-schedule-form-publish.XXXXXX");
mkdir -p "$test_root/runs/old" "$test_root/.candidate";
printf old >"$test_root/runs/old/manifest.json";
ln -s runs/old "$test_root/current";
printf new >"$test_root/.candidate/manifest.json"
  mv "$test_root/.candidate" "$test_root/runs/new" && ln -s runs/new "$test_root/.next" && atomic_replace_symlink "$test_root/.next" "$test_root/current";
test "$(readlink "$test_root/current")" = runs/new;
test "$(cat "$test_root/current/manifest.json")" = new
  if mv "$test_root/.missing" "$test_root/runs/failed"
then echo 'forced publish failure unexpectedly succeeded' >&2;
exit 1
fi
  test "$(readlink "$test_root/current")" = runs/new;
test "$(cat "$test_root/current/manifest.json")" = new;
rm -rf "$test_root";
echo 'Schedule form publication seams passed';
exit 0
fi
architecture=${RUSTZEN_UI_LINUX_ARCH:-};
if [ -z "$architecture" ]
then architecture=$(run_bounded_capture "$info_timeout" "$docker_bin" info --format '{{.Architecture}}') || { echo 'Docker architecture discovery failed or timed out' >&2;
exit 1;
}
fi
case "$architecture" in aarch64) platform=linux/arm64;
target_triple=aarch64-unknown-linux-musl
file_pattern='ELF 64-bit.*ARM aarch64';;
x86_64) platform=linux/amd64;
target_triple=x86_64-unknown-linux-musl
file_pattern='ELF 64-bit.*x86-64';;
*) echo "unsupported Docker architecture: $architecture" >&2;
exit 1;;
esac
read -r verifier_image verifier_key verifier_provenance_sha < <("$root/scripts/ensure-admin-browser-verifier-image.sh" --platform "$platform")
bin_dir=${RUSTZEN_UI_LINUX_BIN_DIR:-"$root/target/rz/build/$architecture/bin"};
read -r head tree_state tree_sha < <("$root/scripts/admin-browser-source-identity.sh")
evidence_root="$root/target/rz/schedule-form-browser";
current="$evidence_root/current";
lock_dir="$evidence_root/.verify.lock";
run_id="$(date -u +%Y%m%dT%H%M%SZ)-$$";
candidate="$evidence_root/.candidate-$run_id";
staged="$evidence_root/.binaries-$run_id";
container="rz-schedule-form-$run_id";
status=1
cleanup() { result=$?
trap - EXIT INT TERM
if [ -d "$candidate" ]; then
  "$docker_bin" logs "$container" >"$candidate/container.log" 2>&1 || true
fi
"$docker_bin" rm -f "$container" >/dev/null 2>&1 || true
rm -rf "$staged"
if [ "$result" -ne 0 ] && [ -d "$candidate" ]; then
  failed_dir="$evidence_root/failed-runs/$run_id"
  mkdir -p "$evidence_root/failed-runs"
  rm -rf "$failed_dir"
  mv "$candidate" "$failed_dir"
  echo "Schedule form failure evidence: $failed_dir" >&2
  [ ! -s "$failed_dir/container.log" ] || cat "$failed_dir/container.log" >&2
fi
rm -rf "$lock_dir"
exit "$result"
}
mkdir -p "$evidence_root/runs"
if [ -e "$current" ] && [ ! -L "$current" ]
then echo "refusing to replace legacy schedule form evidence directory" >&2;
exit 1
fi
if ! mkdir "$lock_dir" 2>/dev/null
then echo "another schedule form verification owns $lock_dir" >&2;
exit 1
fi;
trap cleanup EXIT;
trap 'exit 130' INT;
trap 'exit 143' TERM;
mkdir -p "$candidate" "$staged"
for name in rz-admin rz-monitor rz-insights rz-reports
do test -x "$bin_dir/$name" || { echo "missing Linux binary: $bin_dir/$name;
run just build-admin-browser-linux" >&2;
exit 1;
}
file "$bin_dir/$name" | grep -q "$file_pattern" || { echo "wrong Linux binary architecture: $name" >&2;
exit 1;
}
done
test -s "$bin_dir/build-provenance.txt" || { echo 'missing build provenance' >&2;
exit 1;
};
expected="$candidate/build-provenance.txt"
{ printf 'schemaVersion\t1\ngitHead\t%s\nsourceTreeState\t%s\nsourceTreeSha256\t%s\narchitecture\t%s\ntargetTriple\t%s\nplatform\t%s\ndistribution\tfull\n' "$head" "$tree_state" "$tree_sha" "$architecture" "$target_triple" "$platform";
for name in rz-admin rz-monitor rz-insights rz-reports
do printf '%s\t%s\n' "$name" "$(shasum -a 256 "$bin_dir/$name" | awk '{print $1}')"
done;
} >"$expected"
cmp -s "$expected" "$bin_dir/build-provenance.txt" || { echo 'Linux binaries do not match current source provenance' >&2;
exit 1;
};
for name in rz-admin rz-monitor rz-insights rz-reports
do cp "$bin_dir/$name" "$staged/$name"
done
pnpm dlx bun@1.3.14 "$root/scripts/schedule-form-browser-steps.mjs" >"$candidate/browser-steps.json"
run_bounded "$timeout" "$docker_bin" run --name "$container" --platform "$platform" --security-opt seccomp=unconfined --env RUSTZEN_VERIFY_HEAD="$head" --env RUSTZEN_VERIFY_SOURCE_TREE_STATE="$tree_state" --env RUSTZEN_VERIFY_SOURCE_TREE_SHA256="$tree_sha" --env RUSTZEN_VERIFY_ARCHITECTURE="$architecture" --env RUSTZEN_VERIFY_PLATFORM="$platform" --env RUSTZEN_VERIFY_CHROMIUM_VERSION=120.0.6099.224-1~deb11u1 --env RUSTZEN_VERIFY_VERIFIER_IMAGE_ID="$verifier_image" --env RUSTZEN_VERIFY_VERIFIER_KEY="$verifier_key" --env RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256="$verifier_provenance_sha" --mount "type=bind,src=$staged,dst=/verify/bin,readonly" --mount "type=bind,src=$candidate,dst=/verify/evidence" --mount "type=bind,src=$root/scripts/verify-schedule-form-linux-inner.sh,dst=/verify/run.sh,readonly" --mount "type=bind,src=$root/scripts/schedule-form-evidence-lib.sh,dst=/verify/schedule-form-evidence-lib.sh,readonly" --mount "type=bind,src=$root/scripts/admin-browser-fault-proxy.py,dst=/verify/fault-proxy.py,readonly" "$verifier_image" bash /verify/run.sh
verify_schedule_form_manifest "$candidate/manifest.json" "$head" "$tree_state" "$tree_sha" "$platform" "$candidate/browser-steps.json"
verify_schedule_form_receipts "$candidate" "$candidate/manifest.json" "$candidate/browser-steps.json"
verify_schedule_form_artifacts "$candidate" "$candidate/manifest.json"
read -r final_head final_state final_sha < <("$root/scripts/admin-browser-source-identity.sh");
test "$final_head" = "$head" && test "$final_state" = "$tree_state" && test "$final_sha" = "$tree_sha"
mv "$candidate" "$evidence_root/runs/$run_id" && ln -s "runs/$run_id" "$evidence_root/.current-$run_id" && atomic_replace_symlink "$evidence_root/.current-$run_id" "$current";
status=0;
rm -rf "$staged" "$lock_dir";
trap - EXIT INT TERM
echo "Schedule form Linux evidence published: $current/manifest.json"
