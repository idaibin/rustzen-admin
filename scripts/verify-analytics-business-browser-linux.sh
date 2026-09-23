#!/usr/bin/env bash
# P8f runs only against a fresh retained P8e analytics PID1 container.
set -euo pipefail
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"; cd "$root"
usage(){ echo "usage: $0 --context FILE --container-facts FILE --output NEW_DIRECTORY --signals DIRECTORY [--chromium PATH]" >&2; exit 2; }
test "$#" = 8 -o "$#" = 10 || usage; chromium=chromium
while test "$#" -gt 0; do case "$1" in --context) context="$2";;--container-facts) facts="$2";;--output) output="$2";;--signals) signals="$2";;--chromium) chromium="$2";;*) usage;;esac; shift 2; done
test -f "${context:-}" -a -f "${facts:-}" -a -d "${signals:-}" && test ! -e "${output:-}" || exit 2
command -v "$chromium" >/dev/null 2>&1 || { echo "chromium binary is unavailable: $chromium" >&2; exit 2; }
candidate="${output}.candidate-$$"; test ! -e "$candidate" || exit 2; mkdir "$candidate"
profile="$(mktemp -d "$candidate/.chrome.XXXXXX")"; log="$candidate/chrome.log"; chrome_pid=
stop_chrome(){ test -z "$chrome_pid" || { kill "$chrome_pid" 2>/dev/null || true; for _ in $(seq 1 100); do kill -0 "$chrome_pid" 2>/dev/null || break; sleep .05; done; kill -0 "$chrome_pid" 2>/dev/null && kill -KILL "$chrome_pid" 2>/dev/null || true; wait "$chrome_pid" 2>/dev/null || true; chrome_pid=; }; }
cleanup(){ status=$?; stop_chrome; rm -rf "$profile"; test "$status" -eq 0 || rm -rf "$candidate"; exit "$status"; }; trap cleanup EXIT INT TERM
verify_output(){ local directory=$1
  test "$(find "$directory" -mindepth 1 -maxdepth 1 -type f -exec basename {} \; | LC_ALL=C sort | tr '\n' ' ')" = 'details-empty-mobile.png details-populated.png error-state.png login-error.png overview-empty.png overview-loading.png overview-populated.png receipt.json ' || { echo "P8f output allowlist differs" >&2; return 1; }
  test "$(find "$directory" -mindepth 1 ! -type f -print -quit)" = "" || { echo "P8f output contains a non-file" >&2; return 1; }
  pnpm dlx bun@1.3.14 -e 'import { sha256 } from "./distribution/release-manifest-core.ts"; const [dir]=process.argv.slice(1), r=await Bun.file(`${dir}/receipt.json`).json(); for (const item of r.screenshots) { const bytes=new Uint8Array(await Bun.file(`${dir}/${item.file}`).arrayBuffer()), view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength); if (bytes.length!==item.bytes||sha256(bytes)!==item.sha256||new TextDecoder().decode(bytes.slice(1,4))!=="PNG"||view.getUint32(16)!==item.dimensions.width||view.getUint32(20)!==item.dimensions.height) throw Error("P8f screenshot reread differs") }' "$directory"
}
container="$(pnpm dlx bun@1.3.14 -e 'console.log((await Bun.file(process.argv[1]).json()).containerName)' "$context")"
docker exec "$container" sh -c 'systemctl is-active --quiet rz-admin.service; systemctl is-active --quiet rz-insights.service'
"$chromium" --headless --no-first-run --disable-gpu --remote-debugging-port=0 --user-data-dir="$profile" about:blank >"$log" 2>&1 & chrome_pid=$!
for _ in $(seq 1 100); do cdp="$(sed -n 's@.*127.0.0.1:\([0-9][0-9]*\)/devtools.*@\1@p' "$log"|tail -1)"; test -n "$cdp" && break; sleep .05; done
test -n "${cdp:-}" || { cat "$log" >&2; exit 1; }
pnpm dlx bun@1.3.14 scripts/analytics-business-browser-driver.ts --context "$context" --cdp "$cdp" --signals "$signals" --output "$candidate"
pnpm dlx bun@1.3.14 scripts/analytics-business-browser-receipt.ts "$candidate/receipt.json"
stop_chrome; rm -rf "$profile"; rm -f "$log"; verify_output "$candidate"; mv "$candidate" "$output"; verify_output "$output"
echo "P8f analytics business browser closure PASS: $output/receipt.json"
