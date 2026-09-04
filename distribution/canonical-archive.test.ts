import { rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    archiveSha256,
    createCanonicalArchive,
    readCanonicalArchive,
} from "./canonical-archive.ts";
import {
    manifestInputs,
    monitorSelection,
    serverManifestFixture,
    stagedPayloadFixture,
} from "./release-manifest-fixtures.ts";
import { produceReleaseManifest } from "./release-manifest.ts";

async function archiveFixture(kind: "server" | "node-agent" = "server") {
    if (kind === "server") {
        const fixture = await serverManifestFixture();
        const input = {
            selection: monitorSelection,
            staging: fixture.staging,
            manifest: fixture.manifest,
        };
        return { fixture, input, bytes: await createCanonicalArchive(input) };
    }
    const fixture = await stagedPayloadFixture("agent");
    const selection = { preset: "node-agent", target: monitorSelection.target };
    const input = {
        selection,
        staging: fixture.staging,
        manifest: await produceReleaseManifest({
            ...manifestInputs,
            selection,
            staging: fixture.staging,
        }),
    };
    return { fixture, input, bytes: await createCanonicalArchive(input) };
}
test("canonical ustar is deterministic and binds one staging snapshot", async () => {
    for (const [kind, digest] of [
        [
            "server",
            "49776e78167effd9b73eef77f1eccf83cbf220dea3b84f3685fdda656f7efe73",
        ],
        [
            "node-agent",
            "2833c1a10cc11599b83a6a38c10f217b6865f31f8fac1e9c61d62c15427569d3",
        ],
    ] as const) {
        const { fixture, input, bytes } = await archiveFixture(kind);
        try {
            expect(await createCanonicalArchive(input)).toEqual(bytes);
            const parsed = readCanonicalArchive(bytes, input.selection);
            expect(parsed.manifest).toEqual(input.manifest);
            expect(parsed.root).toBe(`rz-${kind}-${input.manifest.buildId}`);
            expect(parsed.manifestSha256).toMatch(/^[a-f0-9]{64}$/);
            expect(archiveSha256(bytes)).toBe(digest);
        } finally {
            await rm(fixture.root, { recursive: true, force: true });
        }
    }
});
test("archive reader rejects header, payload, padding and trailing mutations", async () => {
    const { fixture, bytes } = await archiveFixture();
    try {
        for (const mutate of [
            (value: Uint8Array) => {
                value[0] ^= 1;
            },
            (value: Uint8Array) => {
                value[513] ^= 1;
            },
            (value: Uint8Array) => {
                value[520] = 1;
            },
            (value: Uint8Array) => {
                value[value.length - 1] = 1;
            },
            (value: Uint8Array) => {
                value[136] = 0x31;
                checksum(value);
            },
        ]) {
            const copy = bytes.slice();
            mutate(copy);
            expect(() =>
                readCanonicalArchive(copy, monitorSelection),
            ).toThrow();
        }
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
function checksum(value: Uint8Array) {
    value.fill(32, 148, 156);
    const total = value
        .slice(0, 512)
        .reduce((current, byte) => current + byte, 0)
        .toString(8)
        .padStart(6, "0");
    value.set(new TextEncoder().encode(`${total}\0 `), 148);
}
test("archive writer rejects stale staging", async () => {
    const { fixture, input } = await archiveFixture();
    try {
        await writeFile(join(fixture.payloadRoot, "bin/rz-admin"), "changed");
        await expect(createCanonicalArchive(input)).rejects.toThrow(
            "inventory",
        );
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
