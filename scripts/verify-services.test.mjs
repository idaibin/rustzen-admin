import { expect, test } from "bun:test";
import { mkdtemp, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const script = await Bun.file(new URL("./verify-services.sh", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const moduleLogHelper = await Bun.file(new URL("./verify-module-log-diagnostics.sh", import.meta.url)).text();

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

const latencyContractCommand = "pnpm dlx bun@1.3.14 test scripts/gateway-latency-contract.test.mjs scripts/verify-insights-scenarios.test.mjs scripts/verify-reports-scenarios.test.mjs scripts/verify-worker-contracts.test.mjs scripts/verify-services.test.mjs";

function justRecipe(name, nextName) {
    return justfile.slice(justfile.indexOf(`${name}:`), justfile.indexOf(`\n${nextName}:`));
}

test("public verifier targets run pinned latency contracts before Rust builds", () => {
    const serviceTarget = justRecipe("verify-services", "verify-modules-mvp");
    const mvpTarget = justRecipe("verify-modules-mvp", "contract-generate");
    for (const target of [serviceTarget, mvpTarget]) {
        expect(target).toContain(latencyContractCommand);
        expect(target.indexOf(latencyContractCommand)).toBeLessThan(target.indexOf("cargo build"));
    }
    expect(serviceTarget).toContain("gateway_fails_closed_when_the_authority_database_is_closed");
    expect(serviceTarget).not.toContain("warm_gateway_streams_with_memory_auth_and_a_closed_database");
});

const pinnedBunWrapper = 'run_bun() {\n    pnpm dlx bun@1.3.14 "$@"\n}';

function hasBareBun(value) {
    return /\bbun\b/.test(value.replace(pinnedBunWrapper, ""));
}

test("service verifier pins every Bun execution through the wrapper", () => {
    expect(script.match(new RegExp(pinnedBunWrapper.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"), "g"))).toHaveLength(1);
    expect(hasBareBun(script)).toBeFalse();
    expect(script).toContain('run_bun "$PROJECT_ROOT/scripts/verify-worker-contracts.mjs"');
    expect(script.match(/run_bun -e/g)?.length).toBeGreaterThan(0);
});

test("service verifier rejects bare Bun in every supported shell position", () => {
    for (const mutation of [
        'MARKER=present bun -e "process.exit()"',
        'value="$(bun -e "console.log(1)")"',
        'bun -e "console.log(1)" | cat',
    ]) {
        expect(hasBareBun(`${script}\n${mutation}`)).toBeTrue();
    }
});

test("service verifier validates profiles before temporary state and derives their output paths", () => {
    expect(script).toContain('RUSTZEN_VERIFY_BUILD_PROFILE="${RUSTZEN_VERIFY_BUILD_PROFILE:-release}"');
    expect(script).toContain('debug) latency_output_default="$PROJECT_ROOT/target/rz/gateway-latency-debug.json" ;;');
    expect(script).toContain('release) latency_output_default="$PROJECT_ROOT/target/rz/gateway-latency.json" ;;');
    expect(script).toContain('export RUSTZEN_GATEWAY_LATENCY_OUTPUT="${RUSTZEN_GATEWAY_LATENCY_OUTPUT:-$latency_output_default}"');
    expect(script.indexOf('RUSTZEN_VERIFY_BUILD_PROFILE="${RUSTZEN_VERIFY_BUILD_PROFILE:-release}"')).toBeLessThan(script.indexOf('ROOT="$(mktemp -d'));
});

test("invalid profile exits before creating the disposable service root", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-services-profile-"));
    try {
        const result = Bun.spawnSync({
            cmd: [
                "/usr/bin/env", `TMPDIR=${root}`, "RUSTZEN_VERIFY_BUILD_PROFILE=profiling",
                new URL("./verify-services.sh", import.meta.url).pathname,
                ...Array(6).fill("/usr/bin/true"),
            ],
            stdout: "pipe", stderr: "pipe",
        });
        expect(result.exitCode).not.toBe(0);
        expect(new TextDecoder().decode(result.stderr).trim()).toBe(
            "verify-services: RUSTZEN_VERIFY_BUILD_PROFILE must be debug or release",
        );
        expect(await readdir(root)).toEqual([]);
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});


test("module-log diagnostics are sourced after cleanup is installed and before their call", () => {
    const helperPath = 'MODULE_LOG_HELPER="$PROJECT_ROOT/scripts/verify-module-log-diagnostics.sh"';
    const source = '. "$MODULE_LOG_HELPER"';
    const trap = "trap cleanup EXIT INT TERM";
    const call = "verify_module_log_diagnostics";
    expect(script).toContain(helperPath);
    expect(script).toContain('if [ ! -f "$MODULE_LOG_HELPER" ] || [ -L "$MODULE_LOG_HELPER" ]; then');
    expect(script).toContain(source);
    expect(script).not.toContain("verify_module_log_diagnostics() {");
    expect(moduleLogHelper).toContain("verify_module_log_diagnostics() {");
    expect(script.indexOf(trap)).toBeLessThan(script.indexOf(source));
    expect(script.indexOf(source)).toBeLessThan(script.lastIndexOf(call));
    expect(moduleLogHelper).toContain("const expectedLines = 24_000;");
});

test("module-log helper propagates a failed pinned Bun command", () => {
    const helper = new URL("./verify-module-log-diagnostics.sh", import.meta.url).pathname;
    const result = Bun.spawnSync({
        cmd: [
            "sh", "-ceu",
            'run_bun() { return 23; }; ROOT="$2"; RUSTZEN_ADMIN_TOKEN=x; denied_token=y; . "$1"; verify_module_log_diagnostics',
            "sh", helper, tmpdir(),
        ],
        stdout: "pipe", stderr: "pipe",
    });
    expect(result.exitCode).toBe(23);
});
