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
import { produceNativeStaging } from "./native-staging.ts";
import {
    produceSelectedProtocol,
    reviewedProtocolOutput,
} from "./selected-protocol.ts";
import {
    produceSchemaContract,
    readSchemaContract,
} from "./schema-contract.ts";
import {
    writeFixtureWebBinding,
    writeFixtureWebPolicyFiles,
} from "./release-manifest-fixtures.ts";
import { WEB_BINDING_SLOT } from "./selected-web-binding.ts";
import { resolveSelection } from "./resolver.ts";
import { selectedWebRoutes } from "../scripts/distribution-web-inventory-policy.ts";

const target = "x86_64-unknown-linux-musl";
const selection = { preset: "monitor", target };
const h = (letter: string) => letter.repeat(64);
const inputs = {
    releaseVersion: "0.5.0",
    sourceIdentity: "git:abc",
    toolchain: "rustc-1.90",
    selectedRoutes: selectedWebRoutes(resolveSelection(selection)),
};

test("server binds verified API bytes and Agent forbids API inputs", async () => {
    const digests = {
        apiDigest: h("a"),
        schemaDigest: h("b"),
        configDigest: h("c"),
        nativeLayoutDigest: h("e"),
        protocolArtifactDigest: h("f"),
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
    expect(deriveBuildId(selection, inputs, digests)).not.toBe(
        deriveBuildId(selection, inputs, {
            ...digests,
            protocolArtifactDigest: h("0"),
        }),
    );
    expect(() =>
        deriveBuildId(selection, inputs, {
            configDigest: h("c"),
            nativeLayoutDigest: h("e"),
            protocolArtifactDigest: h("f"),
        }),
    ).toThrow("requires API and schema");
    expect(() =>
        deriveBuildId({ preset: "node-agent", target }, inputs, digests),
    ).toThrow("forbids server digests");
    const root = await mkdtemp(join(tmpdir(), "rz-manifest-api-"));
    const artifactRoot = join(root, "server");
    const webRoot = join(root, "web", "dist");
    const apiRoot = join(root, "api");
    const schemaRoot = join(root, "schema");
    const configRoot = join(root, "selected-config");
    const agentConfigRoot = join(root, "agent-config");
    const nativeRoot = join(root, "native");
    const agentNativeRoot = join(root, "agent-native");
    const protocolRoot = join(root, "protocol");
    const agentProtocolRoot = join(root, "agent-protocol");
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
        await writeFile(
            join(webRoot, "index.html"),
            `<meta name="rustzen-web-binding" content="${WEB_BINDING_SLOT}" />monitor`,
        );
        await writeFixtureWebPolicyFiles(webRoot, selection);
        await writeFixtureWebBinding(webRoot, selection);
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
        await produceSelectedProtocol(
            selection,
            protocolRoot,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        await produceSelectedProtocol(
            { preset: "node-agent", target },
            agentProtocolRoot,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
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
        let stageSequence = 0;
        const stageServer = async () =>
            produceNativeStaging({
                ...inputs,
                selection,
                outputParent: join(root, `staged-${stageSequence++}`),
                trustedRoot: root,
                binaryRoot: artifactRoot,
                webRoot,
                apiRoot,
                schemaRoot,
                configRoot,
                nativeRoot,
                protocolRoot,
            });
        const stageAgent = async () =>
            produceNativeStaging({
                ...inputs,
                selection: { preset: "node-agent", target },
                outputParent: join(root, `staged-${stageSequence++}`),
                trustedRoot: root,
                binaryRoot: agentRoot,
                configRoot: agentConfigRoot,
                nativeRoot: agentNativeRoot,
                protocolRoot: agentProtocolRoot,
            });

        const server = await produceReleaseManifest({
            ...inputs,
            selection,
            staging: await stageServer(),
        });
        expect((server as any).apiDigest).toMatch(/^[0-9a-f]{64}$/);
        expect((server as any).configDigest).toMatch(/^[0-9a-f]{64}$/);
        expect((server as any).nativeLayoutDigest).toMatch(/^[0-9a-f]{64}$/);
        expect((server as any).protocolArtifactDigest).toMatch(
            /^[0-9a-f]{64}$/,
        );
        expect((server as any).agentProtocolContractId).toMatch(
            /^[0-9a-f]{64}$/,
        );
        expect((server as any).protocolArtifactDigest).not.toBe(
            (server as any).agentProtocolContractId,
        );
        await writeFile(join(protocolRoot, "protocol.json"), "{}");
        await expect(stageServer()).rejects.toThrow("reviewed descriptor");
        await produceSelectedProtocol(
            selection,
            protocolRoot,
            reviewedProtocolOutput(),
            reviewedProtocolOutput(),
        );
        await writeFile(join(nativeRoot, "native-layout.json"), "{}");
        await expect(stageServer()).rejects.toThrow(
            "selected generated source",
        );
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
        await expect(stageServer()).rejects.toThrow("fresh-install SQL");
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
            { protocolArtifactDigest: h("a") },
            { protocolId: h("a") },
            { agentProtocolContractId: h("a") },
        ]) {
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection,
                    staging: await stageServer(),
                    ...supplied,
                } as any),
            ).rejects.toThrow("caller-supplied");
        }
        await writeFile(join(configRoot, "config.json"), "{}");
        await expect(stageServer()).rejects.toThrow("reviewed descriptors");
        await writeFile(
            join(configRoot, "config.json"),
            canonicalJson(completeSelectedConfigForTest(selection)),
        );

        await writeFile(join(apiRoot, "api.json"), "{}");
        await expect(stageServer()).rejects.toThrow();
        await writeFile(join(apiRoot, "api.json"), validApi);
        await writeFile(join(apiRoot, "extra.json"), "{}");
        await expect(stageServer()).rejects.toThrow("exactly api.json");

        for (const apiInput of [{ apiRoot }, { schemaRoot }]) {
            await expect(
                produceReleaseManifest({
                    ...inputs,
                    selection: { preset: "node-agent", target },
                    staging: await stageAgent(),
                    ...apiInput,
                } as any),
            ).rejects.toThrow("forbids");
        }
        const agent = await produceReleaseManifest({
            ...inputs,
            selection: { preset: "node-agent", target },
            staging: await stageAgent(),
        });
        expect("apiDigest" in agent).toBeFalse();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});
