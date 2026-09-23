// Analytics P8g load certification driver. Runs against one retained P8e
// deployment of the signed release; writes one canonical load receipt.
import { canonicalJson, sha256 } from "../distribution/release-manifest-core.ts";
import { parseAnalyticsLoadReceipt } from "./analytics-load-receipt-schema.ts";
import { seedBatches, trackBatch, overviewEnvelope, eventsEnvelope, readSample, trackSample, lane, pacedTrackLane, analyticsFault, recoveryLane, type Clock } from "./analytics-load-contract.ts";
import { docker, listener, listenerGone, restartedOwner, service, certifiedOwner, type Docker, type Owner } from "./monitor-load-runtime.ts";
import { phase, maxima, compareQuiet, type Reading } from "./monitor-load-sampler.ts";
import { mkdir, readFile, writeFile } from "node:fs/promises";

const args = new Map<string, string>();
for (let index = 2; index < Bun.argv.length; index += 2) args.set(Bun.argv[index]!, Bun.argv[index + 1]!);
const required = ["--context", "--output"];
if (required.some((flag) => !args.get(flag)) || args.size !== required.length) throw Error("usage: --context FILE --output NEW_DIRECTORY");
const output = args.get("--output")!;
await mkdir(output, { recursive: false });
const context = JSON.parse(await readFile(args.get("--context")!, "utf8")) as {
    adminUrl: string; containerName: string; releaseResult: string; certificate: string; publicKey: string;
    exportRoot: string; passwordFile: string; projectKeyFile: string; nativeEvidence: string; nativeEvidenceSha256: string;
    browserReceipt: string; browserReceiptSha256: string; runtimeEvidence: string; runtimeEvidenceSha256: string;
};
const base = context.adminUrl;
const clock: Clock = { now: () => Date.now(), sleep: (ms) => new Promise((resolve) => setTimeout(resolve, ms)) };
const fetcher: typeof fetch = (input, init) => fetch(input, init);
const run: Docker = docker;

async function usageAnalytics(): Promise<Omit<Reading, "at">> {
    const script = `for f in memory.max memory.current memory.peak pids.max pids.current pids.peak;do printf '%s=' "$f";cat /sys/fs/cgroup/$f;done;for e in max oom oom_kill;do printf 'memory.events.%s=' "$e";awk -v e=$e '$1==e{print $2}' /sys/fs/cgroup/memory.events;done;printf 'pids.events.max=';awk -v e=max '$1==e{print $2}' /sys/fs/cgroup/pids.events;rss=0;hwm=0;for p in $(systemctl show -p MainPID --value rz-admin.service rz-insights.service);do test "$p" -gt 1 && test -r /proc/$p/status || continue;set -- $(awk '/VmRSS/{r=$2}/VmHWM/{h=$2} END{print r+0,h+0}' /proc/$p/status);rss=$((rss+$1));hwm=$((hwm+$2));done;printf 'rss=%s\\nhwm=%s\\n' "$((rss*1024))" "$((hwm*1024))"`;
    const raw = await run(["exec", context.containerName, "sh", "-ceu", script], 1800);
    const v = Object.fromEntries(raw.split("\n").filter(Boolean).map((x) => x.split("=", 2)));
    const n = (k: string) => { const x = Number(v[k]); if (!Number.isFinite(x)) throw Error(`invalid resource sample: ${k}`); return x; };
    if (n("memory.max") !== 512 * 1024 * 1024 || n("pids.max") !== 256 || n("pids.current") > 64 || n("pids.peak") > 64 || n("memory.peak") > 384 * 1024 * 1024) throw Error("container limits differ");
    return { rss: n("rss"), hwm: n("hwm"), pidsCurrent: n("pids.current"), pidsPeak: n("pids.peak"), memoryCurrent: n("memory.current"), memoryPeak: n("memory.peak"), events: { "memory.max": n("memory.max"), "memory.events.max": n("memory.events.max"), "memory.events.oom": n("memory.events.oom"), "memory.events.oom_kill": n("memory.events.oom_kill"), "pids.events.max": n("pids.events.max") } };
}

type Services = { admin: Owner; insights: Owner };
const currentServices = async (): Promise<Services> => ({
    admin: await service(run, context.containerName, "rz-admin.service", "/opt/rz/current/bin/rz-admin"),
    insights: await service(run, context.containerName, "rz-insights.service", "/opt/rz/current/bin/rz-insights"),
});
const stable = (p: { services: { before: Services; after: Services } }) => {
    for (const name of ["admin", "insights"] as const)
        if (p.services.before[name].pid !== p.services.after[name].pid || p.services.before[name].sha256 !== p.services.after[name].sha256)
            throw Error(`phase service identity differs: ${name}`);
};

