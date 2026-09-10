import { parseNativeLayoutBytes } from "./native-layout.ts";
import { parseReleaseManifest } from "./release-manifest-validator.ts";
import {
    readArtifactFileTree,
    type ArtifactFile,
} from "./release-manifest-artifacts.ts";
import {
    canonicalJson,
    deriveBuildId,
    filesDigest,
    nonempty,
    selectionDigest,
    sha256,
} from "./release-manifest-core.ts";
import { parseSelectedApiBytes } from "./selected-contract-validator.ts";
import { parseSelectedConfigBytes } from "./selected-config.ts";
import { parseSelectedProtocolBytes } from "./selected-protocol.ts";
import { resolveSelection } from "./resolver.ts";
import { isReviewedContainerServerPlan } from "./container-export-plan.ts";
import { parseSchemaArtifactBytes } from "./schema-contract.ts";
import { releaseWebDigest } from "./release-manifest-web-binding.ts";
import type {
    AgentManifest,
    BinaryDigest,
    BuildInputs,
    ManifestBase,
    ProduceInput,
    ReleaseManifest,
    ServerManifest,
} from "./release-manifest-types.ts";
export type {
    AgentManifest,
    BinaryDigest,
    BuildContractDigests,
    BuildInputs,
    Digest,
    DigestSource,
    FileEntry,
    ManifestBase,
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
    for (const forbidden of [
        "apiDigest",
        "schemaDigest",
        "schemaFingerprints",
        "dataContractIds",
        "configDigest",
        "nativeLayoutDigest",
        "protocolArtifactDigest",
        "protocolId",
        "agentProtocolContractId",
        "artifactRoot",
        "webRoot",
        "apiRoot",
        "schemaRoot",
        "configRoot",
        "nativeRoot",
        "protocolRoot",
        "payloadRoot",
    ])
        if (forbidden in input)
            throw new Error(
                "manifest forbids caller-supplied contract digests",
            );
    const staging = input.staging;
    if (!staging || typeof staging !== "object")
        throw new Error("manifest requires strict staging reference");
    if (
        canonicalJson(Object.keys(staging).sort()) !==
        canonicalJson([
            "artifactClass",
            "buildId",
            "files",
            "root",
            "sha256",
            "target",
        ])
    )
        throw new Error("staging reference fields are invalid");
    if (
        staging.target !== plan.target ||
        staging.artifactClass !== plan.artifactClass
    )
        throw new Error("staging reference selection mismatch");
    if (
        !staging.root.endsWith(
            `/${staging.buildId}/${staging.target}/${staging.artifactClass}/payload`,
        )
    )
        throw new Error("staging root does not match reference tuple");
    const snapshot = await readArtifactFileTree(staging.root);
    if (
        canonicalJson(snapshot.map((file) => file.entry)) !==
            canonicalJson(staging.files) ||
        sha256(canonicalJson(staging.files)) !== staging.sha256
    )
        throw new Error("staging reference inventory differs from payload");
    return deriveReleaseManifestFromFiles({
        ...input,
        files: snapshot,
        expectedBuildId: staging.buildId,
    });
}

