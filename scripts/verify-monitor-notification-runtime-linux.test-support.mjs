import {
    appendFileSync,
    chmodSync,
    existsSync,
    mkdirSync,
    mkdtempSync,
    readFileSync,
    readdirSync,
    readlinkSync,
    rmSync,
    symlinkSync,
    unlinkSync,
    writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const receipts = [
    "steps.log", "open-event.json", "open-event.json.identity", "selected-ingress-listener.txt",
    "inbox-open.json", "duplicate.json", "bad-signature.json", "unsigned.json",
    "public-internal.json", "inbox-final.json", "selected-state.json", "plain-admin-api.json",
    "plain-admin-config.json", "plain-monitor-api.json", "plain-monitor-config.json",
    "plain-absence.json", "plain-notification-route.json", "plain-listeners.txt",
];

function hash(bytes) {
    return new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
}

function parseDockerEnv(args) {
    const values = {};
    for (let index = 0; index < args.length; index += 1) {
        if (args[index] !== "--env") continue;
        const [key, ...rest] = args[index + 1].split("=");
        values[key] = rest.join("=");
    }
    return values;
}

function writeRuntimeEvidence(args) {
    const mount = args.find((arg) => arg.includes("dst=/verify/evidence"));
    const candidate = mount.match(/src=([^,]+)/)[1];
    const values = parseDockerEnv(args);
    const eventId = "11111111-1111-4111-8111-111111111111";
    const resolvedId = "22222222-2222-4222-8222-222222222222";
    const subjectId = "33333333-3333-4333-8333-333333333333";
    const json = (name, value) => writeFileSync(join(candidate, name), JSON.stringify(value));
    const opened = {
        schemaVersion: 1, eventId, producer: "monitor", topic: "monitor.incident.opened",
        occurredAt: "2026-09-07T00:00:00Z", expiresAt: "2026-09-14T00:00:00Z",
        subject: { kind: "monitor-incident", id: subjectId, revision: 1 },
        audience: { policy: "monitor-incident-readers" },
        content: { title: "CPU high", summary: "Incident opened" },
    };
    const final = {
        code: 0, message: "Success", data: {
            items: [
                { topic: "monitor.incident.resolved", subjectId, subjectRevision: 2 },
                { topic: "monitor.incident.opened", subjectId, subjectRevision: 1 },
            ], nextCursor: null, revision: 2, retentionDays: 30,
        },
    };
    if (process.env.FAKE_TAMPER === "artifact-status") final.data.items[0].topic = "wrong";
    json("open-event.json", opened);
    writeFileSync(join(candidate, "open-event.json.identity"), `${eventId}\t${subjectId}\n`);
    writeFileSync(join(candidate, "steps.log"), "STEP manifest\n");
    writeFileSync(join(candidate, "selected-ingress-listener.txt"), "LISTEN 127.0.0.1:19811\n");
    const inboxOpen = { code: 0, data: { items: [{ topic: opened.topic, subjectId, subjectRevision: 1 }] } };
    if (process.env.FAKE_TAMPER === "open-status") inboxOpen.data.items[0].topic = "wrong";
    json("inbox-open.json", inboxOpen);
    json("duplicate.json", { status: 200, contentType: "application/json", body: { code: "duplicate" } });
    json("bad-signature.json", { status: 401, contentType: "application/json", body: { code: "bad-producer" } });
    json("unsigned.json", { code: "invalid-protocol" });
    json("public-internal.json", { status: 404, contentType: "application/json", body: { code: 10001, message: "API route not found.", data: null } });
    json("inbox-final.json", final);
    json("selected-state.json", {
        receipts: [[eventId, "stored"], [resolvedId, "stored"]],
        messages: [["monitor.incident.opened", subjectId, 1], ["monitor.incident.resolved", subjectId, 2]],
        recipients: 2, outbox: 0,
    });
    json("plain-admin-api.json", { version: 1, routes: [] });
    json("plain-admin-config.json", { version: 1, owner: "access", consumer: "rz-admin", fields: [] });
    json("plain-monitor-api.json", { contractVersion: 1, module: "monitor", routes: [] });
    json("plain-monitor-config.json", { version: 1, owner: "monitor", consumer: "rz-monitor", fields: [] });
    const plainAbsence = { admin: [], monitor: [] };
    if (process.env.FAKE_TAMPER === "plain-status") plainAbsence.admin.push("notifications");
    json("plain-absence.json", plainAbsence);
    json("plain-notification-route.json", { code: 404, data: null, message: "Not found" });
    writeFileSync(join(candidate, "plain-listeners.txt"), "LISTEN 127.0.0.1:19820\n");
    let receiptRows = receipts.map((file) => {
        const bytes = readFileSync(join(candidate, file));
        return { file, sha256: hash(bytes), bytes: bytes.byteLength };
    });
    const manifest = {
        schemaVersion: 1, status: "passed", gitHead: values.RUSTZEN_VERIFY_HEAD,
        sourceTreeState: values.RUSTZEN_VERIFY_SOURCE_TREE_STATE,
        sourceTreeSha256: values.RUSTZEN_VERIFY_SOURCE_TREE_SHA256,
        platform: { architecture: values.RUSTZEN_VERIFY_ARCHITECTURE, name: values.RUSTZEN_VERIFY_PLATFORM },
        binaryHashes: JSON.parse(values.RUSTZEN_VERIFY_BINARY_HASHES),
        buildProvenanceSha256: values.RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256,
        verifier: { imageId: values.RUSTZEN_VERIFY_VERIFIER_IMAGE_ID, key: values.RUSTZEN_VERIFY_VERIFIER_KEY, provenanceSha256: values.RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256 },
        delivery: { openEventId: eventId, subjectId, opened: "stored", duplicate: "duplicate", resolved: "stored", receiptCount: 2, messageCount: 2, recipientCount: 2 },
        authentication: { unsigned: 400, badSignature: 401, publicInternal: 404 },
        pureMonitor: { adminNotificationOwner: false, monitorNotificationOwner: false, notificationConfig: false, notificationRoutes: false, relayTask: false, notificationSchemaObjects: 0, notificationListeners: 0 },
        receipts: receiptRows,
    };
    if (process.env.FAKE_TAMPER === "duplicate-receipt") manifest.receipts.push(receiptRows[0]);
    if (process.env.FAKE_TAMPER === "wrong-status") manifest.delivery.resolved = "duplicate";
    if (process.env.FAKE_TAMPER === "path-traversal") manifest.receipts[0].file = "../steps.log";
    json("manifest.json", manifest);
}

async function fakeDocker(args) {
    process.on("SIGINT", () => process.exit(130));
    process.on("SIGTERM", () => process.exit(143));
    mkdirSync(process.env.FAKE_STATE_ROOT, { recursive: true });
    appendFileSync(process.env.FAKE_CALL_LOG, `${args.join(" ")}\n`);
    const command = args[0];
    if (command === "info") return console.log("aarch64");
    if (command === "logs") {
        if (process.env.FAKE_BLOCK === "logs") await Bun.sleep(30_000);
        return console.log("fake container log");
    }
    if (command === "rm") {
        if (process.env.FAKE_BLOCK === "rm") await Bun.sleep(30_000);
        if (process.env.FAKE_RM_FAIL === "1") process.exit(1);
        const name = args.at(-1);
        const state = join(process.env.FAKE_STATE_ROOT, name);
        if (existsSync(state)) unlinkSync(state);
        return;
    }
    if (command === "container" && args[1] === "inspect") {
        process.exit(existsSync(join(process.env.FAKE_STATE_ROOT, args.at(-1))) ? 0 : 1);
    }
    if (command !== "run") process.exit(2);
    const name = args[args.indexOf("--name") + 1];
    writeFileSync(join(process.env.FAKE_STATE_ROOT, name), String(process.pid));
    const stage = args.includes("rust:1.95-bookworm") ? "build" : "runtime";
    if (process.env.FAKE_BLOCK === stage) await Bun.sleep(30_000);
    if (stage === "build") {
        const bin = join(process.env.FAKE_EVIDENCE_ROOT, "build/aarch64/bin");
        mkdirSync(bin, { recursive: true });
        for (const name of ["rz-admin-notify", "rz-monitor-notify", "rz-admin-pure", "rz-monitor-pure"]) {
            writeFileSync(join(bin, name), `fake-${name}`);
            chmodSync(join(bin, name), 0o755);
        }
        return;
    }
    if (process.env.FAKE_RUNTIME_EXIT) process.exit(Number(process.env.FAKE_RUNTIME_EXIT));
    writeRuntimeEvidence(args);
}

function executable(path, body) {
    writeFileSync(path, body);
    chmodSync(path, 0o755);
}

export function fakeGateFixture(outer, options = {}) {
    const root = mkdtempSync(join(tmpdir(), "rz-monitor-notify-gate-"));
    const evidence = join(root, "evidence");
    const states = join(root, "states");
    const calls = join(root, "docker.calls");
    mkdirSync(join(evidence, "runs/previous"), { recursive: true });
    mkdirSync(states);
    writeFileSync(join(evidence, "runs/previous/manifest.json"), '{"status":"previous"}\n');
    symlinkSync("runs/previous", join(evidence, "current"));
    const source = join(root, "source.sh");
    const verifier = join(root, "verifier.sh");
    const file = join(root, "file.sh");
    const docker = join(root, "docker.sh");
    executable(source, "#!/bin/sh\nprintf 'fakehead\\tclean\\tfakesource\\n'\n");
    executable(verifier, "#!/bin/sh\nprintf 'sha256:fake\\tfakekey\\tfakeprovenance\\n'\n");
    executable(file, "#!/bin/sh\necho 'ELF 64-bit LSB executable, ARM aarch64'\n");
    executable(docker, `#!/bin/sh\nexec '${process.execPath}' '${import.meta.filename}' --fake-docker "$@"\n`);
    const env = {
        ...process.env,
        RUSTZEN_MONITOR_NOTIFY_DOCKER: docker,
        RUSTZEN_MONITOR_NOTIFY_SOURCE_IDENTITY: source,
        RUSTZEN_MONITOR_NOTIFY_VERIFIER_HELPER: verifier,
        RUSTZEN_MONITOR_NOTIFY_FILE: file,
        RUSTZEN_MONITOR_NOTIFY_EVIDENCE_ROOT: evidence,
        RUSTZEN_MONITOR_NOTIFY_TIMEOUT: options.timeout ?? "5",
        RUSTZEN_MONITOR_NOTIFY_CLEANUP_TIMEOUT: "1",
        RUSTZEN_MONITOR_NOTIFY_KILL_GRACE: "1",
        FAKE_EVIDENCE_ROOT: evidence,
        FAKE_STATE_ROOT: states,
        FAKE_CALL_LOG: calls,
        FAKE_TAMPER: options.tamper ?? "",
        FAKE_BLOCK: options.block ?? "",
        FAKE_RUNTIME_EXIT: options.runtimeExit ?? "",
        FAKE_RM_FAIL: options.rmFail ? "1" : "",
    };
    return {
        root, evidence, states, calls, env, outer,
        cleanup: () => rmSync(root, { recursive: true, force: true }),
        current: () => readlinkSync(join(evidence, "current")),
        active: () => readdirSync(states),
        failed: () => existsSync(join(evidence, "failed-runs")) ? readdirSync(join(evidence, "failed-runs")) : [],
        dockerCalls: () => existsSync(calls) ? readFileSync(calls, "utf8") : "",
    };
}

export function runFakeGate(fixture) {
    return Bun.spawnSync(["bash", fixture.outer], { env: fixture.env, stdout: "pipe", stderr: "pipe" });
}

export async function runSignaledFakeGate(fixture, signal) {
    const process = Bun.spawn(["bash", fixture.outer], {
        env: fixture.env,
        stdout: "pipe",
        stderr: "pipe",
    });
    for (let attempt = 0; attempt < 500 && fixture.active().length === 0; attempt += 1) await Bun.sleep(10);
    if (fixture.active().length === 0) throw new Error("fake Docker run did not start");
    process.kill(signal);
    await Bun.sleep(20);
    const active = fixture.active();
    if (active.length > 0) {
        const dockerPid = readFileSync(join(fixture.states, active[0]), "utf8").trim();
        Bun.spawnSync(["/bin/kill", "-TERM", dockerPid]);
    }
    const exitCode = await process.exited;
    return { exitCode, stdout: await new Response(process.stdout).text(), stderr: await new Response(process.stderr).text() };
}

if (process.argv[2] === "--fake-docker") await fakeDocker(process.argv.slice(3));
