import { expect, test } from "bun:test";
import { chmod, mkdir, mkdtemp, readdir, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const script = await Bun.file(new URL("./verify-services.sh", import.meta.url)).text();
const justfile = await Bun.file(new URL("../justfile", import.meta.url)).text();
const moduleLogHelper = await Bun.file(new URL("./verify-module-log-diagnostics.sh", import.meta.url)).text();
const databaseIsolationHelper = await Bun.file(new URL("./verify-database-isolation.sh", import.meta.url)).text();
const lifecycleHelper = await Bun.file(new URL("./verify-service-lifecycle.sh", import.meta.url)).text();
const authModuleGatewayHelper = await Bun.file(new URL("./verify-service-auth-module-gateway.sh", import.meta.url)).text();

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
    expect(lifecycleHelper).toContain('monitor) "$MONITOR" controller >"$log" 2>&1 & ;;');
    expect(lifecycleHelper).not.toMatch(/monitor\).*\b(?:init-db|bind-database|validate-database)\b/);

});

const latencyContractCommand = "pnpm dlx bun@1.3.14 test scripts/gateway-latency-contract.test.mjs scripts/verify-insights-scenarios.test.mjs scripts/verify-reports-scenarios.test.mjs scripts/verify-worker-contracts.test.mjs scripts/verify-service-auth-module-gateway.test.mjs scripts/verify-services.test.mjs";

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


test("database isolation is sourced before its post-termination restore phase", () => {
    const source = '. "$DATABASE_ISOLATION_HELPER"';
    expect(script).toContain('DATABASE_ISOLATION_HELPER="$PROJECT_ROOT/scripts/verify-database-isolation.sh"');
    expect(script).toContain('if [ ! -f "$DATABASE_ISOLATION_HELPER" ] || [ -L "$DATABASE_ISOLATION_HELPER" ]; then');
    expect(script).toContain(source);
    expect(script).not.toContain("verify_database_isolation() {");
    expect(databaseIsolationHelper).toContain("expect_corrupt_start_failure() {");
    expect(databaseIsolationHelper).toContain('reports) path="$ROOT/data/reports/db/reports.db" ;;');
    expect(databaseIsolationHelper).toContain('*) path="$ROOT/data/db/$database.db" ;;');
    const adminRestore = databaseIsolationHelper.slice(
        databaseIsolationHelper.indexOf('if [ "$db_service" = admin ]; then'),
        databaseIsolationHelper.indexOf('\n    else', databaseIsolationHelper.indexOf('if [ "$db_service" = admin ]; then')),
    );
    expect(adminRestore).toContain('wait_for_module_state monitor true true');
    expect(adminRestore).toContain('wait_for_module_state insights true true');
    expect(adminRestore).toContain('wait_for_module_state reports true true');
    expect(databaseIsolationHelper).toContain('for database in admin monitor insights; do');
    expect(databaseIsolationHelper).toContain('[ -s "$ROOT/data/db/$database.db" ] || {');
    expect(databaseIsolationHelper.match(/verify_database_isolation (?:monitor monitor|insights insights|reports reports|admin admin)/g)).toEqual([
        "verify_database_isolation monitor monitor",
        "verify_database_isolation insights insights",
        "verify_database_isolation reports reports",
        "verify_database_isolation admin admin",
    ]);
    expect(script.indexOf("trap cleanup EXIT INT TERM")).toBeLessThan(script.indexOf(source));
    expect(script.indexOf('PHASE="termination-$service"')).toBeLessThan(script.lastIndexOf("verify_database_isolations"));
});

test("database-isolation helper propagates an outer lifecycle failure", () => {
    const helper = new URL("./verify-database-isolation.sh", import.meta.url).pathname;
    const result = Bun.spawnSync({
        cmd: [
            "sh", "-ceu",
            'stop_service() { return 29; }; ROOT="$2"; . "$1"; verify_database_isolation monitor monitor',
            "sh", helper, tmpdir(),
        ],
        stdout: "pipe", stderr: "pipe",
    });
    expect(result.exitCode).toBe(29);
});


test("lifecycle helper is guarded before the cleanup trap and retains process contracts", () => {
    const source = '. "$LIFECYCLE_HELPER"';
    expect(script).toContain('LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/verify-service-lifecycle.sh"');
    expect(script).toContain('if [ ! -f "$LIFECYCLE_HELPER" ] || [ -L "$LIFECYCLE_HELPER" ]; then');
    expect(script).toContain('if ! . "$LIFECYCLE_HELPER"; then');
    expect(script).not.toContain("start_service() {");
    expect(script.indexOf(source)).toBeLessThan(script.indexOf("trap cleanup EXIT INT TERM"));
    for (const command of [
        'admin) "$ADMIN" serve >"$log" 2>&1 & ;;',
        'monitor) "$MONITOR" controller >"$log" 2>&1 & ;;',
        'insights) "$INSIGHTS" serve >"$log" 2>&1 & ;;',
        'reports) "$REPORTS" serve >"$log" 2>&1 & ;;',
        'monitor_agent) "$AGENT" >"$log" 2>&1 & ;;',
    ]) expect(lifecycleHelper).toContain(command);
    expect(lifecycleHelper).toContain('for name in monitor_agent admin reports insights monitor; do');
    expect(lifecycleHelper).toContain('[ "$count" -lt 50 ]');
    expect(lifecycleHelper).toContain('kill -KILL "$pid" 2>/dev/null || true');
    expect(lifecycleHelper).toContain('[ "$count" -lt 180 ]');
});

