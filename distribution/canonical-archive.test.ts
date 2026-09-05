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
            "d67d678ef09aadc520a36236c3e68312afe7c980d770b2845ee75f9f299dffe8",
        ],
        [
            "node-agent",
            "c272e756b84ac4777181d5e628af4e6387db5a2d59404350ba3e392fcc8f9950",
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
