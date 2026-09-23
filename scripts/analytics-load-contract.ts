// Analytics P8g load contract: pure, testable shaping of the offered load and
// its acceptance thresholds. Mirrors the Monitor P8g contract numbers.
export const VISITORS = 100,
    SEED_EVENTS_PER_VISITOR = 3,
    WORKERS = 32,
    WARMUP = 128;
export const READ_MINIMUM = 3200,
    // The write lane offers the product's documented per-origin admission
    // ceiling: 30 requests / 300 events per 60 seconds. Fifteen 20-event
    // batches fill one window exactly; a sixteenth rides the window roll.
    TRACK_BATCH = 20,
    TRACK_INTERVAL_MS = 4_000,
    TRACK_MINIMUM = 14,
    RECOVERY_MINIMUM = 512;

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

export function trackBody(visitor: number, index: number, now = new Date().toISOString()) {
    const visitorId = `57f9c0de-0000-4000-8000-${String(visitor).padStart(12, "0")}`;
    if (index % 2 === 0)
        return {
            eventName: "page_view",
            visitorId,
            sessionId: `a7e5b1c2-0000-4000-8000-${String(visitor).padStart(12, "0")}`,
            platform: "web",
            pagePath: `/p8g/load/${visitor}/${index}`,
            occurredAt: now,
        };
    return {
        eventName: "api_request",
        visitorId,
        sessionId: `a7e5b1c2-0000-4000-8000-${String(visitor).padStart(12, "0")}`,
        platform: "web",
        apiPath: `/api/p8g/load/${visitor}/${index}`,
        apiMethod: "GET",
        statusCode: 200,
        durationMs: 5 + (index % 7),
        occurredAt: now,
    };
}

export function trackBatch(batch: number, size = TRACK_BATCH) {
    return Array.from({ length: size }, (_, index) => trackBody((batch * size + index) % VISITORS, batch * size + index));
}

export function seedBatches() {
    // Six fifty-event batches spread three events over each of the 100 visitors
    // with an integer round-robin, so the seeded uv equals VISITORS exactly.
    const batches: ReturnType<typeof trackBody>[][] = Array.from({ length: 6 }, () => []);
    for (let event = 0; event < VISITORS * SEED_EVENTS_PER_VISITOR; event += 1) {
        const visitor = Math.floor(event / SEED_EVENTS_PER_VISITOR) % VISITORS;
        batches[event % 6]!.push(trackBody(visitor, event));
    }
    return batches;
}

export function overviewEnvelope(v: unknown) {
    let x = v as { code?: unknown; data?: Record<string, unknown> };
    if (x.code !== 0 || !x.data) throw Error("overview envelope differs");
    for (const key of ["pv", "uv", "eventCount", "requestCount"])
        if (!Number.isFinite(Number(x.data[key])) || Number(x.data[key]) < 0)
            throw Error(`overview metric differs: ${key}`);
    return true;
}

export function eventsEnvelope(v: unknown) {
    let x = v as { code?: unknown; data?: { data?: unknown[]; total?: unknown } };
    if (x.code !== 0 || !x.data || !Array.isArray(x.data.data) || !Number.isFinite(Number(x.data.total)) || Number(x.data.total) < 0)
        throw Error("events envelope differs");
    return true;
}

export async function readSample(fetcher: Fetcher, url: string, token: string, clock: Clock, validate: (v: unknown) => boolean, timeoutMs = 2000): Promise<Sample> {
    const start = clock.now();
    try {
        const r = await fetcher(url, {
            headers: { authorization: `Bearer ${token}` },
            signal: AbortSignal.timeout(timeoutMs),
        });
        const v = (await r.json()) as { code?: number };
        if (r.status === 200 && !validate(v)) return { ok: false, ms: clock.now() - start, status: r.status, code: v.code, failure: "contract" };
        return { ok: r.status === 200, status: r.status, code: v.code, ms: clock.now() - start, ...(r.status === 200 ? {} : { failure: "http" as const }) };
    } catch (error) {
        return { ok: false, ms: clock.now() - start, failure: (error as { name?: string }).name === "TimeoutError" ? "timeout" : "transport" };
    }
}

