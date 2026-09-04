import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { canonicalJson, produceReleaseManifest } from "./release-manifest.ts";
import { completeSelectedApiContractForTest } from "./selected-contract.ts";

const target = "x86_64-unknown-linux-musl";
const selection = { preset: "monitor", target };
const h = (letter: string) => letter.repeat(64);
const inputs = {
    releaseVersion: "0.5.0",
    sourceIdentity: "git:abc",
    toolchain: "rustc-1.90",
    selectedRoutes: ["login", "monitor"],
    schemaDigest: h("b"),
    configDigest: h("c"),
    protocolId: h("d"),
};

test("server binds verified API bytes and Agent forbids API inputs", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-manifest-api-"));
    const artifactRoot = join(root, "server");
    const webRoot = join(root, "web");
    const apiRoot = join(root, "api");
    const agentRoot = join(root, "agent");
    try {
        await mkdir(join(artifactRoot, "bin"), { recursive: true });
        await mkdir(webRoot, { recursive: true });
        await mkdir(apiRoot, { recursive: true });
        await mkdir(join(agentRoot, "bin"), { recursive: true });
        for (const binary of ["rz-admin", "rz-monitor"]) {
            await writeFile(join(artifactRoot, "bin", binary), binary);
            await chmod(join(artifactRoot, "bin", binary), 0o755);
        }
        await writeFile(join(webRoot, "index.html"), "monitor");
        await writeFile(join(agentRoot, "bin", "rz-monitor-agent"), "agent");
        await chmod(join(agentRoot, "bin", "rz-monitor-agent"), 0o755);
        const validApi = canonicalJson(
            completeSelectedApiContractForTest(selection),
        );
        await writeFile(join(apiRoot, "api.json"), validApi);

        const server = await produceReleaseManifest({
            ...inputs,
            selection,
            artifactRoot,
            webRoot,
            apiRoot,
            schemaFingerprints: { admin: h("e"), monitor: h("f") },
            dataContractIds: { admin: h("1"), monitor: h("2") },
        });
        expect((server as any).apiDigest).toMatch(/^[0-9a-f]{64}$/);

        await writeFile(join(apiRoot, "api.json"), "{}");
        await expect(
            produceReleaseManifest({
                ...inputs,
                selection,
                artifactRoot,
                webRoot,
                apiRoot,
                schemaFingerprints: { admin: h("e"), monitor: h("f") },
                dataContractIds: { admin: h("1"), monitor: h("2") },
            }),
        ).rejects.toThrow();
        await writeFile(join(apiRoot, "api.json"), validApi);
        await writeFile(join(apiRoot, "extra.json"), "{}");
        await expect(
            produceReleaseManifest({
                ...inputs,
                selection,
                artifactRoot,
                webRoot,
                apiRoot,
                schemaFingerprints: { admin: h("e"), monitor: h("f") },
                dataContractIds: { admin: h("1"), monitor: h("2") },
            }),
        ).rejects.toThrow("exactly api.json");

        for (const apiInput of [{ apiRoot }, { apiDigest: h("a") }]) {
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection: { preset: "node-agent", target },
                    artifactRoot: agentRoot,
                    ...apiInput,
                } as any),
            ).rejects.toThrow("forbids");
        }
        const agent = await produceReleaseManifest({
            ...inputs,
            selection: { preset: "node-agent", target },
            artifactRoot: agentRoot,
        });
        expect("apiDigest" in agent).toBeFalse();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});
