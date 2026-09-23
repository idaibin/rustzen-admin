# shellcheck shell=sh
# Sourced after lifecycle cleanup and HTTP helpers are installed.

parse_json() {
    expression="$1"
    run_bun -e "const value = JSON.parse(await Bun.stdin.text()); console.log($expression)"
}

login() {
    username="$1"
    password="$2"
    response="$(curl --fail --silent --show-error \
        -H 'content-type: application/json' \
        -d "{\"username\":\"$username\",\"password\":\"$password\"}" \
        "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/auth/login")"
    printf '%s' "$response"
}

wait_for_module_state() {
    module="$1"
    available="$2"
    compatible="$3"
    count=0
    while [ "$count" -lt 180 ]; do
        body="$(curl --silent --show-error \
            -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" \
            "http://127.0.0.1:$RUSTZEN_ADMIN_PORT/api/system/modules" 2>/dev/null || true)"
        if BODY="$body" MODULE_ID="$module" EXPECT_AVAILABLE="$available" \
            EXPECT_COMPATIBLE="$compatible" run_bun -e '
                try {
                    const payload = JSON.parse(process.env.BODY);
                    const module = payload.data?.find((item) => item.id === process.env.MODULE_ID);
                    const matches = module
                        && String(module.available) === process.env.EXPECT_AVAILABLE
                        && String(module.compatible) === process.env.EXPECT_COMPATIBLE;
                    process.exit(matches ? 0 : 1);
                } catch {
                    process.exit(1);
                }
            '
        then
            return 0
        fi
        count=$((count + 1))
        sleep 0.1
    done
    echo "verify-services: module state did not converge: $module available=$available compatible=$compatible" >&2
    dump_logs
    exit 1
}

module_gateway_url() {
    case "$1" in
        monitor) printf 'http://127.0.0.1:%s/api/monitor/nodes\n' "$RUSTZEN_ADMIN_PORT" ;;
        insights) printf 'http://127.0.0.1:%s/api/insights/overview\n' "$RUSTZEN_ADMIN_PORT" ;;
        reports) printf 'http://127.0.0.1:%s/api/reports/systems\n' "$RUSTZEN_ADMIN_PORT" ;;
        *) return 1 ;;
    esac
}

assert_gateway_unavailable() {
    module="$1"
    url="$(module_gateway_url "$module")"
    wait_for_status 503 "$url" "$RUSTZEN_ADMIN_TOKEN"
    body="$(curl --silent --show-error \
        -H "authorization: Bearer $RUSTZEN_ADMIN_TOKEN" "$url" 2>/dev/null || true)"
    BODY="$body" MODULE_ID="$module" run_bun -e '
        const payload = JSON.parse(process.env.BODY);
        const expected = `${process.env.MODULE_ID} worker is temporarily unavailable.`;
        if (payload.code !== 40001 || payload.message !== expected || payload.data !== null) {
            throw new Error(`invalid unavailable envelope: ${JSON.stringify(payload)}`);
        }
    '
}

assert_module_gateways_healthy_except() {
    excluded="$1"
    for module in monitor insights reports; do
        if [ "$module" != "$excluded" ]; then
            wait_for_status 200 "$(module_gateway_url "$module")" "$RUSTZEN_ADMIN_TOKEN"
        fi
    done
}
