import { generateKeyPairSync } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import {
    mkdir,
    readFile,
    rm,
    stat,
    symlink,
    writeFile,
} from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    manifestInputs,
    monitorSelection,
    serverManifestFixture,
    stagedPayloadFixture,
} from "./release-manifest-fixtures.ts";
import { produceReleaseManifest } from "./release-manifest.ts";
import {
    signReleaseEnvelope,
    verifyReleaseEnvelope,
} from "./release-envelope.ts";
import {
    publishSelectedRelease,
    setReleaseBeforePublishHookForTest,
    verifyReleaseDirectory,
} from "./release-publisher.ts";
import { setReleaseReadHookForTest } from "./release-publisher-files.ts";
import { setAtomicRenameBeforeCallHookForTest } from "./atomic-rename.ts";

const keys = generateKeyPairSync("ed25519");
const trusted = {
    keyId: "local-test-key",
    publicKey: keys.publicKey
        .export({ type: "spki", format: "pem" })
        .toString(),
};
const privateKey = keys.privateKey
    .export({ type: "pkcs8", format: "pem" })
    .toString();
async function input(kind: "server" | "node-agent") {
    if (kind === "server") {
        const fixture = await serverManifestFixture();
        return {
            fixture,
            value: {
                selection: monitorSelection,
                staging: fixture.staging,
                manifest: fixture.manifest,
                privateKey,
                trusted,
            },
        };
    }
    const fixture = await stagedPayloadFixture("agent");
    const selection = { preset: "node-agent", target: monitorSelection.target };
    return {
        fixture,
        value: {
            selection,
            staging: fixture.staging,
            manifest: await produceReleaseManifest({
                ...manifestInputs,
                selection,
                staging: fixture.staging,
            }),
            privateKey,
            trusted,
        },
    };
}
test("publisher atomically writes and verifies server and Agent release triplets", async () => {
    for (const kind of ["server", "node-agent"] as const) {
        const { fixture, value } = await input(kind);
        try {
            const published = await publishSelectedRelease(value);
            expect(
                await verifyReleaseDirectory(
                    published.root,
                    value.selection,
                    trusted,
                ),
            ).toEqual(published);
            await expect(publishSelectedRelease(value)).rejects.toThrow(
                "output",
            );
        } finally {
            await rm(fixture.root, { recursive: true, force: true });
        }
    }
});
test("publisher rejects existing paths, races and release artifact mutations", async () => {
    const { fixture, value } = await input("server");
    const parent = join(value.staging.root, "..");
    const final = join(parent, "release");
    try {
        for (const create of [
            async () => writeFile(final, "file"),
            async () => mkdir(final),
            async () => symlink("payload", final),
        ]) {
            await create();
            await expect(publishSelectedRelease(value)).rejects.toThrow(
                "output",
            );
            await rm(final, { recursive: true, force: true });
        }
        setReleaseBeforePublishHookForTest(async () => {
            await mkdir(final);
            await writeFile(join(final, "sentinel"), "keep");
        });
        await expect(publishSelectedRelease(value)).rejects.toThrow("output");
        expect(await Bun.file(join(final, "sentinel")).exists()).toBeTrue();
        await rm(final, { recursive: true, force: true });
        setReleaseBeforePublishHookForTest();
        const published = await publishSelectedRelease(value);
        for (const name of [
            "archive.tar",
            "release-manifest.json",
            "signature-envelope.json",
        ]) {
            await writeFile(join(published.root, name), "changed");
            await expect(
                verifyReleaseDirectory(
                    published.root,
                    value.selection,
                    trusted,
                ),
            ).rejects.toThrow();
            await rm(published.root, { recursive: true, force: true });
            await publishSelectedRelease(value);
        }
    } finally {
        setReleaseBeforePublishHookForTest();
        await rm(fixture.root, { recursive: true, force: true });
    }
});
test("atomic no-replace retains a final created at the native call boundary", async () => {
    const { fixture, value } = await input("server");
    const final = join(value.staging.root, "..", "release");
    try {
        setAtomicRenameBeforeCallHookForTest(() => {
            mkdirSync(final);
            writeFileSync(join(final, "sentinel"), "keep");
        });
        await expect(publishSelectedRelease(value)).rejects.toThrow("atomic");
        expect(await Bun.file(join(final, "sentinel")).text()).toBe("keep");
    } finally {
        setAtomicRenameBeforeCallHookForTest();
        await rm(fixture.root, { recursive: true, force: true });
    }
});
test("umask cannot weaken published release file modes", async () => {
    const inputs = await Promise.all([input("server"), input("node-agent")]);
    const previous = process.umask(0o077);
    try {
        for (const { fixture, value } of inputs) {
            try {
                const published = await publishSelectedRelease(value);
                for (const name of [
                    "archive.tar",
                    "release-manifest.json",
                    "signature-envelope.json",
                ])
                    expect(
                        (await stat(join(published.root, name))).mode & 0o777,
                    ).toBe(0o644);
                await verifyReleaseDirectory(
                    published.root,
                    value.selection,
                    trusted,
                );
            } finally {
                await rm(fixture.root, { recursive: true, force: true });
            }
        }
    } finally {
        process.umask(previous);
    }
});
test("release verification detects a file changed after its descriptor opens", async () => {
    const { fixture, value } = await input("server");
    try {
        const published = await publishSelectedRelease(value);
        setReleaseReadHookForTest(async (path) => writeFile(path, "changed"));
        await expect(
            verifyReleaseDirectory(published.root, value.selection, trusted),
        ).rejects.toThrow("changed while being read");
    } finally {
        setReleaseReadHookForTest();
        await rm(fixture.root, { recursive: true, force: true });
    }
});
test("verified release rejects every signed payload tuple mutation", async () => {
    const { fixture, value } = await input("server");
    try {
        const published = await publishSelectedRelease(value);
        const envelopePath = join(published.root, "signature-envelope.json");
        const original = await readFile(envelopePath);
        const payload = verifyReleaseEnvelope(original, trusted);
        const mutations = {
            keyId: "other-key",
            releaseClass: "test",
            releaseVersion: `${payload.releaseVersion}-x`,
            target: "other-target",
            artifactClass: "node-agent",
            compositionId: "0".repeat(64),
            buildId: "1".repeat(64),
            archiveSha256: "2".repeat(64),
            manifestSha256: "3".repeat(64),
            agentProtocolContractId: "4".repeat(64),
        };
        for (const [key, changed] of Object.entries(mutations)) {
            const signed = signReleaseEnvelope(
                { ...payload, [key]: changed },
                privateKey,
            );
            await writeFile(envelopePath, signed);
            await expect(
                verifyReleaseDirectory(
                    published.root,
                    value.selection,
                    trusted,
                ),
            ).rejects.toThrow();
            await writeFile(envelopePath, original);
        }
        for (const key of ["domain", "envelopeVersion", "algorithm"] as const)
            expect(() =>
                signReleaseEnvelope(
                    {
                        ...payload,
                        [key]: key === "envelopeVersion" ? 2 : "wrong",
                    } as never,
                    privateKey,
                ),
            ).toThrow();
        await expect(
            publishSelectedRelease({
                ...value,
                manifest: { ...value.manifest, releaseClass: "test" },
            }),
        ).rejects.toThrow();
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