// --- admission ---------------------------------------------------------------
const release = JSON.parse(await readFile(context.releaseResult, "utf8"));
const native = JSON.parse(await readFile(context.nativeEvidence, "utf8"));
const binaries = new Map<string, string>((native.release.binaryDigests as Array<{ path: string; sha256: string }>).map((entry) => [entry.path, entry.sha256]));
const health = await (await fetch(`${base}/health`)).json();
if (health?.status !== "ok" || health?.selectedBinding?.buildId !== native.selection.buildId || health?.selectedBinding?.compositionId !== native.selection.compositionId) throw Error("health binding differs from native evidence");
const before = await currentServices();
certifiedOwner(before.admin, binaries.get("bin/rz-admin"));
certifiedOwner(before.insights, binaries.get("bin/rz-insights"));
const adminListener = await listener(run, context.containerName, 19801, before.admin.sha256);
if (adminListener.pid !== before.admin.pid) throw Error("admin listener owner differs");

// --- session and collection policy -------------------------------------------
const password = (await readFile(context.passwordFile, "utf8")).trim();
const projectKey = (await readFile(context.projectKeyFile, "utf8")).trim();
const login = await (await fetch(`${base}/api/auth/login`, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ username: "owner", password }) })).json();
const token = login?.data?.token;
if (typeof token !== "string" || !token) throw Error("owner login failed");
const policy = await (await fetch(`${base}/api/insights/collection-policy`, { method: "PUT", headers: { "content-type": "application/json", authorization: `Bearer ${token}` }, body: JSON.stringify({ collectionEnabled: true, projectKey, allowedOrigins: [new URL(base).origin] }) })).json();
if (policy?.code !== 0 || policy?.data?.collectionEnabled !== true || policy?.data?.projectConfigured !== true) throw Error("collection policy enablement differs");

// --- seed (five admission-ceiling batches, one per rolling window step) ------
const corpus = seedBatches();
for (const batch of corpus) {
    const sample = await trackSample(fetcher, `${base}/api/insights/track`, projectKey, batch, clock);
    if (!sample.ok) throw Error(`seed track rejected: ${JSON.stringify(sample)}`);
}
if (corpus.flat().length !== 300) throw Error("seed volume differs");
const seededRead = await readSample(fetcher, `${base}/api/insights/overview`, token, clock, overviewEnvelope);
if (!seededRead.ok) throw Error("seeded overview read failed");

// --- lanes, drain, quiet -----------------------------------------------------
const overviewUrl = `${base}/api/insights/overview`;
const eventsUrl = `${base}/api/insights/events?current=1&pageSize=20`;
const trackUrl = `${base}/api/insights/track`;
const phases: any[] = [];
const lanes: any[] = [];
const quietBaselines: Array<{ rss: number; pidsCurrent: number }> = [];
const laneWork: Array<[string, () => Promise<any>]> = [
    ["overview-read", () => lane(fetcher, () => readSample(fetcher, overviewUrl, token, clock, overviewEnvelope), clock)],
    ["events-read", () => lane(fetcher, () => readSample(fetcher, eventsUrl, token, clock, eventsEnvelope), clock)],
    ["track-write", () => pacedTrackLane(fetcher, trackUrl, projectKey, clock)],
];
for (let round = 1; round <= 2; round += 1) {
    for (const [name, work] of laneWork) {
        const observed = await phase(`lane-${name}-${round}`, clock, usageAnalytics, currentServices, work);
        stable(observed); phases.push(observed);
        lanes.push({ name, round, ...observed.result, durationMs: observed.workDurationMs, maxima: maxima(observed.snapshots) });
    }
    const drain = await phase(`drain-${round}`, clock, usageAnalytics, currentServices, () => clock.sleep(5000));
    const quiet = await phase(`quiet-${round}`, clock, usageAnalytics, currentServices, () => clock.sleep(30_000));
    stable(drain); stable(quiet); phases.push(drain, quiet);
    quietBaselines.push({ rss: quiet.snapshots[1]!.rss, pidsCurrent: quiet.snapshots[1]!.pidsCurrent });
}
compareQuiet(quietBaselines[0]!, quietBaselines[1]!);

// --- fault boundary -----------------------------------------------------------
const fault = await phase("fault", clock, usageAnalytics, currentServices, async () => {
    const milestones: any[] = [];
    await run(["exec", context.containerName, "systemctl", "stop", "rz-insights.service"]);
    milestones.push({ step: "stop" });
    let failedStatus: number | null = null;
    for (let attempt = 0; attempt < 60; attempt += 1) {
        const sample = await readSample(fetcher, overviewUrl, token, { ...clock, now: () => Date.now() }, () => true, 2000);
        if (!sample.ok && typeof sample.status === "number" && sample.status >= 500) { failedStatus = sample.status; break; }
        if (!sample.ok && sample.failure && failedStatus === null && sample.failure !== "http") { failedStatus = 502; break; }
        await clock.sleep(250);
    }
    if (failedStatus === null) throw Error("outage read never failed during the insights stop");
    milestones.push({ step: "apiFailed", status: failedStatus });
    await run(["exec", context.containerName, "systemctl", "start", "rz-insights.service"]);
    const after = await currentServices();
    const insightsListener = await listener(run, context.containerName, 19802, after.insights.sha256);
    restartedOwner(before.insights, after.insights, insightsListener);
    milestones.push({ step: "listenerReady", pid: after.insights.pid, sha256: after.insights.sha256 });
    const recovered = await readSample(fetcher, overviewUrl, token, clock, overviewEnvelope, 10_000);
    if (!recovered.ok) throw Error("overview did not recover after the insights restart");
    milestones.push({ step: "registryHealthy" });
    return milestones;
});
analyticsFault(fault.result, before.insights.pid, before.insights.sha256);
phases.push(fault);

