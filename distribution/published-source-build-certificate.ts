import { constants, type BigIntStats } from "node:fs";
import { lstat, open, realpath } from "node:fs/promises";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    parseSourceBuildCertificate,
    sourceBuildCertificateBytes,
} from "./source-build-certificate.ts";
import {
    readIssuedSourceBuildCertificate,
    type IssuedSourceBuildCertificate,
} from "./source-build-issuer.ts";

const certificateName = "source-build-manifest.json";
const maxBytes = 64 * 1024;

export type PublishedSourceBuildCertificate = {
    certificateSha256: string;
    selection: { preset: "monitor" | "monitor-notify" | "analytics"; target: string; artifactClass: "server" | "node-agent"; compositionId: string };
    buildId: string;
    binaryDigests: Array<{ path: string; sha256: string }>;
    manifestSha256: string;
    archiveSha256: string;
    envelopeSha256: string;
};

/** Reads the one issued certificate through its stable, canonical file boundary. */
export async function verifyPublishedSourceBuildCertificate(input: {
    issued: IssuedSourceBuildCertificate;
    certificatePath: string;
}): Promise<PublishedSourceBuildCertificate> {
    if (canonicalJson(Object.keys(input).sort()) !== '["certificatePath","issued"]')
        throw new Error("published certificate verification input fields are invalid");
    const issued = readIssuedSourceBuildCertificate(input.issued);
    const expected = join(dirname(resolve(issued.releaseRoot)), "source-build-certificate", certificateName);
    const path = resolve(input.certificatePath);
    if (path !== expected)
        throw new Error("published certificate path is not canonical");
    const trustedRoot = await realpath(resolve(issued.trustedRoot));
    const certificateRoot = await checkedDirectory(dirname(path));
    const parent = await checkedDirectory(dirname(certificateRoot.path));
    assertContained(trustedRoot, certificateRoot.real); assertContained(trustedRoot, parent.real);
    const before = checked(await lstat(path, { bigint: true }));
    const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const opened = checked(await handle.stat({ bigint: true }));
        if (!same(before, opened)) throw new Error("published certificate changed while being opened");
        const bytes = await readBounded(handle);
        const after = checked(await handle.stat({ bigint: true }));
        const pathAfter = checked(await lstat(path, { bigint: true }));
        const certificateRootAfter = await checkedDirectory(certificateRoot.path);
        const parentAfter = await checkedDirectory(parent.path);
        if (!same(before, after) || !same(before, pathAfter) || !sameDirectory(certificateRoot, certificateRootAfter) || !sameDirectory(parent, parentAfter))
            throw new Error("published certificate changed while being read");
        const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
        const certificate = parseSourceBuildCertificate(JSON.parse(text), issued.selection);
        const canonical = sourceBuildCertificateBytes(certificate, issued.selection);
        if (text !== new TextDecoder().decode(canonical) || canonicalJson(certificate) !== canonicalJson(issued.certificate))
            throw new Error("published certificate differs from issued evidence");
        return {
            certificateSha256: sha256(bytes),
            selection: {
                preset: certificate.selection.preset === "monitor" || certificate.selection.preset === "monitor-notify" || certificate.selection.preset === "analytics" ? certificate.selection.preset : (() => { throw new Error("published certificate preset is unsupported"); })(),
                target: certificate.selection.target,
                artifactClass: certificate.selection.artifactClass,
                compositionId: certificate.selection.compositionId,
            },
            buildId: certificate.certifiedLayers.build.buildId,
            binaryDigests: certificate.certifiedLayers.build.binaryDigests,
            manifestSha256: certificate.certifiedLayers.artifact.manifestSha256,
            archiveSha256: certificate.certifiedLayers.artifact.archiveSha256,
            envelopeSha256: certificate.certifiedLayers.artifact.envelopeSha256,
        };
    } finally { await handle.close(); }
}

type Identity = Pick<BigIntStats, "dev" | "ino" | "mode" | "size" | "mtimeNs" | "ctimeNs">;
function checked(stat: BigIntStats): Identity {
    if (!stat.isFile() || stat.isSymbolicLink() || (stat.mode & 0o777n) !== 0o644n)
        throw new Error("published certificate must be a mode 0644 regular non-link file");
    if (stat.size > BigInt(maxBytes)) throw new Error("published certificate exceeds size limit");
    return stat;
}
function same(left: Identity, right: Identity) {
    return left.dev === right.dev && left.ino === right.ino && left.mode === right.mode &&
        left.size === right.size && left.mtimeNs === right.mtimeNs && left.ctimeNs === right.ctimeNs;
}
async function checkedDirectory(path: string) {
    const stat = await lstat(path, { bigint: true }); const real = await realpath(path);
    if (!stat.isDirectory() || stat.isSymbolicLink() || stat.uid !== BigInt(process.getuid()) || (stat.mode & 0o077n) !== 0n) throw new Error("published certificate parent is unsafe");
    return { path, real, stat };
}
function sameDirectory(left: Awaited<ReturnType<typeof checkedDirectory>>, right: Awaited<ReturnType<typeof checkedDirectory>>) { return same(left.stat, right.stat) && left.real === right.real; }
async function readBounded(handle: Awaited<ReturnType<typeof open>>) {
    const bytes = Buffer.allocUnsafe(maxBytes + 1);
    let offset = 0;
    while (offset < bytes.length) {
        const read = await handle.read(bytes, offset, bytes.length - offset, null);
        if (!read.bytesRead) return bytes.subarray(0, offset);
        offset += read.bytesRead;
    }
    throw new Error("published certificate exceeds size limit");
}
function assertContained(root: string, path: string) {
    const value = relative(root, path);
    if (value === ".." || value.startsWith(`..${sep}`) || isAbsolute(value))
        throw new Error("published certificate escaped trusted root");
}
