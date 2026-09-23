import { expect, test } from "bun:test";
import { mkdtemp, readFile, readdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

const helper = new URL("./verify-service-auth-module-gateway.sh", import.meta.url).pathname;
const repoRoot = new URL("../", import.meta.url).pathname;

async function fixture(body, { expectExit = 0 } = {}) {
    const before = (await readdir(repoRoot)).sort();
    const root = await mkdtemp(join(tmpdir(), "rz-auth-gateway-"));
    try {
        const callsFile = join(root, "calls");
        const result = Bun.spawnSync({
            cmd: ["sh", "-ceu", body, "sh", helper, callsFile],
            stdout: "pipe", stderr: "pipe", env: { ...process.env, BUN_BIN: process.execPath },
        });
        expect(result.exitCode).toBe(expectExit);
        return {
            stdout: new TextDecoder().decode(result.stdout),
            stderr: new TextDecoder().decode(result.stderr),
            log: await readFile(callsFile, "utf8").catch(() => ""),
        };
    } finally {
        await rm(root, { recursive: true, force: true });
        expect((await readdir(repoRoot)).sort()).toEqual(before);
    }
}

const header = 'authorization: Bearer token';

test("login forwards its exact JSON request and response", async () => {
    const result = await fixture(`
        . "$1"
        AUTH_TEST_CALLS_FILE="$2"
        curl() { printf '%s\\n' "$@" >"$AUTH_TEST_CALLS_FILE"; printf '%s' '{"data":{"token":"reply"}}'; }
        RUSTZEN_ADMIN_PORT=19001
        login owner password
    `);
    expect(result.stdout).toBe('{"data":{"token":"reply"}}');
    expect(result.log).toContain("--fail\n--silent\n--show-error");
    expect(result.log).toContain("content-type: application/json");
    expect(result.log).toContain('{"username":"owner","password":"password"}');
    expect(result.log).toContain("http://127.0.0.1:19001/api/auth/login");
});

test("module state uses the bearer matcher and converges without sleep", async () => {
    const result = await fixture(`
        . "$1"
        AUTH_TEST_CALLS_FILE="$2"
        curl() { printf 'curl:%s\\n' "$*" >>"$AUTH_TEST_CALLS_FILE"; printf '%s' '{"data":[{"id":"monitor","available":true,"compatible":false}]}'; }
        run_bun() { printf 'match:%s:%s:%s\\n' "$MODULE_ID" "$EXPECT_AVAILABLE" "$EXPECT_COMPATIBLE" >>"$AUTH_TEST_CALLS_FILE"; "$BUN_BIN" "$@"; }
        sleep() { printf 'sleep:%s\\n' "$1" >>"$AUTH_TEST_CALLS_FILE"; }
        dump_logs() { printf dump >>"$AUTH_TEST_CALLS_FILE"; }
        RUSTZEN_ADMIN_PORT=19002 RUSTZEN_ADMIN_TOKEN=token
        wait_for_module_state monitor true false
    `);
    expect(result.log).toContain(`-H ${header}`);
    expect(result.log).toContain("match:monitor:true:false");
    expect(result.log).not.toContain("sleep:");
});


test("module matcher rejects wrong id, availability, compatibility, and malformed payloads before a second poll", async () => {
    for (const payload of [
        '{"data":[{"id":"insights","available":true,"compatible":false}]}',
        '{"data":[{"id":"monitor","available":false,"compatible":false}]}',
        '{"data":[{"id":"monitor","available":true,"compatible":true}]}',
        'not-json',
    ]) {
        const result = await fixture(`
            . "$1"
            AUTH_TEST_CALLS_FILE="$2"
            curl() { printf '%s' '${payload}'; }
            run_bun() { "$BUN_BIN" "$@"; }
            sleep() { printf sleep >>"$AUTH_TEST_CALLS_FILE"; exit 77; }
            dump_logs() { printf dump >>"$AUTH_TEST_CALLS_FILE"; }
            RUSTZEN_ADMIN_PORT=19008 RUSTZEN_ADMIN_TOKEN=token
            wait_for_module_state monitor true false
        `, { expectExit: 77 });
        expect(result.log).toBe("sleep");
    }
});

test("module state times out after 180 polls, sleeps, dumps, and exits", async () => {
    const result = await fixture(`
        . "$1"
        AUTH_TEST_CALLS_FILE="$2"
        curl() { printf curl >>"$AUTH_TEST_CALLS_FILE"; printf '%s' '{"data":[]}'; }
        run_bun() { return 1; }
        sleep() { printf 's' >>"$AUTH_TEST_CALLS_FILE"; }
        dump_logs() { printf d >>"$AUTH_TEST_CALLS_FILE"; }
        RUSTZEN_ADMIN_PORT=19003 RUSTZEN_ADMIN_TOKEN=token
        wait_for_module_state monitor true true
    `, { expectExit: 1 });
    expect((result.log.match(/curl/g) ?? []).length).toBe(180);
    expect((result.log.match(/s/g) ?? []).length).toBe(180);
    expect(result.log.endsWith("d")).toBeTrue();
    expect(result.stderr.trim()).toBe("verify-services: module state did not converge: monitor available=true compatible=true");
});

test("gateway URLs, unavailable envelope, and healthy exclusion use the expected token", async () => {
    const urls = await fixture(`. "$1"; RUSTZEN_ADMIN_PORT=19004; module_gateway_url monitor; module_gateway_url insights; module_gateway_url reports`);
    expect(urls.stdout.trim().split("\n")).toEqual([
        "http://127.0.0.1:19004/api/monitor/nodes",
        "http://127.0.0.1:19004/api/insights/overview",
        "http://127.0.0.1:19004/api/reports/systems",
    ]);
    const unavailable = await fixture(`
        . "$1"
        AUTH_TEST_CALLS_FILE="$2"
        wait_for_status() { printf 'wait:%s:%s:%s\\n' "$@" >"$AUTH_TEST_CALLS_FILE"; }
        curl() { printf '%s' '{"code":40001,"message":"monitor worker is temporarily unavailable.","data":null}'; }
        run_bun() { pnpm dlx bun@1.3.14 "$@"; }
        RUSTZEN_ADMIN_PORT=19005 RUSTZEN_ADMIN_TOKEN=token
        assert_gateway_unavailable monitor
    `);
    expect(unavailable.log).toBe("wait:503:http://127.0.0.1:19005/api/monitor/nodes:token\n");
    const healthy = await fixture(`
        . "$1"
        AUTH_TEST_CALLS_FILE="$2"
        wait_for_status() { printf '%s:%s:%s\\n' "$@" >>"$AUTH_TEST_CALLS_FILE"; }
        RUSTZEN_ADMIN_PORT=19006 RUSTZEN_ADMIN_TOKEN=token
        assert_module_gateways_healthy_except insights
    `);
    expect(healthy.log).toBe("200:http://127.0.0.1:19006/api/monitor/nodes:token\n200:http://127.0.0.1:19006/api/reports/systems:token\n");
});

test("unavailable envelope rejects wrong code, message, and non-null data", async () => {
    for (const payload of [
        '{"code":40002,"message":"monitor worker is temporarily unavailable.","data":null}',
        '{"code":40001,"message":"wrong","data":null}',
        '{"code":40001,"message":"monitor worker is temporarily unavailable.","data":{}}',
    ]) {
        await fixture(`
            . "$1"
        AUTH_TEST_CALLS_FILE="$2"
            wait_for_status() { :; }
            curl() { printf '%s' '${payload}'; }
            run_bun() { pnpm dlx bun@1.3.14 "$@"; }
            RUSTZEN_ADMIN_PORT=19007 RUSTZEN_ADMIN_TOKEN=token
            assert_gateway_unavailable monitor
        `, { expectExit: 1 });
    }
});