export function deriveReleaseManifestFromFiles(
    input: BuildInputs & {
        selection: unknown;
        files: ArtifactFile[];
        expectedBuildId: string;
    },
): ReleaseManifest {
    const plan = resolveSelection(input.selection);
    const snapshot = input.files;
    const byPath = new Map(snapshot.map((file) => [file.entry.path, file]));
    const get = (path: string) => {
        const value = byPath.get(path);
        if (!value) throw new Error(`payload is missing ${path}`);
        return value;
    };
    const config = get("contracts/config/config.json");
    const native = get("contracts/native/native-layout.json");
    const protocol = get("contracts/protocol/protocol.json");
    parseSelectedConfigBytes(config.bytes, input.selection);
    const nativeLayout = parseNativeLayoutBytes(native.bytes, input.selection);
    const selectedProtocol = parseSelectedProtocolBytes(
        protocol.bytes,
        input.selection,
    );
    const binaries = expectedBinaries(plan);
    const actualBinaries = snapshot
        .filter((file) => file.entry.path.startsWith("bin/"))
        .map((file) => file.entry.path)
        .sort();
    if (canonicalJson(actualBinaries) !== canonicalJson(binaries))
        throw new Error(
            "artifact binary inventory does not match selected class",
        );
    const server = plan.artifactClass === "server";
    let apiDigest: string | undefined,
        schema: ReturnType<typeof parseSchemaArtifactBytes> | undefined;
    if (server) {
        const api = get("contracts/api/api.json");
        const schemaFile = get("contracts/schema/schema.json");
        parseSelectedApiBytes(api.bytes, input.selection);
        apiDigest = api.entry.sha256;
        schema = parseSchemaArtifactBytes(schemaFile.bytes, input.selection);
        const web = snapshot.filter((file) =>
            file.entry.path.startsWith("web/"),
        );
        if (!web.length) throw new Error("server payload requires web files");
    }
    const expectedUnits = nativeLayout.layout.units
        .map((unit) => unit.path)
        .sort();
    const actualUnits = snapshot
        .filter((file) => file.entry.path.startsWith("systemd/"))
        .map((file) => ({ path: file.entry.path, sha256: file.entry.sha256 }))
        .sort((a, b) => a.path.localeCompare(b.path));
    if (
        canonicalJson(actualUnits) !==
            canonicalJson(nativeLayout.layout.units) ||
        canonicalJson(actualUnits.map((x) => x.path)) !==
            canonicalJson(expectedUnits)
    )
        throw new Error("payload units differ from native layout");
    const fixedPaths = [
        ...binaries,
        "contracts/config/config.json",
        "contracts/native/native-layout.json",
        "contracts/protocol/protocol.json",
        ...expectedUnits,
        ...(server
            ? [
                  "contracts/api/api.json",
                  "contracts/schema/schema.json",
                  "contracts/web/binding.json",
              ]
            : []),
    ];
    const unexpected = snapshot.some(
        (file) =>
            !fixedPaths.includes(file.entry.path) &&
            !(server && file.entry.path.startsWith("web/")),
    );
    if (
        unexpected ||
        fixedPaths.some((path) => !byPath.has(path)) ||
        (!server && snapshot.some((file) => file.entry.path.startsWith("web/")))
    )
        throw new Error("payload inventory differs from selected class");
    const digests = {
        configDigest: config.entry.sha256,
        nativeLayoutDigest: native.entry.sha256,
        protocolArtifactDigest: protocol.entry.sha256,
        ...(server ? { apiDigest, schemaDigest: schema!.sha256 } : {}),
    };
    const buildId = deriveBuildId(
        input.selection,
        input as BuildInputs,
        digests,
    );
    if (buildId !== input.expectedBuildId)
        throw new Error("staging buildId differs from verified contracts");
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
        buildId,
        sourceIdentity: nonempty(input.sourceIdentity, "sourceIdentity"),
        configDigest: config.entry.sha256,
        nativeLayoutDigest: native.entry.sha256,
        protocolArtifactDigest: protocol.entry.sha256,
        configOwners: plan.configOwners,
        binaryDigests: binaries.map((path) => ({
            path,
            sha256: get(path).entry.sha256,
            source: "binary-file" as const,
        })),
        files: snapshot.map((file) => file.entry),
    };
    if (!server) {
        const manifest = {
            ...base,
            artifactClass: "node-agent",
            agentProtocolContractId: selectedProtocol.protocol.digest,
        } as ReleaseManifest;
        parseReleaseManifest(manifest, input.selection);
        return manifest;
    }
    const webBinding = get("contracts/web/binding.json");
    const manifest: ReleaseManifest = {
        ...base,
        artifactClass: "server",
        apiDigest: apiDigest!,
        agentProtocolContractId: selectedProtocol.protocol.digest,
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
        webDigest: { sha256: releaseWebDigest({ compositionId: plan.compositionId, files: snapshot, descriptor: webBinding }), source: "selected-web-files" },
    };
    parseReleaseManifest(manifest, input.selection);
    return manifest;
}
export {
    canonicalManifestBytes,
    parseReleaseManifest,
    validateServerAgentPair,
} from "./release-manifest-validator.ts";
function expectedBinaries(plan: ReturnType<typeof resolveSelection>): string[] {
    if (plan.artifactClass === "node-agent") return ["bin/rz-monitor-agent"];
    if (isReviewedContainerServerPlan(plan)) return ["bin/rz-admin", "bin/rz-monitor"];
    throw new Error("producer supports only reviewed server selections or node-agent");
}