export async function trackSample(fetcher: Fetcher, url: string, projectKey: string, body: unknown, clock: Clock, timeoutMs = 2000): Promise<Sample> {
    const start = clock.now();
    try {
        const r = await fetcher(url, {
            method: "POST",
            // The public tracker validates the Origin against the collection
            // policy; a non-browser load client must present it explicitly.
            headers: { "content-type": "application/json", "x-rustzen-project-key": projectKey, origin: new URL(url).origin },
            body: JSON.stringify(body),
            signal: AbortSignal.timeout(timeoutMs),
        });
        const v = (await r.json().catch(() => ({}))) as { code?: number };
        return { ok: r.status >= 200 && r.status < 300, status: r.status, code: v.code, ms: clock.now() - start, ...(r.status >= 200 && r.status < 300 ? {} : { failure: "http" as const }) };
    } catch (error) {
        return { ok: false, ms: clock.now() - start, failure: (error as { name?: string }).name === "TimeoutError" ? "timeout" : "transport" };
    }
}

export async function lane(fetcher: Fetcher, worker: () => Promise<Sample>, clock: Clock, duration = 60_000, minimum = READ_MINIMUM) {
    const warm = (await Promise.all(Array.from({ length: WORKERS }, async () => { const values: Sample[] = []; for (let i = 0; i < WARMUP / WORKERS; i++) values.push(await worker()); return values; }))).flat();
    if (warm.some((x) => !x.ok)) throw Error(`warmup failed: ${failureSummary(warm)}`);
    const started = clock.now(), end = started + duration;
    const buckets = await Promise.all(Array.from({ length: WORKERS }, async () => { const out: Sample[] = []; while (clock.now() < end) out.push(await worker()); return out; }));
    const samples = buckets.flat();
    if (samples.length < minimum || samples.some((x) => !x.ok)) throw Error(`load lane failed: offered=${samples.length} minimum=${minimum} ${failureSummary(samples)}`);
    const values = samples.map((x) => x.ms);
    const result = { durationMs: clock.now() - started, warmupCount: warm.length, offered: samples.length, completed: samples.length, failures: 0, latenciesMs: values, p95Ms: rank(values, 0.95), p99Ms: rank(values, 0.99) };
    if (result.p95Ms > 500 || result.p99Ms > 1000) throw Error(`load latency exceeded: offered=${result.offered} p95Ms=${result.p95Ms.toFixed(3)} p99Ms=${result.p99Ms.toFixed(3)}`);
    return result;
}

export async function pacedTrackLane(fetcher: Fetcher, url: string, projectKey: string, clock: Clock, duration = 60_000, minimum = TRACK_MINIMUM) {
    const started = clock.now(), end = started + duration;
    const samples: Sample[] = [];
    for (let batch = 0; clock.now() < end && batch < 30; batch += 1) {
        if (batch > 0) await clock.sleep(TRACK_INTERVAL_MS);
        samples.push(await trackSample(fetcher, url, projectKey, trackBatch(batch), clock));
    }
    if (samples.length < minimum || samples.some((x) => !x.ok)) throw Error(`track lane failed: offered=${samples.length} minimum=${minimum} ${failureSummary(samples)}`);
    const values = samples.map((x) => x.ms);
    return { durationMs: clock.now() - started, warmupCount: 0, offered: samples.length, completed: samples.length, failures: 0, latenciesMs: values, p95Ms: rank(values, 0.95), p99Ms: rank(values, 0.99) };
}

export function failureSummary(samples: readonly Sample[]) {
    const failures = new Map<string, number>();
    for (const sample of samples) if (!sample.ok) failures.set(sample.failure ?? "unknown", (failures.get(sample.failure ?? "unknown") ?? 0) + 1);
    return `failures=${JSON.stringify(Object.fromEntries([...failures].sort()))}`;
}

export type FaultStep = { step: string; status?: number; pid?: number; sha256?: string };

export function analyticsFault(v: FaultStep[], old: number, sha: string) {
    if (JSON.stringify(v.map((x) => x.step)) !== JSON.stringify(["stop", "apiFailed", "listenerReady", "registryHealthy"])) throw Error("fault boundary steps differ");
    const failed = v[1]!;
    if (typeof failed.status !== "number" || failed.status < 500) throw Error("outage read did not fail with a server error");
    const restarted = v[2]!;
    if (typeof restarted.pid !== "number" || restarted.pid === old || restarted.sha256 !== sha) throw Error("insights restart owner differs");
}

export function recoveryLane(completed: number) {
    if (!Number.isInteger(completed) || completed < RECOVERY_MINIMUM) throw Error("recovery lane undershot");
}
