import { isAbsolute, relative, resolve, sep } from "node:path";
import { realpath } from "node:fs/promises";
import { type VerifiedContainerExportSnapshot } from "./container-export-validator.ts";
import { canonicalJson, deriveBuildId } from "./release-manifest-core.ts";
import { readVerifiedNativeSource, fromContainerSnapshot } from "./native-staging-source.ts";
import { type TrustedReleaseKey } from "./release-envelope.ts";
import { verifyReleaseSnapshot } from "./release-publisher.ts";
import { produceSourceBuildCertificate, type SourceBuildCertificate } from "./source-build-certificate.ts";
import { parseSourceIdentity } from "./source-identity.ts";
import { resolveSelection } from "./resolver.ts";
import { deriveReleaseManifestFromFiles } from "./release-manifest.ts";

const issuedToken = Symbol("issued source-build certificate");
type IssuedState = {
    certificate: SourceBuildCertificate;
    selection: unknown;
    releaseRoot: string;
    trustedRoot: string;
};
const issuedStates = new WeakMap<IssuedSourceBuildCertificate, IssuedState>();
let createIssued: (state: IssuedState) => IssuedSourceBuildCertificate;
export class IssuedSourceBuildCertificate {
    private constructor(token: typeof issuedToken, state: IssuedState) {
        if (token !== issuedToken)
            throw new Error("source-build issue construction is internal");
        issuedStates.set(this, structuredClone(state));
        Object.freeze(this);
    }
    static {
        createIssued = (state) => new IssuedSourceBuildCertificate(issuedToken, state);
    }
}
Object.freeze(IssuedSourceBuildCertificate.prototype);
Object.freeze(IssuedSourceBuildCertificate);

export function readIssuedSourceBuildCertificate(value: unknown): IssuedState {
    if (!(value instanceof IssuedSourceBuildCertificate) || Reflect.ownKeys(value).length !== 0)
        throw new Error("source-build issued capability is invalid");
    const state = issuedStates.get(value);
    if (!state) throw new Error("source-build issued capability is invalid");
    return structuredClone(state);
}

/** Derives one certificate only from verified captured bytes and release reread. */
export async function issueSourceBuildCertificate(input: {
    snapshot: VerifiedContainerExportSnapshot;
    releaseRoot: string;
    trustedRoot: string;
    trusted: TrustedReleaseKey;
}): Promise<IssuedSourceBuildCertificate> {
    const releaseRoot = resolve(input.releaseRoot);
    const trustedRoot = await realpath(resolve(input.trustedRoot));
    assertContained(trustedRoot, await realpath(releaseRoot));
    const selection = input.snapshot.selection();
    const release = await verifyReleaseSnapshot(releaseRoot, selection, input.trusted);
    const provenance = input.snapshot.recordedProvenance();
    const identity = parseSourceIdentity(provenance.sourceIdentityInput);
    const source = readVerifiedNativeSource(fromContainerSnapshot(input.snapshot));
    const buildId = deriveBuildId(source.selection, source.buildInputs, source.digests);
    const manifest = release.manifest;
    const plan = resolveSelection(selection);
    if (manifest.artifactClass !== "server")
        throw new Error("source-build manifest is not a server artifact");
    if (
        manifest.releaseVersion !== provenance.releaseVersion ||
        manifest.sourceIdentity !== identity.text ||
        manifest.buildId !== buildId ||
        manifest.target !== plan.target ||
        manifest.compositionId !== plan.compositionId
    ) throw new Error("source-build evidence differs from snapshot");
    const expected = deriveReleaseManifestFromFiles({
        ...source.buildInputs,
        selection: source.selection,
        files: source.files,
        expectedBuildId: buildId,
    });
    if (canonicalJson(manifest) !== canonicalJson(expected))
        throw new Error("source-build full manifest differs from snapshot");
    const certificate = produceSourceBuildCertificate({
        selection,
        manifest,
        sourceTreeSha256: identity.treeSha256,
        toolchain: provenance.rustcVv,
        archiveSha256: release.archiveSha256,
        envelopeSha256: release.envelopeSha256,
    });
    return createIssued({ certificate, selection, releaseRoot, trustedRoot });
}

function assertContained(root: string, path: string) {
    const value = relative(root, path);
    if (value === ".." || value.startsWith(`..${sep}`) || isAbsolute(value))
        throw new Error("source-build release escaped trusted root");
}
