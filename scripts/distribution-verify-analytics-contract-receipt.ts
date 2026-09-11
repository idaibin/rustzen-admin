import { writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";

import { canonicalJson, sha256 } from "../distribution/release-manifest-core.ts";
import { verifyAnalyticsContainerExport } from "../distribution/container-export-validator.ts";
import {
    analyticsContainerCommands,
    verifyAnalyticsContainerContractReceipt,
} from "../distribution/analytics-container-contract-receipt.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const args = parse(Bun.argv.slice(2));
const selectionPath = args.get("--selection");
const exportRoot = args.get("--export-root");
const expectedSourceIdentity = args.get("--expected-source-identity");
const stdoutDir = args.get("--stdout-dir");
const receiptOut = args.get("--receipt-out");
const receiptPath = args.get("--receipt");
const sourceIdentityAfter = args.get("--source-identity-after");
const verifierImageDigest = args.get("--verifier-image-digest");
const dockerContext = args.get("--docker-context");
const common = selectionPath && exportRoot && expectedSourceIdentity && stdoutDir;
const produce = common && receiptOut && sourceIdentityAfter && verifierImageDigest && dockerContext && !receiptPath && args.size === 8;
const verify = common && receiptPath && !receiptOut && !sourceIdentityAfter && !verifierImageDigest && !dockerContext && args.size === 5;
if (!produce && !verify)
    throw new Error(
        "usage: --selection <selection.json> --export-root <container-export-root> --expected-source-identity <source-identity> --stdout-dir <dir> " +
            "(--receipt-out <new-receipt.json> --source-identity-after <source-identity> --verifier-image-digest <sha256:digest> --docker-context <name> | --receipt <receipt.json>)",
    );
if (produce) {
    if (!/^sha256:[a-f0-9]{64}$/.test(verifierImageDigest)) throw new Error("verifier image digest must be a sha256:<64-hex> reference");
    if (!/^[A-Za-z0-9][A-Za-z0-9_.-]*$/.test(dockerContext)) throw new Error("docker context name is invalid");
}
const selection = await Bun.file(resolve(root, selectionPath!)).json();
if ((selection as { preset?: unknown }).preset !== "analytics")
    throw new Error("analytics container receipt is defined for the analytics selection");
const snapshot = await verifyAnalyticsContainerExport(
    resolve(root, exportRoot!),
    selection,
    expectedSourceIdentity!,
    await readWorkspaceVersion(root),
);
const stdoutNames = [
    "rz-admin-contract-selected-admin.stdout",
    "rz-insights-contract-selected.stdout",
    "rz-admin-contract-config-selected-access.stdout",
    "rz-insights-contract-config-selected.stdout",
    "rz-admin-contract-protocol.stdout",
    "rz-insights-contract-protocol.stdout",
];
const stdout = await readStdout(resolve(root, stdoutDir!));
const manifest = snapshot.manifest();
const provenance = snapshot.recordedProvenance();
const artifactPaths = analyticsContainerCommands.map((_, index) =>
    index < 2 ? "release/contracts/api/api.json" : index < 4 ? "release/contracts/config/config.json" : "release/contracts/protocol/protocol.json",
);
const target = resolve(root, produce ? receiptOut! : receiptPath!);
const receipt = produce
    ? {
        schemaVersion: 1,
        kind: "analytics-container-contract-receipt",
        context: { dockerContext: dockerContext!, platform: "linux/amd64" },
        commands: analyticsContainerCommands.map((command, index) => ({
            command: [...command],
            stdoutSha256: sha256(stdout[index]),
            artifactPath: artifactPaths[index],
            artifactSha256: sha256(snapshot.artifact(artifactPaths[index]).bytes),
        })),
        sourceIdentity: { before: provenance.sourceIdentityInput, after: sourceIdentityAfter! },
        manifestSha256: sha256(canonicalJson(manifest)),
        provenanceSha256: sha256(canonicalJson(provenance)),
        verifierImageDigest: verifierImageDigest!,
        restrictions: { network: "none", readOnlyRootfs: true, tmpfs: "noexec,nosuid,nodev", capDrop: "ALL", noNewPrivileges: true, pids: 32, memory: "256m" },
        runtime: false,
        certificate: false,
        signing: false,
        installer: false,
        browser: false,
        load: false,
    }
    : await readReceipt(target);
verifyAnalyticsContainerContractReceipt(snapshot, receipt, stdout);
if (produce) await writeFile(target, new TextEncoder().encode(canonicalJson(receipt)), { flag: "wx", mode: 0o644 });
console.log(canonicalJson({
    compositionId: manifest.compositionId,
    files: snapshot.paths().length,
    receipt: target,
    verified: true,
}));

async function readStdout(directory: string) {
    const output: string[] = [];
    for (const name of stdoutNames) {
        const file = Bun.file(join(directory, name));
        if (!(await file.exists())) throw new Error(`analytics container receipt stdout is missing: ${name}`);
        output.push(new TextDecoder("utf-8", { fatal: true }).decode(await file.bytes()));
    }
    return output;
}

async function readReceipt(path: string) {
    const bytes = await Bun.file(path).bytes();
    const value = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    if (new TextDecoder().decode(bytes) !== canonicalJson(value)) throw new Error("analytics container receipt file is not canonical");
    return value;
}

function parse(values: string[]) {
    const allowed = new Set(["--selection", "--export-root", "--expected-source-identity", "--stdout-dir", "--receipt-out", "--receipt", "--source-identity-after", "--verifier-image-digest", "--docker-context"]);
    if (values.length % 2 !== 0) return new Map<string, string>();
    const result = new Map<string, string>();
    for (let index = 0; index < values.length; index += 2) {
        const name = values[index], value = values[index + 1];
        if (!name || !value || !allowed.has(name) || result.has(name)) return new Map<string, string>();
        result.set(name, value);
    }
    return result;
}
