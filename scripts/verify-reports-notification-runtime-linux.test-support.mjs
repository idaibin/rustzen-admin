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
    "steps.log", "lifecycle.json", "outage-state.json", "retry-state.json", "scheduled-state.json",
    "revoked-state.json", "drop-event.json", "drop-proxy.json", "duplicate.json", "bad-signature.json",
    "unsigned.json", "public-internal.json", "selected-ingress-listener.txt", "selected-reports-config.json",
    "selected-reports-api.json", "inbox-final.json", "selected-state.json", "pure-admin-api.json",
    "reports-runtime-identity.json", "pure-admin-config.json", "pure-reports-config.json", "pure-reports-api.json", "pure-absence.json",
    "pure-notification-route.json", "pure-listeners.txt", "pure-web-binding.json", "pure-installation.json",
];
const compositionId = "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b";
const webDigest = "22a88f2af530ad9bb42b51b7b0b4432e187f87c365740e6162f563c43129f868";
const selectedApiDigest = "95f9a00978f1e4302d249ba06aad92e6ef72d0640f51e3642c65ec5bf124ffb3";

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
    const ids = Object.fromEntries(["success", "failed", "queued", "cooperative", "recoveryFailed",
        "recoveryCancelled", "drop", "retry", "revoked"].map((name, index) =>
        [name, `${index + 1}0000000-0000-4000-8000-000000000001`]));
    const json = (name, value) => writeFileSync(join(candidate, name), JSON.stringify(value));
    const lifecycle = {
        success: { id: ids.success, status: "succeeded" }, failed: { id: ids.failed, status: "failed" },
        queuedCancel: { id: ids.queued, status: "cancelled" },
        cooperativeCancel: { id: ids.cooperative, status: "cancelled" },
        recoveryFailed: { id: ids.recoveryFailed, status: "failed" },
        recoveryCancelled: { id: ids.recoveryCancelled, status: "cancelled" },
        dropResponse: { id: ids.drop, status: "succeeded" },
    };
    if (process.env.FAKE_TAMPER === "lifecycle-status") lifecycle.failed.status = "succeeded";
    json("lifecycle.json", lifecycle);
    json("outage-state.json", { runId: ids.success, status: "succeeded", pending: 1 });
    json("retry-state.json", { source: ids.failed, firstChild: ids.retry, secondChild: ids.retry, initiatorUserId: 1 });
    json("scheduled-state.json", { runId: "70000000-0000-4000-8000-000000000001", initiator: null, receiptCount: 7, gapCount: 0 });
    json("revoked-state.json", { runId: ids.revoked, eventId: "event-revoked", result: "no-recipients" });
    json("drop-event.json", { schemaVersion: 1, producer: "reports" });
    json("drop-proxy.json", { accepted: 2, firstResponseDropped: true });
    json("duplicate.json", { status: 200, contentType: "application/json", body: { code: "duplicate" } });
    json("bad-signature.json", { status: 401, contentType: "application/json", body: { code: "bad-producer" } });
    json("unsigned.json", { code: "invalid-protocol" });
    json("public-internal.json", { status: 404, contentType: "application/json", body: { code: 10001, message: "API route not found.", data: null } });
    writeFileSync(join(candidate, "steps.log"), "STEP manifest\n");
    writeFileSync(join(candidate, "selected-ingress-listener.txt"), "LISTEN 127.0.0.1:19831\n");
    json("selected-reports-config.json", { owner: "reports", fields: [{ name: "RUSTZEN_REPORTS_NOTIFICATION_INGRESS_URL" }] });
    json("selected-reports-api.json", { routes: [{ path: "/notification-delivery" }] });
    const messages = [
        ["reports.run.completed", ids.success], ["reports.run.failed", ids.failed],
        ["reports.run.cancelled", ids.queued], ["reports.run.cancelled", ids.cooperative],
        ["reports.run.failed", ids.recoveryFailed], ["reports.run.cancelled", ids.recoveryCancelled],
        ["reports.run.completed", ids.drop], ["reports.run.failed", ids.retry],
    ];
    json("selected-state.json", { receipts: [["no-recipients", 1], ["stored", 8]], messages,
        recipients: 8, outbox: 0, gaps: [0, 0, 0, 0, 0] });
    const reportIdentity = (prefix) => ({ process: { uid: 999, gid: 999 }, directories: [
        { kind: "runtime", path: `${prefix}/reports`, uid: 999, gid: 999, mode: "0750" },
        { kind: "db", path: `${prefix}/reports/db`, uid: 999, gid: 999, mode: "0750" },
        { kind: "log", path: `${prefix}/reports/logs/reports`, uid: 999, gid: 999, mode: "0750" },
        { kind: "artifact", path: `${prefix}/reports/data/reports`, uid: 999, gid: 999, mode: "0750" },
    ] });
    const runtimeIdentity = { user: { name: "rz-reports", uid: 999, gid: 999 },
        selected: reportIdentity("/tmp/rz-reports-notification-runtime/selected"),
        pure: reportIdentity("/tmp/rz-reports-notification-runtime/pure") };
    if (process.env.FAKE_TAMPER === "reports-root-process") runtimeIdentity.selected.process.uid = 0;
    if (process.env.FAKE_TAMPER === "reports-dir-owner") runtimeIdentity.pure.directories[2].uid = 0;
    json("reports-runtime-identity.json", runtimeIdentity);
    const items = messages.map(([topic, subjectId]) => ({ producer: "reports", topic,
        subjectKind: "reports-run", subjectId }));
    json("inbox-final.json", { data: { items } });
    json("pure-admin-api.json", { owner: "admin", routes: [] });
    json("pure-admin-config.json", { owner: "access", fields: [] });
    json("pure-reports-config.json", { owner: "reports", fields: [] });
    json("pure-reports-api.json", { routes: [{ path: "/runs" }] });
    const absence = { admin: [], reports: [] };
    if (process.env.FAKE_TAMPER === "pure-status") absence.reports.push("notification_outbox");
    json("pure-absence.json", absence);
    json("pure-notification-route.json", { code: 404, message: "Not found", data: null });
    writeFileSync(join(candidate, "pure-listeners.txt"), "LISTEN 127.0.0.1:19844\n");
    const pureBinding = { bindingVersion: 1, webDigest: values.RUSTZEN_VERIFY_PURE_WEB_DIGEST };
    const pureInstallation = { code: 0, data: { compositionId: values.RUSTZEN_VERIFY_PURE_COMPOSITION_ID,
        webDigest: values.RUSTZEN_VERIFY_PURE_WEB_DIGEST, featureIds: ["access", "monitor"] } };
    if (process.env.FAKE_TAMPER === "pure-binding") pureBinding.webDigest = "0".repeat(64);
    if (process.env.FAKE_TAMPER === "pure-installation") pureInstallation.data.featureIds = ["access"];
    json("pure-web-binding.json", pureBinding);
    json("pure-installation.json", pureInstallation);
    const receiptRows = receipts.map((file) => {
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
        reportsRuntime: { user: "rz-reports", nonRoot: true, directoryMode: "0750", selected: true, pure: true },
        delivery: { manualTerminalClasses: 6, retryInitiatorImmutable: true, scheduledSilent: true,
            revoked: "no-recipients", outageBackfilled: true, droppedResponse: "duplicate",
            authentication: { unsigned: 400, badSignature: 401, publicInternal: 404 } },
        pureReports: { notificationSchemaObjects: 0, notificationConfig: false,
            notificationRoute: false, notificationTask: false, notificationListeners: 0 },
        pureWeb: { compositionId: values.RUSTZEN_VERIFY_PURE_COMPOSITION_ID, webDigest: values.RUSTZEN_VERIFY_PURE_WEB_DIGEST },
        receipts: receiptRows,
    };
    if (process.env.FAKE_TAMPER === "duplicate-receipt") manifest.receipts.push(receiptRows[0]);
    if (process.env.FAKE_TAMPER === "wrong-status") manifest.delivery.droppedResponse = "stored";
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
    if (stage === "build" && process.env.FAKE_BUILD_EXIT) process.exit(Number(process.env.FAKE_BUILD_EXIT));
    if (stage === "build") {
        const outputMount = args.find((arg) => arg.includes("dst=/out"));
        const bin = outputMount.match(/src=([^,]+)/)[1];
        mkdirSync(bin, { recursive: true });
        for (const name of ["rz-admin-selected", "rz-reports-selected", "rz-admin-pure", "rz-reports-pure"]) {
            writeFileSync(join(bin, name), `fake-${name}`);
            chmodSync(join(bin, name), 0o755);
        }
        return;
    }
    if (process.env.FAKE_RUNTIME_EXIT) {
        const mount = args.find((arg) => arg.includes("dst=/verify/evidence"));
        const candidate = mount.match(/src=([^,]+)/)[1];
        writeFileSync(join(candidate, "steps.log"), "STEP selected-fresh-start\nSTEP admin-outage-and-backfill\n");
        process.exit(Number(process.env.FAKE_RUNTIME_EXIT));
    }
    writeRuntimeEvidence(args);
}

