import { corpus, fault, lane, recovery, request, resources, seed, type Clock } from "./monitor-load-contract.ts";
import { admitted, credential, directory, freshOutput, json, nativeEvidenceSummary, publish, revalidatedNativeEvidence, stable } from "./monitor-load-admission.ts";
import { parseMonitorLoadReceipt } from "./monitor-load-receipt-schema.ts";
import { pureMonitorAbsence, signedSourceBuild } from "./monitor-load-signed.ts";
import { compareQuiet, faultPhase, maxima, phase, quiet as quietBaseline, stablePhase } from "./monitor-load-sampler.ts";
import { classify, controlledBoundary, milestones, stableOutage } from "./monitor-load-fault.ts";
import { certifiedOwner, docker as run, listener, listenerGone, restartedOwner, service, usage } from "./monitor-load-runtime.ts";
import { image, inspected } from "./verify-selected-web-runtime-attestation.ts";
const names = [
    "--native-runtime-evidence",
    "--browser-receipt",
    "--export-root",
    "--release-result",
    "--certificate",
    "--public-key",
    "--expected-source-identity",
    "--admin-bin",
    "--admin-url",
    "--password-file",
    "--agent-token-file",
    "--runtime-container",
    "--output",
] as const;
type Name = (typeof names)[number];
let a = new Map<Name, string>();
for (let i = 2; i < Bun.argv.length; i += 2) {
    let k = Bun.argv[i] as Name,
        v = Bun.argv[i + 1];
    if (!names.includes(k) || !v || a.has(k))
        throw Error("invalid P8g arguments");
    a.set(k, v);
}
if (a.size !== names.length) throw Error("missing P8g arguments");
let get = (key: Name) => a.get(key)!, output = await freshOutput(get("--output")), exportRoot = await directory(get("--export-root")), native = await revalidatedNativeEvidence(get("--native-runtime-evidence"), get("--release-result")) as unknown as Record<string, unknown>, browser = await json(get("--browser-receipt")),
    admission = admitted(native, browser),
    p8eSidecars = await nativeEvidenceSummary(get("--native-runtime-evidence"), get("--release-result")),
    inputSpecs = [
        { path: get("--release-result"), limit: 2 * 1024 * 1024 },
        { path: get("--certificate"), limit: 2 * 1024 * 1024 },
        { path: get("--public-key"), limit: 2 * 1024 * 1024 },
        { path: get("--admin-bin"), limit: 64 * 1024 * 1024 },
        { path: get("--native-runtime-evidence"), limit: 2 * 1024 * 1024 },
        { path: get("--browser-receipt"), limit: 2 * 1024 * 1024 },
        { path: get("--password-file"), limit: 2 * 1024 * 1024 },
        { path: get("--agent-token-file"), limit: 2 * 1024 * 1024 },
    ], inputs = await Promise.all(inputSpecs.map(({ path, limit }) => stable(path, limit)));
if (admission.source.expected !== get("--expected-source-identity"))
    throw Error("source identity differs");
let { verified, snapshot } = await signedSourceBuild({
    exportRoot, expectedSourceIdentity: get("--expected-source-identity"),
    releaseResult: get("--release-result"), certificate: get("--certificate"), publicKey: get("--public-key"), adminSha256: inputs[3]!.sha256,
});
if (
    verified.certificateSha256 !== admission.release.certificateSha256 ||
    verified.manifestSha256 !== admission.release.manifestSha256 ||
    verified.archiveSha256 !== admission.release.archiveSha256 ||
    verified.envelopeSha256 !== admission.release.envelopeSha256 ||
    verified.buildId !== admission.selection.buildId ||
    verified.selection?.compositionId !== admission.selection.compositionId ||
    verified.binaryDigests?.find((x: { path?: unknown }) => x.path === "bin/rz-admin")?.sha256 !== inputs[3]!.sha256 || JSON.stringify(verified.binaryDigests) !== JSON.stringify((native.release as Record<string, unknown>).binaryDigests)
) throw Error("signed selected API/Web/schema tuple differs");
let absence = pureMonitorAbsence(snapshot);
let base = get("--admin-url");
if (!/^http:\/\/127\.0\.0\.1:[1-9][0-9]*$/.test(base))
    throw Error("unsafe load input");
let before = await tuple(get("--runtime-container"), native, admission.runtime),
    password = await credential(get("--password-file")), agent = await credential(get("--agent-token-file"));
if (!password || !agent) throw Error("empty credential input");
let clock: Clock = { now: () => Math.floor(performance.now()), sleep: (ms) => Bun.sleep(ms) },
    login = await fetch(base + "/api/auth/login", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ username: "owner", password }),
        signal: AbortSignal.timeout(2000),
    }),
    token = ((await login.json()) as { data?: { token?: unknown } }).data
        ?.token;
if (!login.ok || typeof token !== "string") throw Error("owner login failed");
let reports = corpus(String(admission.selection.buildId).slice(0, 12)), ids = reports.map((x) => x.nodeId);
await seed(fetch, base, agent, reports);
if (!(await request(fetch, base + "/api/monitor/nodes", token, ids, clock)).ok)
    throw Error("seeded gateway read failed");
