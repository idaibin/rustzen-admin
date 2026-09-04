import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    canonicalJson,
    deriveBuildId,
    produceReleaseManifest,
} from "./release-manifest.ts";
import { completeSelectedApiContractForTest } from "./selected-contract.ts";
import { completeSelectedConfigForTest } from "./selected-config.ts";
import { produceNativeLayout } from "./native-layout.ts";
import {
    produceSchemaContract,
    readSchemaContract,
} from "./schema-contract.ts";

const target = "x86_64-unknown-linux-musl";
const selection = { preset: "monitor", target };
const h = (letter: string) => letter.repeat(64);
const inputs = {
    releaseVersion: "0.5.0",
    sourceIdentity: "git:abc",
    toolchain: "rustc-1.90",
    selectedRoutes: ["login", "monitor"],
    protocolId: h("d"),
};

test("server binds verified API bytes and Agent forbids API inputs", async () => {
    const digests = {
        apiDigest: h("a"),
        schemaDigest: h("b"),
        configDigest: h("c"),
        nativeLayoutDigest: h("e"),
    };
    expect(deriveBuildId(selection, inputs, digests)).not.toBe(
        deriveBuildId(selection, inputs, {
            ...digests,
            schemaDigest: h("d"),
        }),
    );
    expect(deriveBuildId(selection, inputs, digests)).not.toBe(
        deriveBuildId(selection, inputs, {
            ...digests,
            configDigest: h("d"),
        }),
    );
    expect(deriveBuildId(selection, inputs, digests)).not.toBe(
        deriveBuildId(selection, inputs, {
            ...digests,
            nativeLayoutDigest: h("f"),
        }),
    );
    expect(() =>
        deriveBuildId(selection, inputs, {
            configDigest: h("c"),
            nativeLayoutDigest: h("e"),
        }),
    ).toThrow("requires API and schema");
    expect(() =>
        deriveBuildId({ preset: "node-agent", target }, inputs, digests),
    ).toThrow("forbids server digests");
    const root = await mkdtemp(join(tmpdir(), "rz-manifest-api-"));
    const artifactRoot = join(root, "server");
    const webRoot = join(root, "web");
    const apiRoot = join(root, "api");
    const schemaRoot = join(root, "schema");
    const configRoot = join(root, "selected-config");
    const agentConfigRoot = join(root, "agent-config");
    const nativeRoot = join(root, "native");
    const agentNativeRoot = join(root, "agent-native");
    const agentRoot = join(root, "agent");
    try {
        await mkdir(join(artifactRoot, "bin"), { recursive: true });
        await mkdir(webRoot, { recursive: true });
        await mkdir(apiRoot, { recursive: true });
        await mkdir(configRoot, { recursive: true });
        await mkdir(agentConfigRoot, { recursive: true });
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
        await writeFile(
            join(configRoot, "config.json"),
            canonicalJson(completeSelectedConfigForTest(selection)),
        );
        await produceNativeLayout(selection, nativeRoot);
        await produceNativeLayout(
            { preset: "node-agent", target },
            agentNativeRoot,
        );
        await writeFile(
            join(agentConfigRoot, "config.json"),
            canonicalJson(
                completeSelectedConfigForTest({ preset: "node-agent", target }),
            ),
        );
        await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            schemaRoot,
        );

        const server = await produceReleaseManifest({
            ...inputs,
            selection,
            artifactRoot,
            webRoot,
            apiRoot,
            schemaRoot,
            configRoot,
            nativeRoot,
        });
        expect((server as any).apiDigest).toMatch(/^[0-9a-f]{64}$/);
        expect((server as any).configDigest).toMatch(/^[0-9a-f]{64}$/);
        expect((server as any).nativeLayoutDigest).toMatch(/^[0-9a-f]{64}$/);
        await writeFile(join(nativeRoot, "native-layout.json"), "{}");
        await expect(
            produceReleaseManifest({
                ...inputs,
                selection,
                artifactRoot,
                webRoot,
                apiRoot,
                schemaRoot,
                configRoot,
                nativeRoot,
            }),
        ).rejects.toThrow("selected generated source");
        await produceNativeLayout(selection, nativeRoot);
        const forgedSchema = structuredClone(
            (await readSchemaContract(schemaRoot, selection)).contract,
        );
        for (const owner of ["admin", "monitor"] as const) {
            forgedSchema.owners[owner].schemaSha256 = "0".repeat(64);
            forgedSchema.owners[owner].dataContractId = new Bun.CryptoHasher(
                "sha256",
            )
                .update(
                    canonicalJson({
                        owner,
                        version: 1,
                        schemaSha256: forgedSchema.owners[owner].schemaSha256,
                    }),
                )
                .digest("hex");
        }
        await writeFile(
            join(schemaRoot, "schema.json"),
            canonicalJson(forgedSchema),
        );
        await expect(
            produceReleaseManifest({
                ...inputs,
                selection,
                artifactRoot,
                webRoot,
                apiRoot,
                schemaRoot,
                configRoot,
                nativeRoot,
            }),
        ).rejects.toThrow("fresh-install SQL");
        await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            schemaRoot,
        );
        for (const supplied of [
            { apiDigest: h("a") },
            { schemaDigest: h("a") },
            { schemaFingerprints: { admin: h("a"), monitor: h("b") } },
            { dataContractIds: { admin: h("a"), monitor: h("b") } },
            { configDigest: h("a") },
            { nativeLayoutDigest: h("a") },
        ]) {
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    artifactRoot,
                    webRoot,
                    apiRoot,
                    schemaRoot,
                    configRoot,
                    nativeRoot,
                    ...supplied,
                } as any),
            ).rejects.toThrow("caller-supplied");
        }
        await writeFile(join(configRoot, "config.json"), "{}");
        await expect(
            produceReleaseManifest({
                ...inputs,
                selection,
                artifactRoot,
                webRoot,
                apiRoot,
                schemaRoot,
                configRoot,
                nativeRoot,
            }),
        ).rejects.toThrow("reviewed descriptors");
        await writeFile(
            join(configRoot, "config.json"),
            canonicalJson(completeSelectedConfigForTest(selection)),
        );

        await writeFile(join(apiRoot, "api.json"), "{}");
        await expect(
            produceReleaseManifest({
                ...inputs,
                selection,
                artifactRoot,
                webRoot,
                apiRoot,
                schemaRoot,
                configRoot,
                nativeRoot,
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
                schemaRoot,
                configRoot,
                nativeRoot,
            }),
        ).rejects.toThrow("exactly api.json");

        for (const apiInput of [{ apiRoot }, { schemaRoot }]) {
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection: { preset: "node-agent", target },
                    artifactRoot: agentRoot,
                    configRoot: agentConfigRoot,
                    nativeRoot: agentNativeRoot,
                    ...apiInput,
                } as any),
            ).rejects.toThrow("forbids");
        }
        const agent = await produceReleaseManifest({
            ...inputs,
            selection: { preset: "node-agent", target },
            artifactRoot: agentRoot,
            configRoot: agentConfigRoot,
            nativeRoot: agentNativeRoot,
        });
        expect("apiDigest" in agent).toBeFalse();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});
