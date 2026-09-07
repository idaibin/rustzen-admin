import {
    appendFileSync, chmodSync, existsSync, mkdirSync, mkdtempSync, readFileSync,
    readdirSync, readlinkSync, rmSync, symlinkSync, unlinkSync, writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const innerReceipts = [
    "steps.log", "sid-a-after-logout.json", "sid-b-after-logout.json", "revoke-all.json",
    "password-old.json", "password-new.json", "disable-old.json", "disable-login.json",
    "grant-before.json", "grant-removed.json", "grant-restored.json", "bad-jwt.json",
    "valid-before-expiry.json", "expired-jwt.json", "authority-db-failure.json", "selected-state.json",
];
export const receipts = [...innerReceipts, "phase-status.tsv", "build.log", "verifier-helper.log", "runtime.log"];

const hash = (bytes) => new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
const executable = (path, body) => { writeFileSync(path, body); chmodSync(path, 0o755); };
function envValues(args) {
    const values = {};
    for (let i = 0; i < args.length; i += 1) if (args[i] === "--env") {
        const [key, ...value] = args[++i].split("="); values[key] = value.join("=");
    }
    return values;
}

function runtimeEvidence(args) {
    const mount = args.find((arg) => arg.includes("dst=/verify/evidence"));
    const candidate = mount.match(/src=([^,]+)/)[1];
    const values = envValues(args);
    const json = (name, value) => writeFileSync(join(candidate, name), JSON.stringify(value));
    const envelope = (status, code, data = null) => ({
        status, contentType: "application/json", body: { code, message: code === 0 ? "Success" : "denied", data },
    });
    writeFileSync(join(candidate, "steps.log"), "STEP manifest\n");
    const cases = {
        "sid-a-after-logout.json": envelope(401, 401),
        "sid-b-after-logout.json": envelope(200, 0, { id: 4 }),
        "revoke-all.json": envelope(401, 401),
        "password-old.json": envelope(401, 401),
        "password-new.json": envelope(200, 0, { id: 6 }),
        "disable-old.json": envelope(401, 401),
        "disable-login.json": envelope(403, 10004),
        "grant-before.json": envelope(200, 0, []),
        "grant-removed.json": envelope(403, 403),
        "grant-restored.json": envelope(200, 0, []),
        "bad-jwt.json": envelope(401, 401),
        "valid-before-expiry.json": envelope(200, 0, { id: 8 }),
        "expired-jwt.json": envelope(401, 401),
        "authority-db-failure.json": envelope(401, 401),
    };
    if (process.env.FAKE_TAMPER === "status") cases["grant-removed.json"] = envelope(200, 0, []);
    if (process.env.FAKE_TAMPER === "expiry") cases["expired-jwt.json"] = envelope(200, 0, { id: 8 });
    for (const [name, value] of Object.entries(cases)) json(name, value);
    const selected = {
        sessions: 8, active: 4, authzEpoch: 26,
        grantEpochs: { before: 20, removed: 23, restored: 26 },
        users: [
            ["runtime_disable", 2, 2], ["runtime_grant", 1, 1], ["runtime_password", 1, 2],
            ["runtime_revoke", 1, 2], ["runtime_sid", 1, 1],
        ],
        grantCodes: [["system:user:list"], ["system:user:options"]],
        expiryIdentity: {
            claims: { sid: "11111111-1111-4111-8111-111111111111", userId: 8, username: "runtime_grant", userAuthEpoch: 1, exp: 2_000_000_000 },
            expiredClaims: { sid: "11111111-1111-4111-8111-111111111111", userId: 8, username: "runtime_grant", userAuthEpoch: 1, exp: 1_999_999_999 },
            session: ["11111111-1111-4111-8111-111111111111", 8, 1, 2_000_000_000, null],
        },
    };
    if (process.env.FAKE_TAMPER === "state") selected.grantCodes = [["system:user:options"]];
    if (process.env.FAKE_TAMPER === "remove-epoch") selected.grantEpochs.removed = 20;
    if (process.env.FAKE_TAMPER === "restore-epoch") selected.grantEpochs.restored = 23;
    if (process.env.FAKE_TAMPER === "user-epoch") selected.users[4][2] = 2;
    if (process.env.FAKE_TAMPER === "expiry-session") selected.expiryIdentity.session[4] = 1_999_999_998;
    json("selected-state.json", selected);
    const receiptRows = innerReceipts.map((file) => {
        const bytes = readFileSync(join(candidate, file));
        return { file, sha256: hash(bytes), bytes: bytes.byteLength };
    });
    const manifest = {
        schemaVersion: 1, status: "passed", gitHead: values.RUSTZEN_VERIFY_HEAD,
        sourceTreeState: values.RUSTZEN_VERIFY_SOURCE_TREE_STATE,
        sourceTreeSha256: values.RUSTZEN_VERIFY_SOURCE_TREE_SHA256,
        platform: { architecture: values.RUSTZEN_VERIFY_ARCHITECTURE, name: values.RUSTZEN_VERIFY_PLATFORM },
        binarySha256: values.RUSTZEN_VERIFY_BINARY_SHA256,
        buildProvenanceSha256: values.RUSTZEN_VERIFY_BUILD_PROVENANCE_SHA256,
        verifier: { imageId: values.RUSTZEN_VERIFY_VERIFIER_IMAGE_ID, key: values.RUSTZEN_VERIFY_VERIFIER_KEY, provenanceSha256: values.RUSTZEN_VERIFY_VERIFIER_PROVENANCE_SHA256 },
        cases: { twoSessions: true, logoutIsolation: true, revokeAll: true, passwordChange: true, userDisable: true, grantRemoved: true, grantRestored: true, badJwt: true, expiredJwt: true, authorityStorageFailClosed: true },
        receipts: receiptRows,
    };
    if (process.env.FAKE_TAMPER === "duplicate") manifest.receipts.push(receiptRows[0]);
    if (process.env.FAKE_TAMPER === "path") manifest.receipts[0].file = "../steps.log";
    json("manifest.json", manifest);
}

export async function fakeDocker(args) {
    process.on("SIGINT", () => process.exit(130));
    process.on("SIGTERM", () => process.exit(143));
    mkdirSync(process.env.FAKE_STATE_ROOT, { recursive: true });
    appendFileSync(process.env.FAKE_CALL_LOG, `${args.join(" ")}\n`);
    if (args[0] === "info") return console.log("aarch64");
    if (args[0] === "logs") {
        if (process.env.FAKE_BLOCK === "logs") await Bun.sleep(30_000);
        return console.log("fake runtime log");
    }
    if (args[0] === "rm") {
        if (process.env.FAKE_BLOCK === "rm") await Bun.sleep(30_000);
        if (process.env.FAKE_RM_FAIL === "1") process.exit(1);
        const state = join(process.env.FAKE_STATE_ROOT, args.at(-1));
        if (existsSync(state)) unlinkSync(state);
        return;
    }
    if (args[0] === "container" && args[1] === "inspect") {
        process.exit(existsSync(join(process.env.FAKE_STATE_ROOT, args.at(-1))) ? 0 : 1);
    }
    if (args[0] !== "run") process.exit(2);
    const name = args[args.indexOf("--name") + 1];
    writeFileSync(join(process.env.FAKE_STATE_ROOT, name), String(process.pid));
    const build = args.includes("rust:1.95-bookworm");
    if (process.env.FAKE_BLOCK === (build ? "build" : "runtime")) await Bun.sleep(30_000);
    if (build) {
        if (process.env.FAKE_BUILD_EXIT) process.exit(Number(process.env.FAKE_BUILD_EXIT));
        const bin = join(process.env.FAKE_EVIDENCE_ROOT, "build/aarch64/bin");
        mkdirSync(bin, { recursive: true }); writeFileSync(join(bin, "rz-admin"), "fake-admin");
        chmodSync(join(bin, "rz-admin"), 0o755); return;
    }
    if (process.env.FAKE_RUNTIME_EXIT) process.exit(Number(process.env.FAKE_RUNTIME_EXIT));
    runtimeEvidence(args);
}

export function fixture(outer, options = {}) {
    const root = mkdtempSync(join(tmpdir(), "rz-admin-session-gate-"));
    const evidence = join(root, "evidence"), states = join(root, "states"), calls = join(root, "calls");
    mkdirSync(join(evidence, "runs/previous"), { recursive: true }); mkdirSync(states);
    writeFileSync(join(evidence, "runs/previous/manifest.json"), '{"status":"previous"}\n');
    symlinkSync("runs/previous", join(evidence, "current"));
    const source = join(root, "source.sh"), verifier = join(root, "verifier.sh");
    const file = join(root, "file.sh"), docker = join(root, "docker.sh");
    executable(source, "#!/bin/sh\nprintf 'fakehead\\tclean\\tfakesource\\n'\n");
    executable(verifier, `#!/bin/sh
set -eu
probe="$FAKE_STATE_ROOT/verifier-probe"
child=
cleanup() { [ -z "$child" ] || kill "$child" 2>/dev/null || true; rm -f "$probe"; }
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
: >"$probe"
echo "helper logical seconds $FAKE_HELPER_LOGICAL_SECONDS" >>"$FAKE_CALL_LOG"
echo 'fake verifier helper started' >&2
if [ -n "$FAKE_HELPER_DELAY" ]; then sleep "$FAKE_HELPER_DELAY" & child=$!; wait "$child"; child=; fi
if [ "$FAKE_HELPER_FAIL" = 1 ]; then echo 'fake verifier helper failure' >&2; exit 9; fi
printf 'sha256:fake\tfakekey\tfakeprovenance\n'
`);
    executable(file, "#!/bin/sh\necho 'ELF 64-bit LSB executable, ARM aarch64'\n");
    executable(docker, `#!/bin/sh\nexec '${process.execPath}' '${import.meta.filename}' --fake-docker "$@"\n`);
    const env = {
        ...process.env, RUSTZEN_ADMIN_SESSION_DOCKER: docker,
        RUSTZEN_ADMIN_SESSION_SOURCE_IDENTITY: source,
        RUSTZEN_ADMIN_SESSION_VERIFIER_HELPER: verifier, RUSTZEN_ADMIN_SESSION_FILE: file,
        RUSTZEN_ADMIN_SESSION_EVIDENCE_ROOT: evidence,
        RUSTZEN_ADMIN_SESSION_TIMEOUT: options.timeout ?? "5",
        RUSTZEN_ADMIN_SESSION_CLEANUP_TIMEOUT: "1", RUSTZEN_ADMIN_SESSION_KILL_GRACE: "1",
        RUSTZEN_ADMIN_SESSION_HELPER_TIMEOUT: options.helperTimeout ?? "5",
        RUSTZEN_ADMIN_SESSION_HELPER_CLEANUP_GRACE: "2",
        FAKE_EVIDENCE_ROOT: evidence, FAKE_STATE_ROOT: states, FAKE_CALL_LOG: calls,
        FAKE_TAMPER: options.tamper ?? "", FAKE_BLOCK: options.block ?? "",
        FAKE_RUNTIME_EXIT: options.runtimeExit ?? "", FAKE_RM_FAIL: options.rmFail ? "1" : "",
        FAKE_BUILD_EXIT: options.buildExit ?? "", FAKE_HELPER_FAIL: options.helperFail ? "1" : "",
        FAKE_HELPER_DELAY: options.helperDelay ?? "", FAKE_HELPER_LOGICAL_SECONDS: options.helperLogicalSeconds ?? "0",
    };
    return {
        root, evidence, states, calls, env, outer,
        cleanup: () => rmSync(root, { recursive: true, force: true }),
        current: () => readlinkSync(join(evidence, "current")),
        active: () => readdirSync(states),
        failed: () => existsSync(join(evidence, "failed-runs")) ? readdirSync(join(evidence, "failed-runs")) : [],
        callsText: () => existsSync(calls) ? readFileSync(calls, "utf8") : "",
    };
}

export const run = (item) => Bun.spawnSync(["bash", item.outer], { env: item.env, stdout: "pipe", stderr: "pipe" });
export async function signal(item, name, stateName) {
    const child = Bun.spawn(["bash", item.outer], { env: item.env, stdout: "pipe", stderr: "pipe" });
    const ready = () => stateName ? item.active().includes(stateName) : item.active().length > 0;
    for (let i = 0; i < 500 && !ready(); i += 1) await Bun.sleep(10);
    if (!ready()) throw new Error(`fake resource never started: ${stateName ?? "Docker"}`);
    child.kill(name); return await child.exited;
}

if (process.argv[2] === "--fake-docker") await fakeDocker(process.argv.slice(3));
