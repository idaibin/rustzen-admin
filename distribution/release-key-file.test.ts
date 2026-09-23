import { chmod, mkdir, mkdtemp, rename, rm, symlink, writeFile } from "node:fs/promises";
import { generateKeyPairSync } from "node:crypto";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    readReleaseKeyFile,
    setReleaseKeyAfterOpenHookForTest,
    validateReleaseSigningKeyPair,
} from "./release-key-file.ts";

test("release key reader enforces no-follow ownership-safe modes and bounded input", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-key-file-"));
    try {
        const privatePath = join(root, "private.pem");
        const publicPath = join(root, "public.pem");
        await writeFile(privatePath, "private", { mode: 0o600 });
        await writeFile(publicPath, "public", { mode: 0o644 });
        expect(await readReleaseKeyFile(privatePath, "private")).toBe("private");
        expect(await readReleaseKeyFile(publicPath, "public")).toBe("public");
        await chmod(privatePath, 0o644);
        await expect(readReleaseKeyFile(privatePath, "private")).rejects.toThrow("mode");
        await chmod(privatePath, 0o600);
        await symlink(privatePath, join(root, "link.pem"));
        await expect(readReleaseKeyFile(join(root, "link.pem"), "private")).rejects.toThrow("non-link");
        await writeFile(privatePath, "x".repeat(16 * 1024 + 1));
        await expect(readReleaseKeyFile(privatePath, "private")).rejects.toThrow("size");
        await mkdir(join(root, "directory.pem"), { mode: 0o700 });
        await expect(readReleaseKeyFile(join(root, "directory.pem"), "private")).rejects.toThrow("regular");
    } finally { await rm(root, { recursive: true, force: true }); }
});

test("release signing validation accepts only one matching Ed25519 SPKI public PEM", () => {
    const keys = generateKeyPairSync("ed25519");
    const other = generateKeyPairSync("ed25519");
    const privateKey = keys.privateKey.export({ type: "pkcs8", format: "pem" }).toString();
    const publicKey = keys.publicKey.export({ type: "spki", format: "pem" }).toString();
    expect(() => validateReleaseSigningKeyPair(privateKey, publicKey)).not.toThrow();
    expect(() => validateReleaseSigningKeyPair(privateKey, privateKey)).toThrow("SPKI");
    expect(() => validateReleaseSigningKeyPair(privateKey, `${publicKey}${publicKey}`)).toThrow("SPKI");
    expect(() => validateReleaseSigningKeyPair(
        privateKey,
        other.publicKey.export({ type: "spki", format: "pem" }).toString(),
    )).toThrow("match");
});

test("release key reader rejects replacement and growth during descriptor reads", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-key-race-"));
    const path = join(root, "private.pem");
    try {
        await writeFile(path, "private", { mode: 0o600 });
        setReleaseKeyAfterOpenHookForTest(async () => {
            const replacement = join(root, "replacement.pem");
            await writeFile(replacement, "replacement", { mode: 0o600 });
            await rename(replacement, path);
        });
        await expect(readReleaseKeyFile(path, "private")).rejects.toThrow("changed");
        setReleaseKeyAfterOpenHookForTest(async () => {
            await writeFile(path, "x".repeat(16 * 1024 + 1));
        });
        await expect(readReleaseKeyFile(path, "private")).rejects.toThrow("size");
    } finally {
        setReleaseKeyAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});