function executable(path, body) {
    writeFileSync(path, body);
    chmodSync(path, 0o755);
}

export function fakeGateFixture(outer, options = {}) {
    const root = mkdtempSync(join(tmpdir(), "rz-reports-notify-gate-"));
    const evidence = join(root, "evidence");
    const states = join(root, "states");
    const calls = join(root, "docker.calls");
    const selectedWeb = join(root, "selected-web", compositionId);
    mkdirSync(join(evidence, "runs/previous"), { recursive: true });
    mkdirSync(states);
    mkdirSync(selectedWeb, { recursive: true });
    writeFileSync(join(selectedWeb, "binding.json"), JSON.stringify({
        bindingVersion: 1,
        compositionId,
        selectedApiDigest,
        webDigest,
    }));
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
        RUSTZEN_REPORTS_NOTIFY_DOCKER: docker,
        RUSTZEN_REPORTS_NOTIFY_SOURCE_IDENTITY: source,
        RUSTZEN_REPORTS_NOTIFY_VERIFIER_HELPER: verifier,
        RUSTZEN_REPORTS_NOTIFY_FILE: file,
        RUSTZEN_REPORTS_NOTIFY_EVIDENCE_ROOT: evidence,
        RUSTZEN_REPORTS_NOTIFY_SELECTED_WEB_ROOT: join(root, "selected-web"),
        RUSTZEN_REPORTS_NOTIFY_BUILD_TIMEOUT: options.buildTimeout ?? "5",
        RUSTZEN_REPORTS_NOTIFY_RUNTIME_TIMEOUT: options.runtimeTimeout ?? "5",
        RUSTZEN_REPORTS_NOTIFY_CLEANUP_TIMEOUT: "1",
        RUSTZEN_REPORTS_NOTIFY_KILL_GRACE: "1",
        FAKE_EVIDENCE_ROOT: evidence,
        FAKE_STATE_ROOT: states,
        FAKE_CALL_LOG: calls,
        FAKE_TAMPER: options.tamper ?? "",
        FAKE_BLOCK: options.block ?? "",
        FAKE_BUILD_EXIT: options.buildExit ?? "",
        FAKE_RUNTIME_EXIT: options.runtimeExit ?? "",
        FAKE_RM_FAIL: options.rmFail ? "1" : "",
    };
    return {
        root, evidence, states, calls, env, outer,
        cleanup: () => rmSync(root, { recursive: true, force: true }),
        current: () => readlinkSync(join(evidence, "current")),
        active: () => readdirSync(states),
        failed: () => existsSync(join(evidence, "failed-runs")) ? readdirSync(join(evidence, "failed-runs")) : [],
        locked: () => existsSync(join(evidence, ".verify.lock")),
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
