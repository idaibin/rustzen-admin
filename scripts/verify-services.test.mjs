import { expect, test } from "bun:test";

const script = await Bun.file(new URL("./verify-services.sh", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();

test("four-service verifier binds Monitor's selected database before every controller start", () => {
    const identityNames = [
        "RUSTZEN_BUILD_ID",
        "RUSTZEN_COMPOSITION_ID",
        "RUSTZEN_MONITOR_SCHEMA_FINGERPRINT",
        "RUSTZEN_MONITOR_DATA_CONTRACT_ID",
    ];
    for (const name of identityNames) {
        const value = script.match(new RegExp(`^export ${name}=([a-f0-9]{64})$`, "m"))?.[1];
        expect(value).toMatch(/^[a-f0-9]{64}$/);
    }

    const init = script.indexOf('"$MONITOR" init-db');
    const bind = script.indexOf('"$MONITOR" bind-database');
    const validate = script.indexOf('"$MONITOR" validate-database');
    const adminAlone = script.indexOf('PHASE="admin-alone"');
    const orders = script.indexOf("ORDERS");
    expect(script.match(/"\$MONITOR" init-db/g)).toHaveLength(1);
    expect(script.match(/"\$MONITOR" bind-database/g)).toHaveLength(1);
    expect(script.match(/"\$MONITOR" validate-database/g)).toHaveLength(1);
    expect(init).toBeGreaterThan(-1);
    expect(bind).toBeGreaterThan(init);
    expect(validate).toBeGreaterThan(bind);
    expect(adminAlone).toBeGreaterThan(validate);
    expect(orders).toBeGreaterThan(validate);
    const startService = script.slice(script.indexOf("start_service()"), script.indexOf("\nstop_service()"));
    expect(startService).toContain('monitor) "$MONITOR" controller >"$log" 2>&1 & ;;');
    expect(startService).not.toMatch(/monitor\).*\b(?:init-db|bind-database|validate-database)\b/);
});

test("public verifier target runs the contract harness and current authority test", () => {
    const target = justfile.slice(justfile.indexOf("verify-services:"), justfile.indexOf("\nverify-modules-mvp:"));
    expect(target).toContain("pnpm dlx bun@1.3.14 test scripts/verify-services.test.mjs");
    expect(target).toContain("gateway_fails_closed_when_the_authority_database_is_closed");
    expect(target).not.toContain("warm_gateway_streams_with_memory_auth_and_a_closed_database");
});
