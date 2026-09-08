import { expect, test } from "bun:test";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { h, monitorSelection, serverManifestFixture } from "./release-manifest-fixtures.ts";
import {
    parseSourceBuildCertificate,
    produceSourceBuildCertificate,
    sourceBuildCertificateBytes,
    verifySourceBuildCertificate,
} from "./source-build-certificate.ts";
import { rm } from "node:fs/promises";

test("monitor source-build certificate binds exact manifest and cannot claim later layers", async () => {
    const fixture = await serverManifestFixture();
    try {
        const input = {
            selection: monitorSelection,
            manifest: fixture.manifest,
            sourceTreeSha256: h("a"),
            toolchain: "rustc 1.90",
            archiveSha256: h("b"),
            envelopeSha256: h("c"),
        };
        const value = produceSourceBuildCertificate(input);
        expect(new TextDecoder().decode(sourceBuildCertificateBytes(value, monitorSelection))).toBe(
            canonicalJson(value),
        );
        expect(value.certifiedLayers.artifact.manifestSha256).toBe(
            sha256(canonicalJson(fixture.manifest)),
        );
        expect(verifySourceBuildCertificate(value, input)).toEqual(value);
        expect(() =>
            parseSourceBuildCertificate({ ...value, runtime: true }, monitorSelection),
        ).toThrow("cannot claim");
        expect(() =>
            parseSourceBuildCertificate(
                { ...value, certifiedLayers: { ...value.certifiedLayers, deployment: {} } },
                monitorSelection,
            ),
        ).toThrow("unknown");
        expect(() =>
            parseSourceBuildCertificate(
                { ...value, selection: { ...value.selection, capabilities: ["access"] } },
                monitorSelection,
            ),
        ).toThrow("selection differs");
        expect(() =>
            parseSourceBuildCertificate(
                { ...value, selection: { ...value.selection, selectionDigest: h("d") } },
                monitorSelection,
            ),
        ).toThrow("selection differs");
        expect(() =>
            produceSourceBuildCertificate({
                selection: { preset: "monitor-notify", target: monitorSelection.target },
                manifest: fixture.manifest,
                sourceTreeSha256: h("a"),
                toolchain: "rustc",
                archiveSha256: h("b"),
                envelopeSha256: h("c"),
            }),
        ).toThrow("only monitor");
        for (const changed of [
            {
                ...value,
                certifiedLayers: {
                    ...value.certifiedLayers,
                    source: { identity: "other", treeSha256: h("d") },
                },
            },
            {
                ...value,
                certifiedLayers: {
                    ...value.certifiedLayers,
                    build: { ...value.certifiedLayers.build, toolchain: "other", buildId: h("d") },
                },
            },
            {
                ...value,
                certifiedLayers: {
                    ...value.certifiedLayers,
                    build: {
                        ...value.certifiedLayers.build,
                        binaryDigests: value.certifiedLayers.build.binaryDigests.map((binary) => ({
                            ...binary,
                            sha256: h("d"),
                        })),
                    },
                },
            },
            {
                ...value,
                certifiedLayers: {
                    ...value.certifiedLayers,
                    artifact: {
                        manifestSha256: h("d"),
                        archiveSha256: h("e"),
                        envelopeSha256: h("f"),
                    },
                },
            },
        ])
            expect(() => verifySourceBuildCertificate(changed, input)).toThrow(
                "authoritative inputs",
            );
    } finally {
        await rm(fixture.root, { recursive: true, force: true });
    }
});
