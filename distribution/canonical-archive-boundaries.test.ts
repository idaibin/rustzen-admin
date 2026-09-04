import { rm } from "node:fs/promises";
import { expect, test } from "bun:test";
import {
    createCanonicalArchive,
    readCanonicalArchive,
    splitUstarPath,
} from "./canonical-archive.ts";
import {
    monitorSelection,
    serverManifestFixture,
} from "./release-manifest-fixtures.ts";

type Span = { header: number; data: number; end: number };
async function fixture() {
    const value = await serverManifestFixture();
    const input = {
        selection: monitorSelection,
        staging: value.staging,
        manifest: value.manifest,
    };
    return { value, input, bytes: await createCanonicalArchive(input) };
}
test("reader rejects checksum-valid mode, duplicate, order, path and manifest mutations", async () => {
    const { value, bytes } = await fixture();
    try {
        const spans = spansOf(bytes);
        const manifest = spans.at(-1)!;
        const mutations: Array<(archive: Uint8Array) => void> = [
            (archive) => {
                archive.set(
                    new TextEncoder().encode("0000755\0"),
                    manifest.header + 100,
                );
                checksum(archive, manifest.header);
            },
            (archive) => {
                archive.set(archive.slice(0, 100), spans[1].header);
                archive.set(archive.slice(345, 500), spans[1].header + 345);
                checksum(archive, spans[1].header);
            },
            (archive) => {
                expect(spans[0].end - spans[0].header).toBe(
                    spans[1].end - spans[1].header,
                );
                const first = archive.slice(spans[0].header, spans[0].end);
                archive.copyWithin(
                    spans[0].header,
                    spans[1].header,
                    spans[1].end,
                );
                archive.set(first, spans[1].header);
            },
            (archive) => {
                archive[0] = 0x78;
                checksum(archive, 0);
            },
            (archive) => {
                archive[manifest.header + 156] = 0x78;
                checksum(archive, manifest.header);
            },
            (archive) => {
                archive[manifest.data] ^= 1;
            },
        ];
        for (const mutate of mutations) {
            const copy = bytes.slice();
            mutate(copy);
            expect(() =>
                readCanonicalArchive(copy, monitorSelection),
            ).toThrow();
        }
    } finally {
        await rm(value.root, { recursive: true, force: true });
    }
});
test("ustar uses UTF-8 byte limits for name and prefix", () => {
    expect(splitUstarPath("a".repeat(100)).name).toHaveLength(100);
    expect(splitUstarPath("p".repeat(155) + "/" + "名".repeat(33))).toEqual({
        prefix: new TextEncoder().encode("p".repeat(155)),
        name: new TextEncoder().encode("名".repeat(33)),
    });
    expect(() => splitUstarPath("a".repeat(101))).toThrow();
    expect(() => splitUstarPath("p".repeat(156) + "/a")).toThrow();
    expect(() =>
        splitUstarPath("p".repeat(155) + "/" + "名".repeat(34)),
    ).toThrow();
});
test("writer rejects every mutable staging reference field", async () => {
    const { value, input } = await fixture();
    try {
        for (const changed of [
            { buildId: "0".repeat(64) },
            { target: "wrong" },
            { artifactClass: "node-agent" },
            { root: `${input.staging.root}.wrong` },
            { files: [] },
            { sha256: "0".repeat(64) },
        ])
            await expect(
                createCanonicalArchive({
                    ...input,
                    staging: { ...input.staging, ...changed } as never,
                }),
            ).rejects.toThrow();
    } finally {
        await rm(value.root, { recursive: true, force: true });
    }
});
function spansOf(bytes: Uint8Array): Span[] {
    const result: Span[] = [];
    for (let offset = 0; bytes[offset] !== 0;) {
        const size = Number.parseInt(
            new TextDecoder().decode(bytes.slice(offset + 124, offset + 135)),
            8,
        );
        const data = offset + 512;
        const end = data + size + ((512 - (size % 512)) % 512);
        result.push({ header: offset, data, end });
        offset = end;
    }
    return result;
}
function checksum(bytes: Uint8Array, header: number) {
    bytes.fill(32, header + 148, header + 156);
    const total = bytes
        .slice(header, header + 512)
        .reduce((current, byte) => current + byte, 0)
        .toString(8)
        .padStart(6, "0");
    bytes.set(new TextEncoder().encode(`${total}\0 `), header + 148);
}
