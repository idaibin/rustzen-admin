import { canonicalJson, sha256, validHash } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import { parseReleaseManifest, type ReleaseManifest } from "./release-manifest.ts";
import { auditSourceBuildReadiness } from "./source-build-readiness.ts";
import { reviewedContainerServerPlan } from "./container-export-plan.ts";

export type SourceBuildCertificate = {
    schemaVersion: 1;
    kind: "source-build-manifest";
    selection: {
        preset: string;
        target: string;
        artifactClass: "server" | "node-agent";
        compositionId: string;
        selectionDigest: string;
        capabilities: string[];
        services: string[];
    };
    certifiedLayers: {
        source: { identity: string; treeSha256: string };
        build: {
            toolchain: string;
            buildId: string;
            binaryDigests: Array<{ path: string; sha256: string }>;
        };
        artifact: { manifestSha256: string; archiveSha256: string; envelopeSha256: string };
    };
    runtime: false;
    browser: false;
    load: false;
    releaseReady: false;
};

export type SourceBuildCertificateInput = {
    selection: unknown;
    manifest: unknown;
    sourceTreeSha256: string;
    toolchain: string;
    archiveSha256: string;
    envelopeSha256: string;
};

export function produceSourceBuildCertificate(
    input: SourceBuildCertificateInput,
): SourceBuildCertificate {
    const plan = resolveSelection(input.selection);
    reviewedContainerServerPlan(plan);
    if (!auditSourceBuildReadiness(input.selection).admissionReady)
        throw new Error("source-build certification requires producer admission");
    const manifest = parseReleaseManifest(input.manifest, input.selection);
    if (manifest.releaseClass !== "production")
        throw new Error("source-build certification requires a production manifest");
    const certificate: SourceBuildCertificate = {
        schemaVersion: 1,
        kind: "source-build-manifest",
        selection: {
            preset: plan.preset,
            target: plan.target,
            artifactClass: plan.artifactClass,
            compositionId: plan.compositionId,
            selectionDigest: manifest.selectionDigest.sha256,
            capabilities: plan.capabilities,
            services: plan.services,
        },
        certifiedLayers: {
            source: {
                identity: required(manifest.sourceIdentity, "source identity"),
                treeSha256: validHash(input.sourceTreeSha256),
            },
            build: {
                toolchain: required(input.toolchain, "toolchain"),
                buildId: manifest.buildId,
                binaryDigests: manifest.binaryDigests.map(({ path, sha256 }) => ({ path, sha256 })),
            },
            artifact: {
                manifestSha256: sha256(canonicalManifest(manifest, input.selection)),
                archiveSha256: validHash(input.archiveSha256),
                envelopeSha256: validHash(input.envelopeSha256),
            },
        },
        runtime: false,
        browser: false,
        load: false,
        releaseReady: false,
    };
    return parseSourceBuildCertificate(certificate, input.selection);
}

export function sourceBuildCertificateBytes(
    value: SourceBuildCertificate,
    selection: unknown,
): Uint8Array {
    return new TextEncoder().encode(canonicalJson(parseSourceBuildCertificate(value, selection)));
}

export function verifySourceBuildCertificate(
    value: unknown,
    input: SourceBuildCertificateInput,
): SourceBuildCertificate {
    const actual = parseSourceBuildCertificate(value, input.selection);
    const expected = produceSourceBuildCertificate(input);
    if (canonicalJson(actual) !== canonicalJson(expected))
        throw new Error("source-build certificate differs from authoritative inputs");
    return actual;
}

export function parseSourceBuildCertificate(
    value: unknown,
    selection: unknown,
): SourceBuildCertificate {
    const record = object(value, "source-build certificate");
    only(record, [
        "schemaVersion",
        "kind",
        "selection",
        "certifiedLayers",
        "runtime",
        "browser",
        "load",
        "releaseReady",
    ]);
    if (record.schemaVersion !== 1 || record.kind !== "source-build-manifest")
        throw new Error("source-build certificate version or kind is invalid");
    if (
        record.runtime !== false ||
        record.browser !== false ||
        record.load !== false ||
        record.releaseReady !== false
    )
        throw new Error(
            "source-build certificate cannot claim runtime, browser, load, or release readiness",
        );
    const plan = resolveSelection(selection);
    reviewedContainerServerPlan(plan);
    if (!auditSourceBuildReadiness(selection).admissionReady)
        throw new Error("source-build certification requires producer admission");
    const expected = {
        preset: plan.preset,
        target: plan.target,
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        selectionDigest: sha256(canonicalJson(plan)),
        capabilities: plan.capabilities,
        services: plan.services,
    };
    if (canonicalJson(record.selection) !== canonicalJson(expected))
        throw new Error("source-build certificate selection differs from resolved plan");
    const layers = object(record.certifiedLayers, "certifiedLayers");
    only(layers, ["source", "build", "artifact"]);
    const source = object(layers.source, "certifiedLayers.source");
    const build = object(layers.build, "certifiedLayers.build");
    const artifact = object(layers.artifact, "certifiedLayers.artifact");
    only(source, ["identity", "treeSha256"]);
    only(build, ["toolchain", "buildId", "binaryDigests"]);
    only(artifact, ["manifestSha256", "archiveSha256", "envelopeSha256"]);
    const binaryDigests = Array.isArray(build.binaryDigests)
        ? build.binaryDigests.map((binary) => {
              const digest = object(binary, "binary digest");
              only(digest, ["path", "sha256"]);
              return {
                  path: required(digest.path, "binary path"),
                  sha256: validHash(required(digest.sha256, "binary digest")),
              };
          })
        : (() => {
              throw new Error("binaryDigests must be an array");
          })();
    if (
        canonicalJson(binaryDigests.map((x) => x.path)) !==
        canonicalJson(["bin/rz-admin", "bin/rz-monitor"])
    )
        throw new Error("source-build certificate binary inventory is invalid");
    return {
        schemaVersion: 1,
        kind: "source-build-manifest",
        selection: expected,
        certifiedLayers: {
            source: {
                identity: required(source.identity, "source identity"),
                treeSha256: validHash(required(source.treeSha256, "source tree digest")),
            },
            build: {
                toolchain: required(build.toolchain, "toolchain"),
                buildId: validHash(required(build.buildId, "build ID")),
                binaryDigests,
            },
            artifact: {
                manifestSha256: validHash(required(artifact.manifestSha256, "manifest digest")),
                archiveSha256: validHash(required(artifact.archiveSha256, "archive digest")),
                envelopeSha256: validHash(required(artifact.envelopeSha256, "envelope digest")),
            },
        },
        runtime: false,
        browser: false,
        load: false,
        releaseReady: false,
    };
}

function canonicalManifest(manifest: ReleaseManifest, selection: unknown): Uint8Array {
    // Parsing above binds the manifest to the same resolved composition before it is hashed.
    parseReleaseManifest(manifest, selection);
    return new TextEncoder().encode(canonicalJson(manifest));
}
function object(value: unknown, label: string): Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error(`${label} must be an object`);
    return value as Record<string, unknown>;
}
function only(record: Record<string, unknown>, keys: string[]) {
    const allowed = new Set(keys);
    for (const key of Object.keys(record))
        if (!allowed.has(key)) throw new Error(`unknown source-build certificate field: ${key}`);
}
function required(value: unknown, label: string): string {
    if (typeof value !== "string" || !value) throw new Error(`${label} must be nonempty`);
    return value;
}
