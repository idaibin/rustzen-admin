import { expect, test } from "bun:test";

import { verifyReportsFlowScenarios } from "./verify-reports-flow-scenarios.mjs";
import { SCHEDULE_WAIT_ATTEMPTS, SCHEDULE_WAIT_MS, verifyReportsScheduleScenarios, waitForScheduleDecision } from "./verify-reports-schedule-scenarios.mjs";

function response(status, data) {
    return { status, data };
}

const expectStatus = async (value, status, label) => {
    if (value.status !== status) throw new Error(`${label}: expected ${status}, got ${value.status}`);
    return value;
};
const responseData = async (value) => value.data;

test("Reports flow verifier keeps creation, cross-origin, queue, and view-only retry order", async () => {
    const calls = [];
    let flowCreates = 0;
    const { flow } = await verifyReportsFlowScenarios({
        reportsBase: "http://reports.test", adminBase: "http://admin.test", expectStatus, responseData,
        directRequest: async (_base, module, path, access, init = {}) => {
            calls.push({ module, path, access, method: init.method ?? "GET", body: init.body ? JSON.parse(init.body) : undefined });
            if (path === "/api/reports/systems") return response(200, { id: "target" });
            if (path === "/api/reports/flows") return response(flowCreates++ === 0 ? 200 : 400, { id: "flow" });
            if (path === "/api/reports/runs") return response(200, { id: "run", status: "queued" });
            if (path === "/api/reports/runs/run/retry") return response(403, null);
            throw new Error(`unexpected request ${path}`);
        },
    });
    expect(flow.id).toBe("flow");
    expect(calls).toEqual([
        { module: "reports", path: "/api/reports/systems", access: "reports:system:manage", method: "POST", body: { name: "Verification fixture", baseUrl: "http://admin.test", notes: "Contract fixture" } },
        { module: "reports", path: "/api/reports/flows", access: "reports:flow:manage", method: "POST", body: { systemId: "target", name: "Fixture template", steps: [{ action: "goto", url: "/health" }, { action: "assertText", selector: "body", text: "ok" }] } },
        { module: "reports", path: "/api/reports/flows", access: "reports:flow:manage", method: "POST", body: { systemId: "target", name: "Cross origin", steps: [{ action: "goto", url: "https://example.com" }] } },
        { module: "reports", path: "/api/reports/runs", access: "reports:run:manage", method: "POST", body: { flowId: "flow", input: {} } },
        { module: "reports", path: "/api/reports/runs/run/retry", access: "reports:run:view", method: "POST", body: undefined },
    ]);
});

