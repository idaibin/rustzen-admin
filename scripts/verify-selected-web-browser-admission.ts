import { constants } from "node:fs";
import { lstat, open } from "node:fs/promises";
import { relative, resolve, sep } from "node:path";
import { verifierProvenance } from "./selected-web-browser-provenance.ts";

const root = resolve(import.meta.dir, "..");
const required = ["--selection", "--export-root", "--release-result", "--certificate", "--public-key", "--expected-source-identity", "--admin-bin"];
const values = new Map<string, string>();
for (let i = 2; i < Bun.argv.length; i += 2) {
    const key = Bun.argv[i], value = Bun.argv[i + 1];
    if (!key || !value || !required.includes(key) || values.has(key)) throw new Error("invalid browser admission arguments");
    values.set(key, value);
}
if (values.size !== required.length) throw new Error("missing browser admission arguments");
const inside = (path: string) => path === root || !relative(root, path).startsWith(`..${sep}`);
const file = async (value: string, label: string) => {
    const path = resolve(value); if (!inside(path)) throw new Error(`${label} escaped repository root`); const before = await lstat(path, { bigint: true });
    if (!before.isFile() || before.isSymbolicLink()) throw new Error(`${label} must be a regular non-link file`);
    const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const opened = await handle.stat({ bigint: true });
        if (opened.dev !== before.dev || opened.ino !== before.ino || opened.size !== before.size) throw new Error(`${label} changed while opened`);
        const bytes = await handle.readFile();
        const after = await handle.stat({ bigint: true });
        if (after.dev !== before.dev || after.ino !== before.ino || after.size !== before.size) throw new Error(`${label} changed while read`);
        return { path, bytes };
    } finally { await handle.close(); }
};
const [selection, release, certificate, publicKey, admin] = await Promise.all([
    file(values.get("--selection")!, "selection"), file(values.get("--release-result")!, "release result"),
    file(values.get("--certificate")!, "certificate"), file(values.get("--public-key")!, "public key"),
    file(values.get("--admin-bin")!, "admin binary"),
]);
const selected = JSON.parse(new TextDecoder().decode(selection.bytes));
if (selected?.schemaVersion !== 1 || !["monitor", "monitor-notify"].includes(selected.preset) || selected.target !== "x86_64-unknown-linux-musl" || Object.keys(selected).sort().join(",") !== "preset,schemaVersion,target")
    throw new Error("browser selection is unsupported");
const result = JSON.parse(new TextDecoder().decode(release.bytes));
const releaseRoot = resolve(result.root);
if (!inside(releaseRoot) || releaseRoot === root) throw new Error("release root escaped repository root");
const exportRoot = resolve(values.get("--export-root")!); const exportStat = await lstat(exportRoot);
if (!inside(exportRoot) || !exportStat.isDirectory() || exportStat.isSymbolicLink()) throw new Error("export root is unsafe");
const rootStat = await lstat(releaseRoot); if (!rootStat.isDirectory() || rootStat.isSymbolicLink()) throw new Error("release root is unsafe");
const [manifest, envelope] = await Promise.all([file(`${releaseRoot}/release-manifest.json`, "release manifest"), file(`${releaseRoot}/signature-envelope.json`, "release envelope")]);
const manifestJson = JSON.parse(new TextDecoder().decode(manifest.bytes));
const webDigest = manifestJson.webDigest?.sha256;
if (!/^[a-f0-9]{64}$/.test(webDigest || "")) throw new Error("release manifest has no web digest");
const keyId = JSON.parse(new TextDecoder().decode(envelope.bytes)).payload?.keyId;
if (typeof keyId !== "string") throw new Error("release envelope has no key id");
const source = Bun.spawnSync(["scripts/admin-browser-source-identity.sh"], { cwd: root, stdout: "pipe", stderr: "pipe" });
if (source.exitCode !== 0) throw new Error("cannot derive current source identity");
const sourceFields = /^([0-9a-f]{40})\t(clean|dirty)\t([0-9a-f]{64})\n?$/.exec(new TextDecoder().decode(source.stdout));
if (!sourceFields) throw new Error("current source identity output is invalid");
const currentSourceIdentity = `git:${sourceFields[1]} tree:${sourceFields[3]} state:${sourceFields[2]}`;
const command = ["pnpm", "dlx", "bun@1.3.14", "scripts/distribution-verify-published-source-build-certificate.ts", "--selection", selection.path, "--export-root", values.get("--export-root")!, "--expected-source-identity", values.get("--expected-source-identity")!, "--release-root", releaseRoot, "--public-key", publicKey.path, "--key-id", keyId, "--certificate", certificate.path];
const run = Bun.spawnSync(command, { cwd: root, stdout: "pipe", stderr: "pipe" });
if (run.exitCode !== 0) throw new Error(new TextDecoder().decode(run.stderr));
const admission = JSON.parse(new TextDecoder().decode(run.stdout));
if (admission.selection?.preset !== selected.preset || admission.selection?.target !== selected.target || admission.selection?.artifactClass !== "server")
    throw new Error("verified release selection differs from browser selection");
const adminSha256 = new Bun.CryptoHasher("sha256").update(admin.bytes).digest("hex");
if (admission.binaryDigests?.find((row: any) => row.path === "bin/rz-admin")?.sha256 !== adminSha256) throw new Error("admin binary digest differs from verified release");
const provenance = await verifierProvenance(root);
console.log(JSON.stringify({ admission, webDigest, adminSha256, productSourceIdentity: values.get("--expected-source-identity"), verifierSourceIdentity: currentSourceIdentity, provenance }));
