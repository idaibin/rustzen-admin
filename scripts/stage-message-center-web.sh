#!/usr/bin/env bash
set -euo pipefail

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
output=${1:?usage: stage-message-center-web.sh OUTPUT}
bun=(pnpm dlx bun@1.3.14)
notify_selection=distribution/fixtures/monitor-notify.json
pure_selection=distribution/fixtures/monitor.json

notify_id=$("${bun[@]}" scripts/distribution-resolve.ts resolve --selection "$notify_selection" \
  | "${bun[@]}" -e 'console.log((await Bun.stdin.json()).compositionId)')
pure_id=$("${bun[@]}" scripts/distribution-resolve.ts resolve --selection "$pure_selection" \
  | "${bun[@]}" -e 'console.log((await Bun.stdin.json()).compositionId)')

"${bun[@]}" scripts/distribution-build-web.ts --selection "$notify_selection"
"${bun[@]}" scripts/distribution-verify-web.ts --selection "$notify_selection"
"${bun[@]}" scripts/distribution-build-web.ts --selection "$pure_selection"
"${bun[@]}" scripts/distribution-verify-web.ts --selection "$pure_selection"

mkdir -p "$output/notify/dist" "$output/pure/dist"
cp "target/distributions/$notify_id/web/inventory.json" "$output/notify/inventory.json"
cp -R "target/distributions/$notify_id/web/dist/." "$output/notify/dist"
cp "target/distributions/$pure_id/web/inventory.json" "$output/pure/inventory.json"
cp -R "target/distributions/$pure_id/web/dist/." "$output/pure/dist"

jq -e '.preset == "monitor-notify" and (.moduleIds | any(contains("src/notifications/")))' \
  "$output/notify/inventory.json" >/dev/null
jq -e '.preset == "monitor" and (.moduleIds | all(contains("src/notifications/") | not))' \
  "$output/pure/inventory.json" >/dev/null
grep -R -a -q '/api/notifications/stream' "$output/notify/dist"
! grep -R -a -E -q '/api/notifications|Message center|消息中心' "$output/pure/dist"
