import { expect, test } from "bun:test";
import * as c from "./monitor-load-contract.ts";
import { controlledBoundary } from "./monitor-load-fault.ts";
import { faultPhase, maxima, phase, quiet, stablePhase } from "./monitor-load-sampler.ts";
import { validatePhase } from "./monitor-load-receipt-schema.ts";
test("P8g preflight recipe passes exactly one prepared context", async () => {
    const result = Bun.spawnSync(["just", "--dry-run", "preflight-monitor-load-runtime", "/tmp/context.json"], { cwd: new URL("..", import.meta.url).pathname, stdout: "pipe", stderr: "pipe" });
    expect(result.exitCode).toBe(0);
    expect(new TextDecoder().decode(result.stderr).trim()).toBe("pnpm dlx bun@1.3.14 scripts/verify-monitor-load-runtime-preflight.ts \"/tmp/context.json\"");
});
const reports = c.corpus("sentinel", "2026-01-01T00:00:00.000Z"),
    rows = reports.map((x) => ({ nodeId: x.nodeId, disks: x.disks })),
    ids = reports.map((x) => x.nodeId);
test("P8g fixed corpus and negative response contracts", () => {
    expect(reports).toHaveLength(100);
    expect(reports.every((x) => x.disks.length === 4)).toBeTrue();
    c.response({ code: 0, data: rows }, ids);
    expect(() => c.response({ code: 0, data: rows.slice(1) }, ids)).toThrow();
    expect(() =>
        c.response({ code: 0, data: [...rows, { ...rows[0] }] }, ids),
    ).toThrow();
    expect(c.rank([1, 2, 3, 4], 0.95)).toBe(4);
});
test("P8g phase snapshots are exactly paired and exclude probes from work time", async () => {
    let at = 0, reads = 0, owner = { pid: 1, dev: "1", ino: "1", sha256: "a".repeat(64) };
    const reading = { rss: 10, hwm: 20, pidsCurrent: 2, pidsPeak: 3, memoryCurrent: 30, memoryPeak: 40, events: {} };
    const result = await phase("lane-1", { now: () => at }, async () => { reads++; at += 10; return reading; }, async () => ({ admin: owner, monitor: owner }), async () => { at += 59000; return true; });
    expect(reads).toBe(2); expect(result.workDurationMs).toBe(59000); expect(result.snapshots).toHaveLength(2);
    expect(maxima(result.snapshots)).toMatchObject({ pidsPeak: 3 }); stablePhase(result);
    expect(quiet(result.snapshots[1])).toMatchObject({ pidsCurrent: 2 });
    expect(() => faultPhase(result)).toThrow();
});
test("P8g receipt phase rejects missing, third, reversed, short, and identity-masked snapshots", () => {
    const hash = "a".repeat(64), owner = { pid: 2, dev: "1", ino: "1", sha256: hash }, reading = at => ({ at, rss: 1, hwm: 1, pidsCurrent: 1, pidsPeak: 1, memoryCurrent: 1, memoryPeak: 1, events: { "memory.max": 1, "memory.events.max": 0, "memory.events.oom": 0, "memory.events.oom_kill": 0, "pids.events.max": 0 } });
    const value = { name: "lane-1", workDurationMs: 5, snapshots: [reading(0), reading(5)], services: { before: { admin: owner, monitor: owner }, after: { admin: owner, monitor: owner } } };
    expect(validatePhase(value, 0)).toBe(value);
    expect(() => validatePhase({ ...value, snapshots: [reading(0)] }, 0)).toThrow();
    expect(() => validatePhase({ ...value, snapshots: [reading(0), reading(5), reading(6)] }, 0)).toThrow();
    expect(() => validatePhase({ ...value, snapshots: [reading(5), reading(0)] }, 0)).toThrow();
    expect(() => validatePhase({ ...value, workDurationMs: 6 }, 0)).toThrow();
    expect(() => validatePhase({ ...value, snapshots: [reading(0), { ...reading(5), pidsCurrent: 65, pidsPeak: 1 }] }, 0)).toThrow();
    expect(() => validatePhase({ ...value, services: { before: { admin: owner, monitor: owner }, after: { admin: owner, monitor: { ...owner, pid: 3 } } } }, 0)).toThrow();
    const fault = { ...value, name: "fault", services: { before: { admin: owner, monitor: owner }, after: { admin: owner, monitor: { ...owner, pid: 3 } } } };
    expect(validatePhase(fault, 6)).toBe(fault);
    expect(() => validatePhase({ ...fault, services: { before: fault.services.before, after: { admin: { ...owner, pid: 4 }, monitor: fault.services.after.monitor } } }, 6)).toThrow();
});
test("P8g resource and controlled fault contracts reject drift", () => {
    let events = {
        "memory.max": 0,
        "memory.events.max": 0,
        "memory.events.oom": 0,
        "memory.events.oom_kill": 0,
        "pids.events.max": 0,
    };
    c.resources(events, { rss: 1, hwm: 1, pids: 1 }, events);
    expect(() =>
        c.resources(
            events,
            { rss: 1, pids: 1 },
            { ...events, "memory.events.oom": 1 },
        ),
    ).toThrow();
    c.fault(
        [
            { step: "stop" },
            { step: "listenerGone", status: 503, code: 40001 },
            { step: "listenerReady", pid: 2, sha256: "a" },
            { step: "registryHealthy" },
        ],
        1,
        "a",
    );
    expect(() =>
        c.fault(
            [
                { step: "stop" },
                { step: "listenerGone", status: 200, code: 0 },
                { step: "listenerReady", pid: 1, sha256: "a" },
                { step: "registryHealthy" },
            ],
            1,
            "a",
        ),
    ).toThrow();
});
test("controlled inflight records completion at resolve and rejects a pre-stop completion", async () => {
    let now = 0, releases = [], started = 0;
    const good = await controlledBoundary(() => new Promise(resolve => { started++; releases.push(resolve); }), () => ++now, async () => true, async () => { expect(started).toBe(4); let confirmed = ++now; await Bun.sleep(1); now++; for (const resolve of releases) resolve({ status: 503, code: 40001 }); return confirmed; });
    expect(good.rows).toHaveLength(4);
    expect(good.rows.every(row => row.start < good.stopAt && good.stopAt < row.end && row.at === row.end)).toBeTrue();
    let early = [];
    await expect(controlledBoundary(() => new Promise(resolve => early.push(resolve)), () => ++now, async () => true, async () => { for (const resolve of early) resolve({ status: 503, code: 40001 }); await Promise.resolve(); return ++now; })).rejects.toThrow("cross");
    await expect(controlledBoundary(async () => ({ status: 503, code: 40001 }), () => ++now, async () => true, async () => ++now, 1)).rejects.toThrow("cross");
});
test("P8g injection seam measures complete response without retry", async () => {
    let calls = 0,
        clock = {
            value: 0,
            now() {
                return this.value++;
            },
            sleep: async () => {},
        };
    let fetcher = async () => {
        calls++;
        return new Response(JSON.stringify({ code: 0, data: rows }), {
            status: 200,
        });
    };
    let sample = await c.request(fetcher, "http://x", "t", ids, clock);
    expect(sample.ok).toBeTrue();
    expect(calls).toBe(1);
    expect(sample.ms).toBe(1);
});
test("P8g injected lane enforces offered volume and latency", async () => {
    let clock = {
            value: 0,
            now() {
                return this.value++;
            },
            sleep: async () => {},
        },
        fetcher = async () =>
            new Response(JSON.stringify({ code: 0, data: rows }), {
                status: 200,
            });
    let result = await c.lane(fetcher, "http://x", "t", ids, clock, 10_000);
    expect(result.completed).toBeGreaterThanOrEqual(3200);
    expect(result.p99Ms).toBeLessThanOrEqual(64);
    let slow = {
        value: 0,
        now() {
            this.value += 1001;
            return this.value;
        },
        sleep: async () => {},
    };
    await expect(
        c.lane(fetcher, "http://x", "t", ids, slow, 10_000),
    ).rejects.toThrow();
    let recovery = await c.lane(fetcher, "http://x", "t", ids, clock, 10_000, 512);
    expect(recovery.completed).toBeGreaterThanOrEqual(512);
});