test("Reports schedule verifier keeps daily/weekly, retry lineage, permissions, and SQL fixtures", async () => {
    const calls = [], sql = [], deleted = new Set();
    let dailyReads = 0, weeklyReads = 0, sourceRetries = 0;
    const sleeps = [];
    const daily = { id: "daily", cadence: "daily", enabled: false, nextDue: null };
    const weekly = { id: "weekly", cadence: "weekly", enabled: true };
    await verifyReportsScheduleScenarios({
        reportsBase: "http://reports.test", reportsRuntimeRoot: "/runtime", flow: { id: "flow" },
        now: () => new Date("2026-08-10T10:00:00.000Z"), sleep: async (ms) => sleeps.push(ms),
        expectStatus, responseData,
        spawnSync: (args) => {
            sql.push(args);
            const query = args[2];
            if (query.includes("SELECT flow_id || '|' || input_json")) {
                return { exitCode: 0, stdout: query.includes("COUNT(*)") ? "flow|{}\nsource\n0\n" : "flow|{}\n", stderr: "" };
            }
            return { exitCode: 0, stdout: "", stderr: "" };
        },
        directRequest: async (_base, module, path, access, init = {}) => {
            calls.push({ module, path, access, method: init.method ?? "GET", body: init.body ? JSON.parse(init.body) : undefined });
            const id = path.split("/").at(-1);
            if (path === "/api/reports/settings") return response(200, { timezone: "UTC" });
            if (path === "/api/reports/schedules" && (init.method ?? "GET") === "GET") return response(200, []);
            if (path === "/api/reports/schedules" && init.method === "POST") {
                return response(200, JSON.parse(init.body).cadence === "daily" ? daily : weekly);
            }
            if (path.startsWith("/api/reports/schedules/") && (init.method ?? "GET") === "GET") {
                if (deleted.has(id)) return response(404, null);
                if (id === "daily") {
                    dailyReads += 1;
                    return response(200, dailyReads < 3 ? daily : { ...daily, lastOccurrence: { decision: "enqueued", runId: "source" }, lastRun: { id: "source" } });
                }
                weeklyReads += 1;
                return response(200, weeklyReads < 2 ? weekly : { ...weekly, lastOccurrence: { decision: "skipped", reason: "missed" }, lastRun: null });
            }
            if (path.startsWith("/api/reports/schedules/") && init.method === "PUT") {
                if (access === "reports:schedule:view") return response(403, null);
                return response(200, id === "daily" ? { ...daily, enabled: true, nextDue: "next", timezone: "UTC" } : { ...weekly, enabled: false, nextDue: null });
            }
            if (path.startsWith("/api/reports/schedules/") && init.method === "DELETE") {
                deleted.add(id); return response(200, null);
            }
            if (path.startsWith("/api/reports/runs/") && path.endsWith("/retry")) {
                const run = path.split("/")[4];
                if (run === "source") return response(200, sourceRetries++ < 2 ? { id: "child", status: "queued" } : { id: "child", status: "failed" });
                if (run === "child") return response(200, { id: "grandchild", status: "queued" });
                return response(409, null);
            }
            throw new Error(`unexpected request ${path}`);
        },
    });
    expect(calls.findIndex((call) => call.path === "/api/reports/settings")).toBe(0);
    expect(calls.findIndex((call) => call.access === "reports:schedule:view" && call.method === "PUT")).toBeLessThan(
        calls.findIndex((call) => call.path === "/api/reports/schedules/daily" && call.access === "reports:schedule:manage" && call.method === "PUT"),
    );
    expect(calls.filter((call) => call.path === "/api/reports/runs/source/retry")).toHaveLength(3);
    expect(calls.filter((call) => call.path === "/api/reports/schedules" && call.method === "POST").map((call) => call.body)).toEqual([
        { flowId: "flow", cadence: "daily", weekday: null, dueTime: "10:01", input: {}, enabled: false },
        { flowId: "flow", cadence: "weekly", weekday: 0, dueTime: "09:58", input: {}, enabled: true },
    ]);
    expect(calls.filter((call) => call.method === "PUT").map((call) => call.body)).toEqual([
        { flowId: "flow", cadence: "daily", weekday: null, dueTime: "10:01", input: {}, enabled: false },
        { flowId: "flow", cadence: "daily", weekday: null, dueTime: "10:01", input: {}, enabled: true },
        { flowId: "flow", cadence: "weekly", weekday: 0, dueTime: "09:58", input: {}, enabled: false },
    ]);
    expect(sleeps).toEqual([250, 250]);
    expect(sql.map((args) => args[2]).join("\n")).toContain("UPDATE automation_schedules SET due_time=");
    expect(sql.map((args) => args[2]).join("\n")).toContain("UPDATE automation_runs SET status='failed'");
    expect(sql.every((args) => args[0] === "sqlite3" && args[1] === "/runtime/data/reports/db/reports.db")).toBeTrue();
});

test("Reports schedule wait fails after exactly 360 polls and 90 seconds", async () => {
    let reads = 0; const sleeps = [];
    await expect(waitForScheduleDecision({
        reportsBase: "http://reports.test", id: "never", decision: "enqueued", sleep: async (ms) => sleeps.push(ms),
        directRequest: async () => { reads += 1; return response(200, {}); }, expectStatus, responseData,
    })).rejects.toThrow("schedule never did not record enqueued within 90 seconds");
    expect(reads).toBe(SCHEDULE_WAIT_ATTEMPTS);
    expect(sleeps).toHaveLength(SCHEDULE_WAIT_ATTEMPTS);
    expect(sleeps.every((ms) => ms === SCHEDULE_WAIT_MS)).toBeTrue();
    expect(SCHEDULE_WAIT_ATTEMPTS * SCHEDULE_WAIT_MS).toBe(90_000);
});