// --- recovery lane ------------------------------------------------------------
const recoveryPhase = await phase("recovery", clock, usageAnalytics, currentServices, () => lane(fetcher, () => readSample(fetcher, overviewUrl, token, clock, overviewEnvelope), clock, 10_000, 512));
stable(recoveryPhase); phases.push(recoveryPhase);
recoveryLane(recoveryPhase.result.completed);

// --- stability of inputs and references ---------------------------------------
const inputFiles = [context.releaseResult, context.certificate, context.publicKey, context.passwordFile, context.projectKeyFile];
const inputs: Record<string, string> = {};
for (const file of inputFiles) inputs[file] = sha256(new Uint8Array(await readFile(file)));
if ((await readFile(context.passwordFile, "utf8")).trim() !== password || (await readFile(context.projectKeyFile, "utf8")).trim() !== projectKey) throw Error("credential changed while run");
for (const [file, digest] of Object.entries(inputs)) if (sha256(new Uint8Array(await readFile(file))) !== digest) throw Error(`input changed while run: ${file}`);
if (sha256(new Uint8Array(await readFile(context.runtimeEvidence))) !== context.runtimeEvidenceSha256) throw Error("runtime evidence changed while run");
if (sha256(new Uint8Array(await readFile(context.browserReceipt))) !== context.browserReceiptSha256) throw Error("browser receipt changed while run");
const finalHealth = await (await fetch(`${base}/health`)).json();
if (finalHealth?.selectedBinding?.buildId !== native.selection.buildId) throw Error("final health binding differs");

const sampledReadings = phases.flatMap((p) => p.snapshots as Reading[]);
const overallMaxima = maxima(sampledReadings);
if (overallMaxima.rss > 384 * 1024 * 1024 || overallMaxima.hwm > 384 * 1024 * 1024 || overallMaxima.pidsPeak > 64 || overallMaxima.memoryPeak > 384 * 1024 * 1024) throw Error("resource peak exceeded");
for (const key of ["memory.events.max", "memory.events.oom", "memory.events.oom_kill", "pids.events.max"])
    if ((phases[0]!.snapshots[0] as Reading).events[key] !== (phases.at(-1)!.snapshots[1] as Reading).events[key]) throw Error(`cgroup event changed: ${key}`);

const receipt = parseAnalyticsLoadReceipt({
    schemaVersion: 1,
    kind: "analytics-load-evidence",
    load: true,
    releaseReady: false,
    inputs,
    selection: { preset: "analytics", target: native.selection.target, compositionId: native.selection.compositionId, buildId: native.selection.buildId },
    release: { keyId: native.release.keyId, certificateSha256: native.release.certificateSha256, manifestSha256: release.manifestSha256, archiveSha256: release.archiveSha256, envelopeSha256: release.envelopeSha256 },
    corpus: { visitors: new Set(corpus.flat().map((body: any) => body.visitorId)).size, seedEventsPerVisitor: 3, seedOffered: corpus.flat().length },
    lanes: lanes.map(({ name, round, durationMs, warmupCount, offered, completed, failures, p95Ms, p99Ms }) => ({ name, round, durationMs, warmupCount, offered, completed, failures, p95Ms, p99Ms })),
    quietBaselines,
    fault: fault.result,
    faultDurationMs: fault.workDurationMs,
    recovery: { durationMs: recoveryPhase.workDurationMs, warmupCount: recoveryPhase.result.warmupCount, offered: recoveryPhase.result.offered, completed: recoveryPhase.result.completed, failures: recoveryPhase.result.failures, p95Ms: recoveryPhase.result.p95Ms, p99Ms: recoveryPhase.result.p99Ms },
    initialServices: { admin: { pid: before.admin.pid, sha256: before.admin.sha256 }, insights: { pid: before.insights.pid, sha256: before.insights.sha256 } },
    runtimeRef: { path: context.runtimeEvidence, sha256: context.runtimeEvidenceSha256 },
    browserRef: { path: context.browserReceipt, sha256: context.browserReceiptSha256 },
});
const serialized = canonicalJson(receipt);
if (serialized.includes(password) || serialized.includes(projectKey) || serialized.includes(token)) throw Error("receipt carries runtime secret material");
await writeFile(`${output}/receipt.json`, new TextEncoder().encode(serialized));
console.log(canonicalJson({ lanes: lanes.length, offered: lanes.reduce((sum, lane) => sum + lane.offered, 0), recovery: recoveryPhase.result.completed }));
