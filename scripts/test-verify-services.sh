#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
VERIFY="${VERIFY_SERVICES_SCRIPT:-$ROOT/scripts/verify-services.sh}"
JUSTFILE="${JUSTFILE_PATH:-$ROOT/justfile}"

bash -n "$VERIFY"

grep -Fqx 'AGENT="${6:-target/release/rz-monitor-agent}"' "$VERIFY"
grep -Fqx 'AGENT="$(absolute_binary "$AGENT")"' "$VERIFY"
grep -Fqx 'for binary in "$ADMIN" "$MONITOR" "$INSIGHTS" "$REPORTS" "$CLI" "$AGENT"; do' "$VERIFY"
grep -Fqx 'export RUSTZEN_MONITOR_NODE_ID=verify-monitor-node' "$VERIFY"
grep -Fqx '        monitor_agent) "$AGENT" >"$log" 2>&1 & ;;' "$VERIFY"
if grep -Fq 'monitor_agent) "$MONITOR" agent' "$VERIFY"; then
    echo "verify-services must start the independent rz-monitor-agent binary" >&2
    exit 1
fi

grep -Fqx '    cargo build --release -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent' "$JUSTFILE"
grep -Fqx '    cargo build -p rustzen-monitor --no-default-features --features agent --bin rz-monitor-agent' "$JUSTFILE"
grep -Fqx '    scripts/verify-services.sh target/release/rz-admin target/release/rz-monitor target/release/rz-insights target/release/rz-reports target/release/rz target/release/rz-monitor-agent' "$JUSTFILE"
grep -Fqx '    scripts/verify-services.sh target/debug/rz-admin target/debug/rz-monitor target/debug/rz-insights target/debug/rz-reports target/debug/rz target/debug/rz-monitor-agent' "$JUSTFILE"

echo "verify-services independent Monitor Agent wiring passed"
