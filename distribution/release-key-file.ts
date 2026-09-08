import { constants, type BigIntStats } from "node:fs";
import { createPrivateKey, createPublicKey, sign, verify } from "node:crypto";
import { lstat, open } from "node:fs/promises";

const MAX_PEM_BYTES = 16 * 1024;
let afterOpenHook: (() => Promise<void> | void) | undefined;

/** Test-only stable-read seam. */
export const setReleaseKeyAfterOpenHookForTest = (
    hook?: () => Promise<void> | void,
) => { afterOpenHook = hook; };

type Kind = "private" | "public";
type Identity = {
    dev: bigint;
    ino: bigint;
    mode: bigint;
    size: bigint;
    mtimeNs: bigint;
    ctimeNs: bigint;
};

/** Reads a local PEM only after validating its no-follow, stable file boundary. */
export async function readReleaseKeyFile(path: string, kind: Kind): Promise<string> {
    const before = checked(await lstat(path, { bigint: true }), kind);
    const handle = await open(path, constants.O_RDONLY | constants.O_NOFOLLOW);
    try {
        const opened = checked(await handle.stat({ bigint: true }), kind);
        if (!same(before, opened)) throw new Error("release key file changed while being opened");
        await afterOpenHook?.();
        const bytes = await boundedRead(handle);
        const after = checked(await handle.stat({ bigint: true }), kind);
        const pathAfter = checked(await lstat(path, { bigint: true }), kind);
        if (!same(before, after) || !same(before, pathAfter))
            throw new Error("release key file changed while being read");
        return new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    } finally {
        await handle.close();
    }
}

/** Rejects invalid IDs, non-Ed25519 keys and mismatched public/private PEMs. */
export function validateReleaseSigningKeyPair(
    privateKey: string,
    publicKey: string,
): void {
    const privateObject = createPrivateKey(privateKey);
    if (!singleSpkiPublicKeyPem(publicKey))
        throw new Error("release public key must be one SPKI PUBLIC KEY PEM block");
    const publicObject = createPublicKey(publicKey);
    if (privateObject.asymmetricKeyType !== "ed25519" || publicObject.asymmetricKeyType !== "ed25519")
        throw new Error("release signing keys must be Ed25519");
    const probe = new TextEncoder().encode("rustzen-release-key-pair-v1");
    if (!verify(null, probe, publicObject, sign(null, probe, privateObject)))
        throw new Error("release signing private and public keys do not match");
    const derived = createPublicKey(privateKey).export({ type: "spki", format: "der" });
    const supplied = publicObject.export({ type: "spki", format: "der" });
    if (!Buffer.from(derived).equals(Buffer.from(supplied)))
        throw new Error("release signing private and public keys do not match");
}

function singleSpkiPublicKeyPem(value: string): boolean {
    return /^-----BEGIN PUBLIC KEY-----\r?\n(?:[A-Za-z0-9+/=]{1,64}\r?\n)+-----END PUBLIC KEY-----\r?\n?$/.test(value);
}

async function boundedRead(handle: Awaited<ReturnType<typeof open>>): Promise<Uint8Array> {
    const bytes = Buffer.allocUnsafe(MAX_PEM_BYTES + 1);
    let offset = 0;
    while (offset < bytes.length) {
        const read = await handle.read(bytes, offset, bytes.length - offset, null);
        if (!read.bytesRead) return bytes.subarray(0, offset);
        offset += read.bytesRead;
    }
    throw new Error("release key exceeds size limit");
}

function checked(stat: BigIntStats, kind: Kind): Identity {
    if (!stat.isFile() || stat.isSymbolicLink())
        throw new Error("release key must be a regular non-link file");
    if (stat.uid !== BigInt(process.getuid()))
        throw new Error("release key must be owned by the current user");
    const mode = stat.mode & 0o777n;
    const expected = kind === "private" ? [0o600n] : [0o600n, 0o644n];
    if (!expected.includes(mode)) throw new Error("release key file mode is invalid");
    if (stat.size > BigInt(MAX_PEM_BYTES)) throw new Error("release key exceeds size limit");
    return {
        dev: stat.dev,
        ino: stat.ino,
        mode,
        size: stat.size,
        mtimeNs: stat.mtimeNs,
        ctimeNs: stat.ctimeNs,
    };
}
function same(left: Identity, right: Identity) {
    return left.dev === right.dev && left.ino === right.ino && left.mode === right.mode &&
        left.size === right.size && left.mtimeNs === right.mtimeNs && left.ctimeNs === right.ctimeNs;
}
