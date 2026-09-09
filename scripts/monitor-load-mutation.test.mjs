import { expect, test } from "bun:test";
import { mkdtemp, readFile, readdir, rename, rm, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { cwd } from "node:process";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { completeSelectedApiContractForTest } from "../distribution/selected-contract-validator.ts";
import { createExport, releaseVersion, selection, sourceIdentity } from "../distribution/container-export-test-fixture.ts";
import { verifyContainerExport } from "../distribution/container-export-validator.ts";
import { admitted, directory, freshOutput, json, parseMonitorLoadEvidence, publish, stable } from "./monitor-load-admission.ts";
import { pureMonitorAbsence } from "./monitor-load-signed.ts";
import { boundedProcess, certifiedOwner, parseListenerOwner, parseServiceOwner, restartedOwner, service, usage } from "./monitor-load-runtime.ts";
import { image, inspected, process as attestedProcess } from "./verify-selected-web-runtime-attestation.ts";
import { milestones, stableOutage } from "./monitor-load-fault.ts";
import { compareQuiet, quiet } from "./monitor-load-sampler.ts";
import { recovery, resources } from "./monitor-load-contract.ts";
import { parseSelectedWebBootstrapBrowserReceipt } from "./selected-web-bootstrap-browser-receipt.ts";
import { parseMonitorLoadReceipt } from "./monitor-load-receipt-schema.ts";

const hash = "a".repeat(64), health = { status: "ok", selectedBinding: { buildId: "a".repeat(64), compositionId: "a".repeat(64) } }, runtime = { containerId: "container", containerPort: 8080, dev: "1", hostPort: 3000, imageId: "image", ino: "2", pid: 3, sha256: hash }, native = {
    browser: false, checks: [], health: {}, installation: {}, kind: "monitor-native-runtime-evidence", load: false,
    markers: {}, platform: {}, release: { certificateSha256: hash, manifestSha256: hash, archiveSha256: hash, envelopeSha256: hash }, releaseReady: false,
    runtime: true, schemaVersion: 1, selection: { buildId: hash, compositionId: hash }, services: [],
}, browser = {
    schemaVersion: 1, status: "passed", chromiumVersion: "Chromium", integritySensitivityPassed: true,
    adminHealth: { initial: health, final: health }, cases: ["success", "bindingMismatch", "bindingNetworkFailure", "sriEntryFailure"].map(name => ({ case: name, assertions: { bindingCredentialOmitted: true, bindingFirstActive: true, entryExecuted: name === "success", failureApiAbsent: true }, health: { before: health, after: health }, browser: { alert: name === "success" ? false : true, entryExecuted: name === "success", manualRetry: name === "success" ? null : {canonicalDocument:"/monitoring/nodes?q=web",postRetryAutomaticReloadCount:1}, reloadCount: name === "success" ? 0 : 1, stamp: hash, url: name === "success" ? "http://127.0.0.1/login" : "http://127.0.0.1/monitoring/nodes?q=web&__rz_web_reload=x#node-1", value: name === "success" ? {buildId:hash,compositionId:hash,webDigest:hash,login:true} : null }, requests: [{ authorization: false, cookie: false, method: "GET", path: "/__web-binding" }, ...(name === "success" || name === "sriEntryFailure" ? [{ authorization: false, cookie: true, method: "GET", path: "/assets/app.js" }] : []), ...(name === "success" ? [{authorization:false,cookie:true,method:"POST",path:"/api/auth/login"},{authorization:true,cookie:true,method:"GET",path:"/api/installation"}] : [])] })), sensitivity: { requests: [{authorization:false,cookie:false,method:"GET",path:"/__web-binding"}], marker: true, health: { before: health, after: health } }, verifier: { sources: [] },
    digests: { buildId: hash, compositionId: hash, html: hash, binding: hash, installation: hash, verified: hash, adminBinary: { before: hash, after: hash } },
    release: { certificateSha256: hash, manifestSha256: hash, archiveSha256: hash, envelopeSha256: hash, buildId: hash, binaryDigests: [{ path: "bin/rz-admin", sha256: hash }], selection: { artifactClass: "server", compositionId: hash, target: "x86_64-unknown-linux-musl" } }, runtime: { before: runtime, after: runtime }, sourceIdentity: { expected: "source", current: "source" },
};

test("P8e/P8f admission rejects unknown fields and noncanonical files", async () => {
    expect(admitted(native, browser)).toMatchObject({ selection: native.selection });
    expect(() => admitted({ ...native, unknown: true }, browser)).toThrow("schema fields");
    expect(() => admitted(native, { ...browser, unknown: true })).toThrow();
    const root = await mkdtemp(join(tmpdir(), "rz-load-admission-")), file = join(root, "input.json");
    try {
        await writeFile(file, '{"b":2,"a":1}\n');
        await expect(json(file)).rejects.toThrow("canonical");
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("shared P8f parser admits the real receipt and rejects every protected section mutation", async () => {
    const path = "target/rz/p8fb-browser-postcommit-evidence-20260909T0305/manifest.json", real = await Bun.file(path).json();
    expect(parseSelectedWebBootstrapBrowserReceipt(real)).toBe(real);
    for (const mutate of [value => { value.cases[0].assertions.extra = true; }, value => { value.runtime.after.pid++; }, value => { value.verifier.sources[0].sha256 = "bad"; }, value => { value.digests.extra = hash; }]) {
        const changed = structuredClone(real); mutate(changed); expect(() => parseSelectedWebBootstrapBrowserReceipt(changed)).toThrow();
    }
    for (const mutate of [value => { value.cases[0].browser.entryExecuted = false; }, value => { value.cases[1].browser.reloadCount = 0; }, value => { value.cases[2].requests[0].authorization = true; }, value => { value.cases[3].assertions.failureApiAbsent = false; }, value => { value.sourceIdentity.current = "changed"; }, value => { value.adminHealth.final.status = "bad"; }]) {
        const changed = structuredClone(real); mutate(changed); expect(() => parseSelectedWebBootstrapBrowserReceipt(changed)).toThrow();
    }
    for (const mutate of [value => { value.cases[0].browser.value = null; }, value => { value.cases[0].browser.stamp = hash; }, value => { value.cases[0].browser.url = "http://example.invalid/login"; }, value => { value.sensitivity.marker = false; }, value => { value.sensitivity.requests.find(x => x.path === "/__web-binding").cookie = true; }, value => { value.cases[0].browser.value.login = false; }, value => { value.cases[0].browser.value.extra = true; }, value => { value.cases[0].health.before.selectedBinding.buildId = hash; }]) { const changed=structuredClone(real); mutate(changed); expect(() => parseSelectedWebBootstrapBrowserReceipt(changed)).toThrow(); }
    for (const index of [1, 2, 3]) for (const mutate of [value => { value.cases[index].browser.value = {}; }, value => { value.cases[index].browser.stamp = "b".repeat(64); }]) { const changed = structuredClone(real); mutate(changed); expect(() => parseSelectedWebBootstrapBrowserReceipt(changed)).toThrow(); }
});

test("signed verified export rejects a notifications owner mutation", async () => {
    const root = await createExport();
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity, releaseVersion);
        expect(pureMonitorAbsence(snapshot).forbidden).toEqual({ api: [], web: [], schema: [], services: [] });
        const changed = completeSelectedApiContractForTest({ ...selection, preset: "monitor-notify" });
        changed.preset = "monitor";
        const fake = {
            selection: () => snapshot.selection(), paths: () => snapshot.paths(), manifest: () => snapshot.manifest(), recordedProvenance: () => snapshot.recordedProvenance(),
            artifact(path) { return path === "release/contracts/api/api.json" ? { entry: { path, mode: "0644", size: 0, sha256: hash }, bytes: new TextEncoder().encode(canonicalJson(changed)) } : snapshot.artifact(path); },
        };
        expect(() => pureMonitorAbsence(fake)).toThrow();
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("pure monitor accepts third-party Web module IDs but rejects selected notification routes", async () => {
    const root = await createExport();
    try {
        const snapshot = await verifyContainerExport(root, selection, sourceIdentity, releaseVersion);
        const inventory = await Bun.file(join(root, "release/web/inventory.json")).json();
        inventory.moduleIds.push("node_modules/third-party-notifications/index.js");
        const fake = { selection: () => snapshot.selection(), artifact(path) { return path === "release/web/inventory.json" ? { bytes: new TextEncoder().encode(JSON.stringify(inventory)) } : snapshot.artifact(path); } };
        expect(pureMonitorAbsence(fake).forbidden.web).toEqual([]);
        inventory.selectedRoutes.push("notifications.tsx");
        expect(() => pureMonitorAbsence(fake)).toThrow();
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("usage preserves cgroup readings when a service MainPID is zero", async () => {
    const source = await Bun.file("scripts/monitor-load-runtime.ts").text();
    expect(source).toContain('test "$p" -gt 1 && test -r /proc/$p/status || continue');
    const sample = await usage(async () => "memory.max=536870912\nmemory.current=1\nmemory.peak=2\npids.max=256\npids.current=0\npids.peak=0\nmemory.events.max=0\nmemory.events.oom=0\nmemory.events.oom_kill=0\npids.events.max=0\nrss=0\nhwm=0", "container");
    expect(sample).toMatchObject({ rss: 0, hwm: 0, pidsCurrent: 0, pidsPeak: 0, memoryCurrent: 1, memoryPeak: 2 });
});

test("bounded process stays asynchronous, drains both pipes, and kills a timeout", async () => {
    let timerFired = false;
    setTimeout(() => { timerFired = true; }, 10);
    const bytes = 256 * 1024, output = await boundedProcess([
        Bun.argv[0], "-e",
        `await Bun.sleep(80); process.stdout.write("x".repeat(${bytes})); process.stderr.write("y".repeat(${bytes}));`,
    ], 1000);
    expect(timerFired).toBeTrue();
    expect(output.length).toBe(bytes);

    const root = await mkdtemp(join(tmpdir(), "rz-load-process-")), pidFile = join(root, "pid");
    try {
        await expect(boundedProcess([
            Bun.argv[0], "-e",
            "process.on('SIGTERM',()=>{}); await Bun.write(process.argv[1], String(process.pid)); await Bun.sleep(10000);",
            pidFile,
        ], 75)).rejects.toThrow("timed out after 75ms");
        const pid = Number(await readFile(pidFile, "utf8"));
        await Bun.sleep(20);
        expect(() => process.kill(pid, 0)).toThrow();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("runtime ownership parsers reject port image container and listener ambiguity", () => {
    const inspect = [{ Id: "container", Image: "image", State: { Running: true }, Platform: "linux", NetworkSettings: { Ports: { "8080/tcp": [{ HostIp: "127.0.0.1", HostPort: "3000" }] } } }];
    const runtime = inspected(inspect, "127.0.0.1", 3000);
    expect(() => inspected([{ ...inspect[0], NetworkSettings: { Ports: { "8080/tcp": [{ HostIp: "127.0.0.1", HostPort: "3000" }, { HostIp: "0.0.0.0", HostPort: "3000" }] } } }], "127.0.0.1", 3000)).toThrow();
    expect(() => inspected([{ ...inspect[0], Id: "", }], "127.0.0.1", 3000)).toThrow();
    expect(() => image([{ Os: "linux", Architecture: "arm64" }])).toThrow();
    expect(() => attestedProcess(`1\\t2\\t3\\t${hash}\\n2\\t2\\t3\\t${hash}`, runtime, hash)).toThrow();
    expect(() => parseListenerOwner([["1", "2", "3", hash], ["2", "2", "3", hash]], hash)).toThrow();
    expect(() => parseServiceOwner(["x", "2", "3", hash])).toThrow();
});

test("service ownership resolves the installed current symlink before comparison", async () => {
    let command = [];
    const owner = await service(async args => { command = args; return `7 2 3 ${hash}`; }, "container", "rz-admin.service", "/opt/rz/current/bin/rz-admin");
    expect(owner.pid).toBe(7);
    expect(command[4]).toContain("expected=$(readlink -f /opt/rz/current/bin/rz-admin)");
    expect(command[4]).toContain('test "$exe" = "$expected"');
});

test("P8g rejects recovery undershoot, forbidden faults, resource spikes and quiet/event drift", () => {
    recovery(512); expect(() => recovery(511)).toThrow();
    expect(() => stableOutage([{ at: 1, kind: "response", status: 200, code: 0 }])).toThrow();
    expect(() => milestones([{ step: "stop", at: 2 }, { step: "listenerGone", at: 1 }, { step: "listenerReady", at: 3 }, { step: "registryHealthy", at: 4 }])).toThrow();
    const events = { "memory.max": 0, "memory.events.max": 0, "memory.events.oom": 0, "memory.events.oom_kill": 0, "pids.events.max": 0 };
    expect(() => resources(events, { rss: 384 * 1024 * 1024 + 1, hwm: 1, pidsPeak: 1, memoryPeak: 1 }, events)).toThrow();
    expect(() => resources(events, { rss: 1, hwm: 1, pidsPeak: 1, memoryPeak: 1 }, { ...events, "pids.events.max": 1 })).toThrow();
    expect(() => compareQuiet({ rss: 1, pidsCurrent: 1 }, { rss: 16 * 1024 * 1024 + 2, pidsCurrent: 1 })).toThrow();
    expect(() => stableOutage([{ at: 0, kind: "response", status: 503, code: 40001 }, { at: 250, kind: "response", status: 503, code: 40001 }, { at: 500, kind: "response", status: 503, code: 40001 }])).toThrow();
});

test("P8e historical PID does not constrain current certified process or restart", () => {
    const before = { pid: 101, dev: "1", ino: "2", sha256: hash }, next = { pid: 202, dev: "1", ino: "2", sha256: hash };
    certifiedOwner({ ...before, pid: 9 }, hash);
    restartedOwner(before, next, next);
    expect(() => certifiedOwner(next, "b".repeat(64))).toThrow();
});

test("stable input rejects symlinks and replacement is observable; publish cleans a failed temporary", async () => {
    expect(await Bun.file("scripts/verify-monitor-load-certification.ts").text()).not.toContain(".map(stable)");
    const verifier = await Bun.file("scripts/verify-monitor-load-certification.ts").text();
    expect(verifier).toContain('{ path: get("--admin-bin"), limit: 64 * 1024 * 1024 }');
    expect(verifier).toContain("stable(input.path, spec.limit)");
    const root = await mkdtemp(join(tmpdir(), "rz-load-stable-")), input = join(root, "input"), link = join(root, "link");
    try {
        await writeFile(input, "one"); await symlink(input, link);
        await expect(stable(link)).rejects.toThrow();
        const large = join(root, "large"); await writeFile(large, new Uint8Array(2 * 1024 * 1024 + 1));
        await expect(stable(large)).rejects.toThrow(); const largeSnapshot = await stable(large, 64 * 1024 * 1024); expect((await stable(large, 64 * 1024 * 1024)).sha256).toBe(largeSnapshot.sha256);
        const before = await stable(input); await writeFile(join(root, "replacement"), "two"); await rename(join(root, "replacement"), input);
        expect((await stable(input)).sha256).not.toBe(before.sha256);
        const target = join(cwd(), "target/rz"), output = join(target, `monitor-load-failure-${crypto.randomUUID()}`), beforeTemps = new Set((await readdir(target)).filter(name => name.startsWith(`${output.split("/").at(-1)}.tmp-`)));
        await writeFile(output, "block");
        await expect(publish(output, { ok: true })).rejects.toThrow();
        expect((await readdir(target)).filter(name => name.startsWith(`${output.split("/").at(-1)}.tmp-`)).every(name => beforeTemps.has(name))).toBeTrue();
        await rm(output, { force: true });
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("fresh load output admits a direct target/rz child and rejects escape", async () => {
    const output = join(cwd(), "target/rz", `monitor-load-${crypto.randomUUID()}`);
    expect(await freshOutput(output)).toBe(output);
    await expect(freshOutput(join(tmpdir(), `monitor-load-${crypto.randomUUID()}`))).rejects.toThrow();
    await symlink(`${output}.missing`, output);
    try { await expect(freshOutput(output)).rejects.toThrow(); }
    finally { await rm(output, { force: true }); }
});

test("export directory admission accepts a canonical directory and rejects a symlink", async () => {
    const target = join(cwd(), "target/rz"), link = join(cwd(), "target", `rz-link-${crypto.randomUUID()}`);
    expect(await directory(target)).toBe(target);
    await symlink(target, link);
    try { await expect(directory(link)).rejects.toThrow(); }
    finally { await rm(link, { force: true }); }
});

test("phase receipt parses, publishes canonically, and rejects phase binding mutations", async () => {
    const events = { "memory.max":536870912, "memory.events.max":0, "memory.events.oom":0, "memory.events.oom_kill":0, "pids.events.max":0 }, admin={pid:2,dev:"1",ino:"2",sha256:hash}, monitor={pid:3,dev:"1",ino:"3",sha256:hash}, restarted={...monitor,pid:4}, latencies=Array.from({length:3200},()=>1), recoveryLatencies=Array.from({length:512},()=>1);
    const reading=(at,hwm,peak=3)=>({at,rss:1,hwm,pidsCurrent:2,pidsPeak:peak,memoryCurrent:1,memoryPeak:2,events}), make=(name,work,at,before,after=before,hwm=10)=>({name,workDurationMs:work,snapshots:[reading(at,hwm),reading(at+work,hwm)],services:{before:{admin,monitor:before},after:{admin,monitor:after}}});
    const phases=[make("lane-1",60000,0,monitor),make("drain-1",5000,60000,monitor),make("quiet-1",30000,65000,monitor),make("lane-2",60000,95000,monitor),make("drain-2",5000,155000,monitor),make("quiet-2",30000,160000,monitor),{...make("fault",1000,190000,monitor,restarted,10),snapshots:[reading(190000,10),reading(191000,1)]},make("recovery",10000,191000,restarted,restarted,2)];
    const lane=(phase,drain,quiet)=>({durationMs:phase.workDurationMs,drainDurationMs:drain.workDurationMs,quietDurationMs:quiet.workDurationMs,warmupCount:128,offered:3200,completed:3200,failures:0,p95Ms:1,p99Ms:1,latenciesMs:latencies,snapshots:phase.snapshots,maxima:{rss:1,hwm:10,pidsPeak:3,memoryCurrent:1,memoryPeak:2}}), inputs={"/inputs/a":hash,"/inputs/b":hash,"/inputs/c":hash,"/inputs/d":hash,"/inputs/e":hash,"/inputs/f":hash,"/inputs/g":hash,"/inputs/h":hash}, sidecars=Object.fromEntries(Array.from({length:12},(_,i)=>[`/p8e/${i}`,hash]));
    const boundary=Array.from({length:4},(_,i)=>({at:190101+i*250,kind:"response",status:503,code:40001})), crossing=Array.from({length:4},()=>({start:1,at:190001,end:190001,kind:"response",status:503,code:40001}));
    const receipt={schemaVersion:1,kind:"monitor-load-evidence",load:true,inputs,p8eSidecars:sidecars,selection:{preset:"monitor",target:"x86_64-unknown-linux-musl",artifactClass:"server",compositionId:hash,buildId:hash},release:{},source:{},browserRuntime:{},corpus:{nodes:100,disksPerNode:4,sentinel:"node"},lanes:[lane(phases[0],phases[1],phases[2]),lane(phases[3],phases[4],phases[5])],quietBaselines:[{at:95000,rss:1,pidsCurrent:2},{at:190000,rss:1,pidsCurrent:2}],recovery:{completed:512,offered:512,durationMs:10000,warmupCount:128,failures:0,p95Ms:1,p99Ms:1,latenciesMs:recoveryLatencies},fault:[{step:"stop",at:190100},{step:"listenerGone",at:190851,status:503,code:40001},{step:"listenerReady",at:190900,pid:restarted.pid,sha256:restarted.sha256},{step:"registryHealthy",at:190950}],faultDurationMs:1000,boundary,boundaryInFlight:crossing.map((x,i)=>({...x,start:190000,at:190101+i,end:190101+i})),resources:phases[0].snapshots[0],initialServices:{admin,monitor},pureMonitorSse:{runtimeStatus:404,signedAbsence:{}},overallMaxima:{rss:1,hwm:10,pidsPeak:3,memoryCurrent:1,memoryPeak:2},phases};
    expect(parseMonitorLoadReceipt(receipt)).toBe(receipt);
    const output=join(cwd(),"target/rz",`monitor-load-phase-${crypto.randomUUID()}`); try { await publish(output,receipt,parseMonitorLoadReceipt); expect(await readFile(join(output,"monitor-load-evidence.json"),"utf8")).toBe(canonicalJson(receipt)); } finally { await rm(output,{recursive:true,force:true}); }
    expect(()=>parseMonitorLoadReceipt({...receipt,faultDurationMs:999})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,lanes:[{...receipt.lanes[0],p95Ms:2},receipt.lanes[1]]})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,lanes:[{...receipt.lanes[0],maxima:{...receipt.lanes[0].maxima,hwm:9}},receipt.lanes[1]]})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,phases:receipt.phases.map((p,i)=>i===1?{...p,services:{...p.services,before:{...p.services.before,monitor:{...monitor,pid:9}}}}:p)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,phases:receipt.phases.map((p,i)=>i===4?{...p,snapshots:[p.snapshots[0],{...p.snapshots[1],memoryPeak:1}]}:p)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,phases:receipt.phases.map((p,i)=>i===4?{...p,snapshots:[p.snapshots[0],{...p.snapshots[1],events:{...p.snapshots[1].events,"pids.events.max":1}}]}:p)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,fault:receipt.fault.map((x,i)=>i===2?{...x,pid:99}:x)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,boundary:receipt.boundary.map((x,i)=>i===2?{...x,status:200,code:0}:x)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,boundary:receipt.boundary.map((x,i)=>i===2?{...x,at:190200}:x)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,boundaryInFlight:receipt.boundaryInFlight.map((x,i)=>i===2?{...x,at:190900,end:190900}:x)})).toThrow();
    expect(()=>parseMonitorLoadReceipt({...receipt,phases:receipt.phases.map((p,i)=>i===3?{...p,snapshots:p.snapshots.map(x=>({...x,at:x.at-95000}))}:p)})).toThrow();
});