test("lifecycle symlink guard removes the disposable root before sourcing", async () => {
    const project = await mkdtemp(join(tmpdir(), "rz-services-lifecycle-"));
    try {
        const scripts = join(project, "scripts");
        const temporary = join(project, "tmp");
        await mkdir(scripts);
        await mkdir(temporary);
        const runner = join(scripts, "verify-services.sh");
        const marker = join(scripts, "marker.sh");
        await writeFile(marker, '#!/usr/bin/env sh\ntouch "$MARKER"\n');
        await chmod(marker, 0o755);
        await symlink(marker, join(scripts, "link.sh"));
        await writeFile(runner, script.replace(
            'LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/verify-service-lifecycle.sh"',
            'LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/link.sh"',
        ));
        const result = Bun.spawnSync({
            cmd: ["sh", runner, ...Array(6).fill("/usr/bin/true")],
            env: { ...process.env, TMPDIR: temporary, MARKER: join(project, "marker-ran") },
            stdout: "pipe", stderr: "pipe",
        });
        expect(result.exitCode).not.toBe(0);
        expect(await readdir(temporary)).toEqual([]);
        expect(await Bun.file(join(project, "marker-ran")).exists()).toBeFalse();
    } finally {
        await rm(project, { recursive: true, force: true });
    }
});

test("malformed regular lifecycle helper removes the disposable root", async () => {
    const project = await mkdtemp(join(tmpdir(), "rz-services-malformed-"));
    try {
        const scripts = join(project, "scripts");
        const temporary = join(project, "tmp");
        await mkdir(scripts);
        await mkdir(temporary);
        const runner = join(scripts, "verify-services.sh");
        await writeFile(join(scripts, "malformed.sh"), "(\n");
        await writeFile(runner, script.replace(
            'LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/verify-service-lifecycle.sh"',
            'LIFECYCLE_HELPER="$PROJECT_ROOT/scripts/malformed.sh"',
        ));
        const result = Bun.spawnSync({
            cmd: ["sh", runner, ...Array(6).fill("/usr/bin/true")],
            env: { ...process.env, TMPDIR: temporary },
            stdout: "pipe", stderr: "pipe",
        });
        expect(result.exitCode).not.toBe(0);
        expect(await readdir(temporary)).toEqual([]);
    } finally {
        await rm(project, { recursive: true, force: true });
    }
});

test("lifecycle helper preserves stop order, forced kill, and source failure propagation", async () => {
    const helper = new URL("./verify-service-lifecycle.sh", import.meta.url).pathname;
    const order = Bun.spawnSync({
        cmd: ["sh", "-ceu", '. "$1"; stop_service() { printf "%s\n" "$1"; }; stop_all', "sh", helper],
        stdout: "pipe", stderr: "pipe",
    });
    expect(order.exitCode).toBe(0);
    expect(new TextDecoder().decode(order.stdout).trim().split("\n")).toEqual([
        "monitor_agent", "admin", "reports", "insights", "monitor",
    ]);
    const root = await mkdtemp(join(tmpdir(), "rz-services-kill-"));
    try {
        const timeout = Bun.spawnSync({
            cmd: [
                "sh", "-ceu",
                'ROOT="$2"; mkdir -p "$ROOT/pids"; printf 42 >"$ROOT/pids/admin"; : >"$ROOT/pids/admin.log"; kill() { printf "%s %s\n" "$1" "${2:-}"; return 0; }; sleep() { :; }; wait() { :; }; . "$1"; stop_service admin',
                "sh", helper, root,
            ], stdout: "pipe", stderr: "pipe",
        });
        expect(timeout.exitCode).toBe(0);
        const calls = new TextDecoder().decode(timeout.stdout);
        expect(calls).toContain("-TERM 42");
        expect(calls).toContain("-KILL 42");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
    const failure = Bun.spawnSync({
        cmd: ["sh", "-ceu", '. "$1"; service_health_url unknown', "sh", helper],
        stdout: "pipe", stderr: "pipe",
    });
    expect(failure.exitCode).toBe(1);
});


test("auth/module-gateway helper preserves module waits, URLs, envelopes, and failure propagation", () => {
    const helper = new URL("./verify-service-auth-module-gateway.sh", import.meta.url).pathname;
    expect(script).toContain('AUTH_MODULE_GATEWAY_HELPER="$PROJECT_ROOT/scripts/verify-service-auth-module-gateway.sh"');
    expect(script).toContain('if [ ! -f "$AUTH_MODULE_GATEWAY_HELPER" ] || [ -L "$AUTH_MODULE_GATEWAY_HELPER" ]; then');
    expect(script).toContain('if ! . "$AUTH_MODULE_GATEWAY_HELPER"; then');
    expect(script.indexOf('. "$AUTH_MODULE_GATEWAY_HELPER"')).toBeGreaterThan(script.indexOf("trap cleanup EXIT INT TERM"));
    expect(script).not.toContain("wait_for_module_state() {");
    expect(authModuleGatewayHelper).toContain('[ "$count" -lt 180 ]');
    expect(authModuleGatewayHelper).toContain("/api/monitor/nodes");
    expect(authModuleGatewayHelper).toContain("/api/insights/overview");
    expect(authModuleGatewayHelper).toContain("/api/reports/systems");
    expect(authModuleGatewayHelper).toContain('payload.code !== 40001 || payload.message !== expected || payload.data !== null');
    const failure = Bun.spawnSync({
        cmd: ["sh", "-ceu", '. "$1"; module_gateway_url invalid', "sh", helper],
        stdout: "pipe", stderr: "pipe",
    });
    expect(failure.exitCode).toBe(1);
});
