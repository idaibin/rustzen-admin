export const NODES = 100,
    DISKS = 4,
    WORKERS = 32,
    WARMUP = 128;
export type Clock = { now(): number; sleep(ms: number): Promise<void> };
export type Sample = {
    ok: boolean;
    ms: number;
    status?: number;
    code?: number;
    failure?: "timeout" | "transport" | "http" | "contract";
};
export type Fetcher = (url: string, init: RequestInit) => Promise<Response>;
export function rank(v: readonly number[], p: number) {
    if (!v.length || p <= 0 || p > 1) throw Error("invalid rank input");
    let s = [...v].sort((a, b) => a - b);
    return s[Math.ceil(s.length * p) - 1]!;
}
export function corpus(seed: string, now = new Date().toISOString()) {
    return Array.from({ length: NODES }, (_, i) => ({
        nodeId: `p8g-${seed}-${String(i).padStart(3, "0")}`,
        bootId: `80000000-0000-4000-8000-${String(i).padStart(12, "0")}`,
        sequence: 1,
        hostname: `p8g-${i}.invalid`,
        agentVersion: "p8g/1",
        collectedAt: now,
        cpuPercent: 10 + (i % 10),
        memory: { usedBytes: 1024 + i, totalBytes: 8192 },
        disks: Array.from({ length: DISKS }, (_, d) => ({
            mountPoint: `/p8g/${d}`,
            usedBytes: 100 + i + d,
            totalBytes: 10000,
        })),
    }));
}
export function response(v: unknown, expected: readonly string[]) {
    let x = v as { code?: unknown; data?: unknown };
    if (x.code !== 0 || !Array.isArray(x.data) || x.data.length !== NODES)
        throw Error("gateway corpus envelope");
    let ids = new Set<string>();
    for (let row of x.data) {
        let n = row as { nodeId?: unknown; disks?: unknown };
        if (
            typeof n.nodeId !== "string" ||
            !expected.includes(n.nodeId) ||
            ids.has(n.nodeId) ||
            !Array.isArray(n.disks) ||
            n.disks.length !== DISKS
        )
            throw Error("gateway corpus differs");
        ids.add(n.nodeId);
    }
    if (ids.size !== expected.length)
        throw Error("gateway node identity differs");
}
export async function seed(
    fetcher: Fetcher,
    base: string,
    secret: string,
    values: ReturnType<typeof corpus>,
) {
    for (let report of values) {
        let r = await fetcher(base + "/api/monitor/agent-reports", {
                method: "POST",
                headers: {
                    "content-type": "application/json",
                    "x-rustzen-monitor-agent-token": secret,
                },
                body: JSON.stringify(report),
                signal: AbortSignal.timeout(2000),
            }),
            b = (await r.json()) as {
                code?: unknown;
                data?: { status?: unknown };
            };
        if (r.status !== 200 || b.code !== 0 || b.data?.status !== "accepted")
            throw Error("sentinel seed rejected");
    }
}
export async function request(
    fetcher: Fetcher,
    url: string,
    token: string,
    ids: readonly string[],
    clock: Clock,
    timeoutMs = 2000,
): Promise<Sample> {
    let start = clock.now();
    try {
        let r = await fetcher(url, {
                headers: { authorization: `Bearer ${token}` },
                signal: AbortSignal.timeout(timeoutMs),
            }),
            v = (await r.json()) as { code?: number };
        if (r.status === 200) response(v, ids);
        return {
            ok: r.status === 200,
            status: r.status,
            code: v.code,
            ms: clock.now() - start,
            ...(r.status === 200 ? {} : { failure: "http" as const }),
        };
    } catch (error) {
        return { ok: false, ms: clock.now() - start, failure: (error as { name?: string }).name === "TimeoutError" ? "timeout" : "transport" };
    }
}
export async function lane(
    fetcher: Fetcher,
    url: string,
    token: string,
    ids: readonly string[],
    clock: Clock,
    duration = 60_000,
    minimum = 3200,
) {
    let warm = (await Promise.all(Array.from({ length: WORKERS }, async () => { let values: Sample[] = []; for (let i = 0; i < WARMUP / WORKERS; i++) values.push(await request(fetcher, url, token, ids, clock)); return values; }))).flat();
    if (warm.some((x) => !x.ok))
        throw Error(`warmup failed: ${failureSummary(warm)}`);
    let started = clock.now(), end = started + duration,
        buckets = await Promise.all(
            Array.from({ length: WORKERS }, async () => {
                let out: Sample[] = [];
                while (clock.now() < end)
                    out.push(await request(fetcher, url, token, ids, clock));
                return out;
            }),
        ),
        samples = buckets.flat();
    if (samples.length < minimum || samples.some((x) => !x.ok))
        throw Error(`load lane failed: offered=${samples.length} minimum=${minimum} ${failureSummary(samples)}`);
    let values = samples.map((x) => x.ms),
        result = {
            durationMs: clock.now() - started,
            warmupCount: warm.length,
            offered: samples.length,
            completed: samples.length,
            failures: 0,
            latenciesMs: values,
            p95Ms: rank(values, 0.95),
            p99Ms: rank(values, 0.99),
        };
    if (result.p95Ms > 500 || result.p99Ms > 1000)
        throw Error(`load latency exceeded: offered=${result.offered} p95Ms=${result.p95Ms.toFixed(3)} p99Ms=${result.p99Ms.toFixed(3)}`);
    return result;
}
function failureSummary(samples: readonly Sample[]) {
    let failures = new Map<string, number>();
    for (let sample of samples)
        if (!sample.ok) failures.set(sample.failure ?? "unknown", (failures.get(sample.failure ?? "unknown") ?? 0) + 1);
    return `failures=${JSON.stringify(Object.fromEntries([...failures].sort()))}`;
}
export function resources(
    a: Record<string, number>,
    p: { rss: number; hwm: number; pidsPeak: number; memoryPeak: number },
    b: Record<string, number>,
) {
    if (p.rss > 384 * 1024 * 1024 || p.hwm > 384 * 1024 * 1024 || p.pidsPeak > 64 || p.memoryPeak > 384 * 1024 * 1024) throw Error("resource peak");
    for (let k of [
        "memory.max",
        "memory.events.max",
        "memory.events.oom",
        "memory.events.oom_kill",
        "pids.events.max",
    ])
        if (b[k] !== a[k]) throw Error(`cgroup event changed: ${k}`);
}
export function fault(
    v: Array<{
        step: string;
        status?: number;
        code?: number;
        pid?: number;
        sha256?: string;
    }>,
    old: number,
    sha: string,
) {
    if (
        JSON.stringify(v.map((x) => x.step)) !==
            JSON.stringify([
                "stop",
                "listenerGone",
                "listenerReady",
                "registryHealthy",
            ]) ||
        v[1]?.status !== 503 ||
        v[1]?.code !== 40001 ||
        v[2]?.pid === old ||
        v[2]?.sha256 !== sha
    )
        throw Error("fault boundary differs");
}
export function recovery(completed: number) {
    if (!Number.isInteger(completed) || completed < 512)
        throw Error("recovery lane undershot");
}
