import {
    chmod,
    mkdtemp,
    mkdir,
    rm,
    rename,
    symlink,
    writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "bun:test";
import {
    canonicalJson,
    canonicalManifestBytes,
    deriveBuildId,
    parseReleaseManifest,
    produceReleaseManifest,
    validateServerAgentPair,
} from "./release-manifest.ts";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactDirectoryHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";

const selection = { preset: "monitor", target: "x86_64-unknown-linux-musl" };
const h = (letter: string) => letter.repeat(64);
const inputs = {
    releaseVersion: "0.5.0",
    sourceIdentity: "git:abc",
    toolchain: "rustc-1.90",
    selectedRoutes: ["login", "monitor"],
    apiDigest: h("a"),
    schemaDigest: h("b"),
    configDigest: h("c"),
    protocolId: h("d"),
};
async function fixture(kind: "server" | "agent" = "server") {
    const root = await mkdtemp(join(tmpdir(), "rz-manifest-"));
    const artifactRoot = join(root, "artifact");
    const webRoot = join(root, "web");
    await mkdir(join(artifactRoot, "bin"), { recursive: true });
    await mkdir(join(artifactRoot, "config"), { recursive: true });
    await writeFile(join(artifactRoot, "config", "rz.env"), "PORT=3000\n");
    if (kind === "server") {
        await writeFile(join(artifactRoot, "bin", "rz-admin"), "admin");
        await writeFile(join(artifactRoot, "bin", "rz-monitor"), "monitor");
        await chmod(join(artifactRoot, "bin", "rz-admin"), 0o755);
        await chmod(join(artifactRoot, "bin", "rz-monitor"), 0o755);
        await mkdir(join(webRoot, "assets"), { recursive: true });
        await writeFile(join(webRoot, "index.html"), "<main>monitor</main>");
        await writeFile(join(webRoot, "assets", "main.js"), "monitor");
    } else {
        await writeFile(join(artifactRoot, "bin", "rz-monitor-agent"), "agent");
        await chmod(join(artifactRoot, "bin", "rz-monitor-agent"), 0o755);
    }
    return { root, artifactRoot, webRoot };
}
async function serverManifest() {
    const f = await fixture();
    const manifest = await produceReleaseManifest({
        ...inputs,
        selection,
        releaseVersion: "0.5.0",
        artifactRoot: f.artifactRoot,
        webRoot: f.webRoot,
        schemaFingerprints: { admin: h("e"), monitor: h("f") },
        dataContractIds: { admin: h("1"), monitor: h("2") },
    });
    return { ...f, manifest };
}

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
                expect(deriveBuildId(selection, changed)).not.toBe(
                    deriveBuildId(selection, inputs),
                );
            }
        } finally {
            await rm(root, { recursive: true, force: true });
        }
    });

    test("reads selected artifacts, excludes polluted binaries and follows no links", async () => {
        const { root, artifactRoot, webRoot, manifest } =
            await serverManifest();
        try {
            expect(
                (manifest as any).binaryDigests.map((x: any) => x.path),
            ).toEqual(["bin/rz-admin", "bin/rz-monitor"]);
            const before = (manifest as any).webDigest.sha256;
            await writeFile(join(webRoot, "assets", "main.js"), "changed");
            const changed = await produceReleaseManifest({
                ...inputs,
                selection,
                releaseVersion: "0.5.0",
                artifactRoot,
                webRoot,
                schemaFingerprints: { admin: h("e"), monitor: h("f") },
                dataContractIds: { admin: h("1"), monitor: h("2") },
            });
            expect((changed as any).webDigest.sha256).not.toBe(before);
            await writeFile(
                join(artifactRoot, "bin", "rz-reports"),
                "pollution",
            );
            await chmod(join(artifactRoot, "bin", "rz-reports"), 0o755);
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    artifactRoot,
                    webRoot,
                    schemaFingerprints: { admin: h("e"), monitor: h("f") },
                    dataContractIds: { admin: h("1"), monitor: h("2") },
                }),
            ).rejects.toThrow("binary inventory");
            await rm(join(artifactRoot, "bin", "rz-reports"));
            setArtifactReadHookForTest(async (path) => {
                if (path.endsWith("rz-monitor"))
                    await writeFile(path, "changed");
            });
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    artifactRoot,
                    webRoot,
                    schemaFingerprints: { admin: h("e"), monitor: h("f") },
                    dataContractIds: { admin: h("1"), monitor: h("2") },
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
                if (path === artifactRoot) {
                    await rename(
                        join(artifactRoot, "bin"),
                        join(artifactRoot, "bin-old"),
                    );
                    await symlink(external, join(artifactRoot, "bin"));
                }
            });
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    artifactRoot,
                    webRoot,
                    schemaFingerprints: { admin: h("e"), monitor: h("f") },
                    dataContractIds: { admin: h("1"), monitor: h("2") },
                }),
            ).rejects.toThrow("directory changed");
            expect(
                opened.some((path) => path.startsWith(external)),
            ).toBeFalse();
            setArtifactDirectoryHookForTest();
            setArtifactReadHookForTest();
            await rm(join(artifactRoot, "bin"));
            await rename(
                join(artifactRoot, "bin-old"),
                join(artifactRoot, "bin"),
            );
            await writeFile(join(artifactRoot, "bin", "rz-monitor"), "monitor");
            await chmod(join(artifactRoot, "bin", "rz-monitor"), 0o755);
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
                        join(artifactRoot, "bin"),
                        join(artifactRoot, "bin-open-old"),
                    );
                    await symlink(external, join(artifactRoot, "bin"));
                }
            });
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    artifactRoot,
                    webRoot,
                    schemaFingerprints: { admin: h("e"), monitor: h("f") },
                    dataContractIds: { admin: h("1"), monitor: h("2") },
                }),
            ).rejects.toThrow("directory changed");
            expect(afterOpenCount).toBe(0);
            setArtifactAfterOpenHookForTest();
            setArtifactReadHookForTest();
            await rm(join(artifactRoot, "bin"));
            await rename(
                join(artifactRoot, "bin-open-old"),
                join(artifactRoot, "bin"),
            );
            await rm(join(artifactRoot, "bin", "rz-monitor"));
            await symlink("rz-admin", join(artifactRoot, "bin", "rz-monitor"));
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    releaseVersion: "0.5.0",
                    artifactRoot,
                    webRoot,
                    schemaFingerprints: { admin: h("e"), monitor: h("f") },
                    dataContractIds: { admin: h("1"), monitor: h("2") },
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

    test("rejects malformed file/envelope paths and validates monitor server-Agent protocol", async () => {
        const { root, manifest } = await serverManifest();
        const agentFixture = await fixture("agent");
        try {
            const agent = await produceReleaseManifest({
                ...inputs,
                selection: { preset: "node-agent", target: selection.target },
                releaseVersion: "0.5.0",
                artifactRoot: agentFixture.artifactRoot,
            });
            validateServerAgentPair(manifest as any, agent as any);
            (agent as any).agentProtocolContractId = h("9");
            expect(() =>
                validateServerAgentPair(manifest as any, agent as any),
            ).toThrow("protocol");
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
            await rm(agentFixture.root, { recursive: true, force: true });
        }
    });
});

test("rejects invalid Unicode and binds releaseVersion into build identity", async () => {
    expect(() => canonicalJson("\ud800")).toThrow();
    expect(() => canonicalJson({ "\udc00": "ok" })).toThrow();
    expect(canonicalJson({ "😀": "ok" })).toContain("😀");
    expect(
        deriveBuildId(selection, { ...inputs, releaseVersion: "0.5.0" }),
    ).not.toBe(
        deriveBuildId(selection, { ...inputs, releaseVersion: "0.5.1" }),
    );
});