let phases: any[] = [], lanes: any[] = [], quietBaselines = [],
    currentServices = () => tuple(get("--runtime-container"), native, admission.runtime);
for (let round = 0; round < 2; round++) {
    let observed = await phase(`lane-${round + 1}`, clock, () => usage(run, get("--runtime-container")), currentServices, () => lane(fetch, base + "/api/monitor/nodes", token, ids, clock));
    stablePhase(observed); phases.push(observed);
    lanes.push({
        ...observed.result, durationMs: observed.workDurationMs,
        snapshots: observed.snapshots, maxima: maxima(observed.snapshots),
    });
    let drain = await phase(`drain-${round + 1}`, clock, () => usage(run, get("--runtime-container")), currentServices, () => clock.sleep(5000));
    let quiet = await phase(`quiet-${round + 1}`, clock, () => usage(run, get("--runtime-container")), currentServices, () => clock.sleep(30_000));
    stablePhase(drain); stablePhase(quiet); phases.push(drain, quiet);
    quietBaselines.push(quietBaseline(quiet.snapshots[1]));
    Object.assign(lanes[round]!, { drainDurationMs: drain.workDurationMs, quietDurationMs: quiet.workDurationMs });
}
compareQuiet(quietBaselines[0]!, quietBaselines[1]!);
let observedFault = await phase("fault", clock, () => usage(run, get("--runtime-container")), currentServices, () => outage(get("--runtime-container"), base, token, ids, before.monitor, clock)), events = observedFault.result;
faultPhase(observedFault); phases.push(observedFault);
stableOutage(events.boundary);
milestones(events.milestones);
fault(events.milestones, before.monitor.pid, before.monitor.sha256);
let observedRecovery = await phase("recovery", clock, () => usage(run, get("--runtime-container")), currentServices, () => lane(fetch, base + "/api/monitor/nodes", token, ids, clock, 10_000, 512)), recoveryLane = observedRecovery.result;
stablePhase(observedRecovery); phases.push(observedRecovery);
recovery(recoveryLane.completed);
let sampledReadings = phases.flatMap(phase => phase.snapshots), finalReading = observedRecovery.snapshots[1];
if (!finalReading) throw Error("recovery observation is short");
resources(phases[0]!.snapshots[0].events, maxima(sampledReadings), finalReading.events);
let evidencePhases = phases.map(({ result: _result, ...evidence }) => evidence);
let sse = await fetch(base + "/api/notifications/stream", {
    headers: { authorization: `Bearer ${token}` },
    signal: AbortSignal.timeout(2000),
});
if (sse.status !== 404) throw Error("pure Monitor SSE route exists");
await sse.arrayBuffer();
for (let index = 0; index < inputs.length; index++) {
    let input = inputs[index]!, spec = inputSpecs[index]!;
    if ((await stable(input.path, spec.limit)).sha256 !== input.sha256)
        throw Error("input changed while run");
}
if ((await credential(get("--password-file"))) !== password || (await credential(get("--agent-token-file"))) !== agent || JSON.stringify(await nativeEvidenceSummary(get("--native-runtime-evidence"), get("--release-result"))) !== JSON.stringify(p8eSidecars) || JSON.stringify(await revalidatedNativeEvidence(get("--native-runtime-evidence"), get("--release-result"))) !== JSON.stringify(native))
    throw Error("credential changed while run");
let ending = await signedSourceBuild({
    exportRoot, expectedSourceIdentity: get("--expected-source-identity"),
    releaseResult: get("--release-result"), certificate: get("--certificate"), publicKey: get("--public-key"), adminSha256: inputs[3]!.sha256,
}), endingAbsence = pureMonitorAbsence(ending.snapshot);
if (JSON.stringify(ending.verified) !== JSON.stringify(verified) || JSON.stringify(endingAbsence) !== JSON.stringify(absence))
    throw Error("signed export changed while run");
