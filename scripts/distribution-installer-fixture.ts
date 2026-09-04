import { generateKeyPairSync } from "node:crypto";
import { chmod, mkdir, rm, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { createCanonicalArchive } from "../distribution/canonical-archive.ts";
import { releaseEnvelopePayload, signReleaseEnvelope } from "../distribution/release-envelope.ts";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { agentManifestFixture, serverManifestFixture } from "../distribution/release-manifest-fixtures.ts";
import { publishSelectedRelease } from "../distribution/release-publisher.ts";

const repository = resolve(import.meta.dir, "..");
const output = resolve(repository, process.env.RUSTZEN_INSTALLER_OUTPUT ?? "target/installer-fixture");
const keyId = process.env.RUSTZEN_INSTALLER_KEY_ID ?? "installer-test";
const target = process.env.RUSTZEN_INSTALLER_TARGET ?? "x86_64-unknown-linux-musl";
const mutation = process.env.RUSTZEN_INSTALLER_MUTATION;
const manifestMutation = process.env.RUSTZEN_INSTALLER_MANIFEST_MUTATION;
const envelopeMutation = process.env.RUSTZEN_INSTALLER_ENVELOPE_MUTATION;
const artifact = process.env.RUSTZEN_INSTALLER_ARTIFACT ?? "server";
const agentBinary = process.env.RUSTZEN_INSTALLER_AGENT_BINARY;
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true, mode: 0o700 });
const selection = artifact === "node-agent" ? { preset: "node-agent", target } : { preset: "monitor", target };
const fixture = artifact === "node-agent"
    ? await agentManifestFixture(selection, agentBinary)
    : await serverManifestFixture(selection);
const keys = generateKeyPairSync("ed25519");
try {
    const privateKey = keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
    const publicKey = keys.publicKey.export({ type: "spki", format: "pem" }).toString();
    const publication = await publishSelectedRelease({
        selection,
        staging: fixture.staging,
        manifest: fixture.manifest,
        privateKey,
        trusted: { keyId, publicKey },
    });
    let archive = new Uint8Array(await Bun.file(join(publication.root, "archive.tar")).arrayBuffer());
    if (mutation) archive = mutate(archive, mutation);
    const originalManifest = new TextEncoder().encode(canonicalJson(fixture.manifest));
    const manifest = structuredClone(fixture.manifest) as any;
    if (manifestMutation) mutateManifest(manifest, manifestMutation);
    const manifestBytes = new TextEncoder().encode(canonicalJson(manifest));
    if (manifestMutation) archive = replace(archive, originalManifest, manifestBytes);
    const payload = releaseEnvelopePayload(manifest, keyId, archive, manifestBytes);
    if (envelopeMutation) mutateEnvelope(payload as any, envelopeMutation);
    const envelope = signReleaseEnvelope(payload, privateKey);
    await Bun.write(join(output, "archive.tar"), archive);
    await Bun.write(join(output, "release-manifest.json"), manifestBytes);
    await Bun.write(join(output, "signature-envelope.json"), envelope);
    await writeFile(join(output, "public.pem"), publicKey, { mode: 0o600 });
    await chmod(join(output, "public.pem"), 0o600);
} finally {
    await rm(fixture.root, { recursive: true, force: true });
}
function mutate(archive: Uint8Array, kind: string): Uint8Array {
    const end = archive.length - 1024;
    const firstSize = Number.parseInt(new TextDecoder().decode(archive.slice(124, 135)), 8);
    const first = archive.slice(0, 512 + firstSize + ((512 - (firstSize % 512)) % 512));
    if (kind === "duplicate") return concat([archive.slice(0, end), first, archive.slice(end)]);
    if (kind === "omitted") return concat([archive.slice(first.length, end), archive.slice(end)]);
    if (kind === "path") { const changed = archive.slice(); changed[0] = 47; return changed; }
    if (kind === "header") { const changed = archive.slice(); changed[157] = 1; return changed; }
    if (kind === "checksum-width") { const changed = archive.slice(); changed[148] = 0; return changed; }
    if (kind === "padding") { const changed = archive.slice(); changed[512 + firstSize] = 1; return changed; }
    if (kind === "trailing") return concat([archive, new Uint8Array([1])]);
    throw new Error(`unknown installer mutation: ${kind}`);
}
function mutateEnvelope(payload: any, kind: string) {
    const h = "f".repeat(64);
    if (kind === "build-id") payload.buildId = h;
    else if (kind === "archive-hash") payload.archiveSha256 = h;
    else if (kind === "manifest-hash") payload.manifestSha256 = h;
    else if (kind === "key-id") payload.keyId = "other-key";
    else throw new Error(`unknown installer envelope mutation: ${kind}`);
}
function mutateManifest(manifest: any, kind: string) {
    const h = "f".repeat(64);
    if (kind === "capabilities") manifest.capabilities[manifest.capabilities.length - 1] = "accessx";
    else if (kind === "config-owners") manifest.configOwners[manifest.configOwners.length - 1] = "accessx";
    else if (kind === "binary-digest") manifest.binaryDigests[0].sha256 = h;
    else if (kind === "api-digest") manifest.apiDigest = h;
    else if (kind === "schema-owner") manifest.schemaFingerprints.monitor = h;
    else if (kind === "data-owner") manifest.dataContractIds.monitor = h;
    else if (kind === "web-digest") manifest.webDigest.sha256 = h;
    else if (kind === "native-digest") manifest.nativeLayoutDigest = h;
    else if (kind === "protocol-digest") manifest.protocolArtifactDigest = h;
    else if (kind === "selection-digest") manifest.selectionDigest.sha256 = h;
    else if (kind === "agent-capability") manifest.capabilities[0] = "monitor-bgent";
    else throw new Error(`unknown installer manifest mutation: ${kind}`);
}
function replace(value: Uint8Array, from: Uint8Array, to: Uint8Array): Uint8Array {
    if (from.length !== to.length) throw new Error("manifest mutation must preserve archive member size");
    const at = value.findIndex((_, offset) => from.every((byte, index) => value[offset + index] === byte));
    if (at < 0) throw new Error("embedded manifest not found");
    const output = value.slice(); output.set(to, at); return output;
}
function concat(parts: Uint8Array[]): Uint8Array {
    const output = new Uint8Array(parts.reduce((size, part) => size + part.length, 0));
    let offset = 0;
    for (const part of parts) { output.set(part, offset); offset += part.length; }
    return output;
}
