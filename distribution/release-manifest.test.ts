import { chmod, mkdir, rm, rename, symlink, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { describe, expect, test } from "bun:test";
import {
    canonicalJson,
    canonicalManifestBytes,
    deriveBuildId,
    parseReleaseManifest,
    produceReleaseManifest,
} from "./release-manifest.ts";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactDirectoryHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";
import {
    h,
    manifestContractDigests as contractDigests,
    manifestInputs as inputs,
    monitorSelection as selection,
    serverManifestFixture as serverManifest,
} from "./release-manifest-fixtures.ts";

describe("release manifest producer and validator", () => {
    test("uses UTF-16/JCS ordering and canonical parsed bytes", async () => {
        expect(canonicalJson({ "\uE000": 1, "😀": 2 })).toBe('{"😀":2,"":1}');
        const { root, manifest } = await serverManifest();
        try {
            const parsed = parseReleaseManifest(manifest, selection);
            expect(
                new TextDecoder().decode(
                    canonicalManifestBytes(parsed, selection),
                ),
            ).toBe(canonicalJson(parsed));
            expect(parsed.artifactClass).toBe("server");
        } finally {
            await rm(root, { recursive: true, force: true });
        }
    });

    test("binds every resolved plan field and every build-plan identity", async () => {
        const { root, manifest } = await serverManifest();
        try {
            const fields: ((m: any) => void)[] = [
                (m) => (m.preset = "full"),
                (m) => (m.artifactClass = "node-agent"),
                (m) => (m.releaseClass = "test"),
                (m) => (m.target = "bad"),
                (m) => (m.capabilities = ["access"]),
                (m) => (m.services = ["admin"]),
                (m) => (m.compositionId = h("0")),
                (m) => (m.selectionDigest.sha256 = h("0")),
                (m) => {
                    m.schemaFingerprints.insights = m.schemaFingerprints.admin;
                    delete m.schemaFingerprints.admin;
                },
                (m) => {
                    m.dataContractIds.insights = m.dataContractIds.admin;
                    delete m.dataContractIds.admin;
                },
                (m) => (m.configOwners = ["admin"]),
            ];
            for (const mutate of fields) {
                const copy = structuredClone(manifest);
                mutate(copy);
                expect(() => parseReleaseManifest(copy, selection)).toThrow();
            }
            for (const key of Object.keys(inputs) as (keyof typeof inputs)[]) {
                const changed = {
                    ...inputs,
                    [key]:
                        key === "selectedRoutes"
                            ? ["login"]
                            : key === "sourceIdentity" ||
                                key === "toolchain" ||
                                key === "releaseVersion"
                              ? "changed"
                              : h("9"),
                };
                expect(
                    deriveBuildId(selection, changed, contractDigests),
                ).not.toBe(deriveBuildId(selection, inputs, contractDigests));
            }
        } finally {
            await rm(root, { recursive: true, force: true });
        }
    });

    test("reads selected artifacts, excludes polluted binaries and follows no links", async () => {
        const {
            root,
            artifactRoot,
            webRoot,
            apiRoot,
            schemaRoot,
            configRoot,
            nativeRoot,
            protocolRoot,
            payloadRoot,
            staging,
            manifest,
        } = await serverManifest();
        try {
            expect(
                (manifest as any).binaryDigests.map((x: any) => x.path),
            ).toEqual(["bin/rz-admin", "bin/rz-monitor"]);
            const before = (manifest as any).webDigest.sha256;
            await writeFile(
                join(payloadRoot, "web", "assets", "main.js"),
                "changed",
            );
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    staging,
                }),
            ).rejects.toThrow("inventory");
            await writeFile(
                join(payloadRoot, "bin", "rz-reports"),
                "pollution",
            );
            await chmod(join(payloadRoot, "bin", "rz-reports"), 0o755);
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    staging,
                }),
            ).rejects.toThrow("inventory");
            await rm(join(payloadRoot, "bin", "rz-reports"));
            setArtifactReadHookForTest(async (path) => {
                if (path.endsWith("rz-monitor"))
                    await writeFile(path, "changed");
            });
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    staging,
                }),
            ).rejects.toThrow("changed");
            setArtifactReadHookForTest();
            const external = join(root, "external-bin");
            await mkdir(external);
            await writeFile(join(external, "rz-admin"), "SENTINEL");
            const opened: string[] = [];
            setArtifactReadHookForTest((path) => {
                opened.push(path);
            });
            setArtifactDirectoryHookForTest(async (path) => {
                if (path === payloadRoot) {
                    await rename(
                        join(payloadRoot, "bin"),
                        join(payloadRoot, "bin-old"),
                    );
                    await symlink(external, join(payloadRoot, "bin"));
                }
            });
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    staging,
                }),
            ).rejects.toThrow("directory changed");
            expect(
                opened.some((path) => path.startsWith(external)),
            ).toBeFalse();
            setArtifactDirectoryHookForTest();
            setArtifactReadHookForTest();
            await rm(join(payloadRoot, "bin"));
            await rename(
                join(payloadRoot, "bin-old"),
                join(payloadRoot, "bin"),
            );
            await writeFile(join(payloadRoot, "bin", "rz-monitor"), "monitor");
            await chmod(join(payloadRoot, "bin", "rz-monitor"), 0o755);
            let afterOpenCount = 0;
            setArtifactAfterOpenHookForTest((path) => {
                if (path.endsWith("rz-admin")) {
                    afterOpenCount++;
                    throw new Error("AFTER_OPEN_SENTINEL");
                }
            });
            setArtifactReadHookForTest(async (path) => {
                if (path.endsWith("rz-admin")) {
                    await rename(
                        join(payloadRoot, "bin"),
                        join(payloadRoot, "bin-open-old"),
                    );
                    await symlink(external, join(payloadRoot, "bin"));
                }
            });
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    staging,
                }),
            ).rejects.toThrow("directory changed");
            expect(afterOpenCount).toBe(0);
            setArtifactAfterOpenHookForTest();
            setArtifactReadHookForTest();
            await rm(join(payloadRoot, "bin"));
            await rename(
                join(payloadRoot, "bin-open-old"),
                join(payloadRoot, "bin"),
            );
            await rm(join(payloadRoot, "bin", "rz-monitor"));
            await symlink("rz-admin", join(payloadRoot, "bin", "rz-monitor"));
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    staging,
                }),
            ).rejects.toThrow("symlink");
        } finally {
            setArtifactAfterOpenHookForTest();
            setArtifactAfterOpenHookForTest();
            setArtifactReadHookForTest();
            setArtifactDirectoryHookForTest();
            await rm(root, { recursive: true, force: true });
        }
    });

    test("rejects malformed file/envelope paths", async () => {
        const { root, manifest } = await serverManifest();
        try {
            for (const path of [
                "../bad",
                "bin\\bad",
                "manifest.json",
                "envelope.json",
                "a/../b",
                "a\0b",
            ]) {
                const copy = structuredClone(manifest);
                copy.files[0].path = path;
                expect(() => parseReleaseManifest(copy, selection)).toThrow();
            }
            const duplicate = structuredClone(manifest);
            duplicate.binaryDigests.push({ ...duplicate.binaryDigests[0] });
            expect(() => parseReleaseManifest(duplicate, selection)).toThrow();
        } finally {
            await rm(root, { recursive: true, force: true });
        }
    });
});

test("rejects invalid Unicode and binds releaseVersion into build identity", async () => {
    expect(() => canonicalJson("\ud800")).toThrow();
    expect(() => canonicalJson({ "\udc00": "ok" })).toThrow();
    expect(canonicalJson({ "😀": "ok" })).toContain("😀");
    expect(
        deriveBuildId(
            selection,
            { ...inputs, releaseVersion: "0.5.0" },
            contractDigests,
        ),
    ).not.toBe(
        deriveBuildId(
            selection,
            { ...inputs, releaseVersion: "0.5.1" },
            contractDigests,
        ),
    );
});