let receipt = {
    schemaVersion: 1,
    kind: "monitor-load-evidence",
    load: true,
    inputs: Object.fromEntries(inputs.map(input => [input.path, input.sha256])),
    p8eSidecars,
    selection: admission.selection,
    release: admission.release,
    source: admission.source,
    browserRuntime: admission.runtime,
    corpus: { nodes: 100, disksPerNode: 4, sentinel: ids[0] },
    lanes,
    recovery: { ...recoveryLane, durationMs: observedRecovery.workDurationMs },
    fault: events.milestones,
    phases: evidencePhases,
    faultDurationMs: observedFault.workDurationMs,
    boundary: events.boundary,
    resources: phases[0]!.snapshots[0],
    initialServices: before,
    pureMonitorSse: { runtimeStatus: 404, signedAbsence: absence },
    quietBaselines,
    overallMaxima: maxima(sampledReadings),
    boundaryInFlight: events.boundaryInFlight,
};
await publish(output, receipt, parseMonitorLoadReceipt);
async function tuple(
    container: string,
    evidence: Record<string, unknown>,
    browser: unknown,
) {
    if (
        (await run([
            "inspect",
            "--format",
            "{{.HostConfig.NanoCpus}}",
            container,
        ])) !== "4000000000"
    )
        throw Error("container CPU limit differs");
    let b = browser as Record<string, unknown>, raw = JSON.parse(await run(["inspect", container])),
        mapped = inspected(raw, "127.0.0.1", Number(new URL(get("--admin-url")).port));
    image(JSON.parse(await run(["image", "inspect", mapped.imageId])));
    if (mapped.containerId !== b.containerId || mapped.imageId !== b.imageId || mapped.hostPort !== b.hostPort || mapped.containerPort !== b.containerPort) throw Error("runtime Docker mapping differs");
    let
        listenerOwner = await listener(
            run,
            container,
            Number(b.containerPort),
            String(b.sha256),
        );
    let admin = await service(run, container, "rz-admin.service", "/opt/rz/current/bin/rz-admin"),
        monitor = await service(run, container,
            "rz-monitor.service",
            "/opt/rz/current/bin/rz-monitor",
        ),
        binaries = new Map(((evidence.release as Record<string, unknown>).binaryDigests as Array<Record<string, unknown>>).map(x => [x.path, x.sha256]));
    if (
        admin.pid !== b.pid ||
        admin.dev !== b.dev ||
        admin.ino !== b.ino ||
        admin.sha256 !== b.sha256 ||
        listenerOwner.pid !== admin.pid ||
        listenerOwner.dev !== admin.dev ||
        listenerOwner.ino !== admin.ino ||
        (await run(["inspect", "--format", "{{.Id}}", container])) !==
            b.containerId
    )
        throw Error("runtime tuple differs");
    certifiedOwner(admin, binaries.get("bin/rz-admin"));
    certifiedOwner(monitor, binaries.get("bin/rz-monitor"));
    return { admin, monitor };
}
async function outage(
    container: string,
    base: string,
    token: string,
    ids: string[],
    old: { pid: number; dev: string; ino: string; sha256: string },
    clock: Clock,
) {
    await run(["exec", container, "systemctl", "freeze", "rz-monitor.service"]);
    let controlled = await controlledBoundary(
        () => request(fetch, base + "/api/monitor/nodes", token, ids, clock, 10_000),
        () => clock.now(),
        async () => { for (let i=0;i<20;i++) { if (Number(await run(["exec",container,"sh","-ceu","awk '$4==\"01\"&&toupper($2)~/:4D5A$/&&$5!~/:00000000$/{n++}END{print n+0}' /proc/net/tcp /proc/net/tcp6"])) >= 4) return true; await Bun.sleep(100); } await run(["exec", container, "systemctl", "thaw", "rz-monitor.service"]); return false; },
        async () => {
            await run(["exec", container, "systemctl", "freeze", "rz-admin.service"]);
            try {
                await run(["exec", container, "systemctl", "kill", "--kill-whom=main", "--signal=KILL", "rz-monitor.service"]);
                await run(["exec", container, "sh", "-ceu", "test ! -e /proc/$1/exe", "sh", String(old.pid)]);
                let stopAt = clock.now();
                await run(["exec", container, "systemctl", "stop", "rz-monitor.service"]);
                return stopAt;
            } finally {
                await run(["exec", container, "systemctl", "thaw", "rz-admin.service"]);
            }
        },
    );
    await listenerGone(run, container, 19802);
    let started = controlled.stopAt,
        out: Array<{
            step: string;
            status?: number;
            code?: number;
            pid?: number;
            sha256?: string;
            at: number;
        }> = [{ step: "stop", at: started }],
        boundary: Array<{
            at: number;
            status?: number;
            code?: number;
            kind: "response" | "timeout" | "transport";
        }> = [];
    for (let i = 0; i < 20; i++) {
        let result = await request(
            fetch,
            base + "/api/monitor/nodes",
            token,
            ids,
            clock,
        );
        let item = classify(result, clock.now());
        boundary.push(item);
        if (result.status === 503 && result.code === 40001 && boundary.length >= 4 && item.at - boundary[0]!.at >= 750) {
            out.push({
                step: "listenerGone",
                status: 503,
                code: 40001,
                at: item.at,
            });
            break;
        }
        if (result.status !== 503 || result.code !== 40001)
            throw Error("invalid fault boundary");
        await Bun.sleep(250);
    }
    if (out.length !== 2) throw Error("stable outage missing");
    await run(["exec", container, "systemctl", "start", "rz-monitor.service"]);
    let next = await tuple(container, native, admission.runtime), monitorListener = await listener(run, container, 19802, next.monitor.sha256);
    restartedOwner(old, next.monitor, monitorListener);
    out.push({
        step: "listenerReady",
        pid: next.monitor.pid,
        sha256: next.monitor.sha256,
        at: clock.now(),
    });
    let healthy = await request(
        fetch,
        base + "/api/monitor/nodes",
        token,
        ids,
        clock,
    );
    if (!healthy.ok) throw Error("registry unhealthy");
    out.push({ step: "registryHealthy", at: clock.now() });
    return { milestones: out, boundary, boundaryInFlight: controlled.rows };
}
