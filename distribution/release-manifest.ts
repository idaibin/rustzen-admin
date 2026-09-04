import { resolveSelection } from "./resolver.ts";
import { parseReleaseManifest } from "./release-manifest-validator.ts";
import {
    canonicalJson,
    deriveBuildId,
    nonempty,
    selectionDigest,
    sha256,
    sortedStrings,
    validHash,
} from "./release-manifest-core.ts";
import {
    readArtifactFiles,
    readWebDigest,
} from "./release-manifest-artifacts.ts";
import { readSelectedApiContract } from "./selected-contract.ts";
import { readSchemaContract } from "./schema-contract.ts";
import { readSelectedConfig } from "./selected-config.ts";
import type {
    AgentManifest,
    BinaryDigest,
    BuildContractDigests,
    ManifestBase,
    BuildInputs,
    Digest,
    DigestSource,
    FileEntry,
    ProduceInput,
    ReleaseManifest,
    ServerManifest,
} from "./release-manifest-types.ts";
export type {
    AgentManifest,
    BinaryDigest,
    BuildContractDigests,
    ManifestBase,
    BuildInputs,
    Digest,
    DigestSource,
    FileEntry,
    ProduceInput,
    ReleaseManifest,
    ServerManifest,
} from "./release-manifest-types.ts";
export {
    canonicalJson,
    deriveBuildId,
    filesDigest,
    selectionDigest,
    sha256,
} from "./release-manifest-core.ts";

export async function produceReleaseManifest(
    input: ProduceInput,
): Promise<ReleaseManifest> {
    const plan = resolveSelection(input.selection);
    if (
        "apiDigest" in input ||
        "schemaDigest" in input ||
        "schemaFingerprints" in input ||
        "dataContractIds" in input ||
        "configDigest" in input
    )
        throw new Error("manifest forbids caller-supplied contract digests");
    if (
        plan.artifactClass === "node-agent" &&
        ("apiRoot" in input || "schemaRoot" in input)
    )
        throw new Error("node-agent manifest forbids server contract roots");
    const apiDigest =
        plan.artifactClass === "server"
            ? (
                  await readSelectedApiContract(
                      required(input.apiRoot, "apiRoot"),
                      input.selection,
                  )
              ).sha256
            : undefined;
    const schema =
        plan.artifactClass === "server"
            ? await readSchemaContract(
                  required(input.schemaRoot, "schemaRoot"),
                  input.selection,
              )
            : undefined;
    const config = await readSelectedConfig(
        required(input.configRoot, "configRoot"),
        input.selection,
    );
    const files = await readArtifactFiles(input.artifactRoot);
    const binaries = expectedBinaries(plan);
    const actualBinaries = files
        .filter((file) => file.path.startsWith("bin/"))
        .map((file) => file.path)
        .sort();
    if (canonicalJson(actualBinaries) !== canonicalJson(binaries))
        throw new Error(
            "artifact binary inventory does not match selected class",
        );
    const binaryDigests = binaries.map((path) => ({
        path,
        sha256: files.find((file) => file.path === path)!.sha256,
        source: "binary-file" as const,
    }));
    const buildInputs: BuildInputs = input;
    const base: ManifestBase = {
        manifestVersion: 1,
        releaseClass: plan.releaseClass,
        releaseVersion: nonempty(input.releaseVersion, "releaseVersion"),
        target: plan.target,
        artifactClass: plan.artifactClass as "server" | "node-agent",
        preset: plan.preset,
        capabilities: plan.capabilities,
        services: plan.services,
        compositionId: plan.compositionId,
        selectionDigest: selectionDigest(input.selection),
        buildId: deriveBuildId(
            input.selection,
            buildInputs,
            plan.artifactClass === "server"
                ? {
                      apiDigest,
                      schemaDigest: schema!.sha256,
                      configDigest: config.sha256,
                  }
                : { configDigest: config.sha256 },
        ),
        sourceIdentity: nonempty(input.sourceIdentity, "sourceIdentity"),
        configDigest: config.sha256,
        configOwners: plan.configOwners,
        binaryDigests,
        files,
    };
    if (plan.artifactClass === "node-agent") {
        const manifest = {
            ...base,
            artifactClass: "node-agent",
            agentProtocolContractId: validHash(
                required(input.protocolId, "protocolId"),
            ),
        } as ReleaseManifest;
        parseReleaseManifest(manifest, input.selection);
        return manifest;
    }
    if (!input.webRoot)
        throw new Error("server producer requires selected Web contracts");
    const manifest: ReleaseManifest = {
        ...base,
        artifactClass: "server",
        apiDigest: apiDigest!,
        agentProtocolContractId: plan.capabilities.includes("monitor")
            ? validHash(required(input.protocolId, "protocolId"))
            : undefined,
        schemaFingerprints: Object.fromEntries(
            Object.entries(schema!.contract.owners).map(([owner, value]) => [
                owner,
                value.schemaSha256,
            ]),
        ),
        dataContractIds: Object.fromEntries(
            Object.entries(schema!.contract.owners).map(([owner, value]) => [
                owner,
                value.dataContractId,
            ]),
        ),
        webDigest: await readWebDigest(input.webRoot),
    };
    parseReleaseManifest(manifest, input.selection);
    return manifest;
}

export {
    canonicalManifestBytes,
    parseReleaseManifest,
    validateServerAgentPair,
} from "./release-manifest-validator.ts";

type Plan = ReturnType<typeof resolveSelection>;
function expectedBinaries(plan: Plan): string[] {
    if (plan.artifactClass === "node-agent") return ["bin/rz-monitor-agent"];
    if (plan.preset === "monitor") return ["bin/rz-admin", "bin/rz-monitor"];
    throw new Error("producer supports only monitor server or node-agent");
}
function required(value: unknown, label: string): string {
    if (value === undefined) throw new Error(`${label} is required`);
    return nonempty(value, label);
}
