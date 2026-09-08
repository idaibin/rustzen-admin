import { produceMonitorNativeStagingManifest } from "./container-export-native-staging.ts";
import { type VerifiedContainerExportSnapshot } from "./container-export-validator.ts";
import { validKeyId } from "./release-envelope.ts";
import { validateReleaseSigningKeyPair } from "./release-key-file.ts";
import { publishSelectedRelease, verifyReleaseSnapshot, type VerifiedReleaseSnapshot } from "./release-publisher.ts";

let beforeRereadHook: ((root: string) => Promise<void> | void) | undefined;

/** Test-only seam between publication and publication-bound reread. */
export const setContainerReleaseBeforeRereadHookForTest = (
    hook?: (root: string) => Promise<void> | void,
) => { beforeRereadHook = hook; };

/** Converts one retained Monitor export snapshot into a signed immutable release. */
export async function publishMonitorContainerRelease(input: {
    snapshot: VerifiedContainerExportSnapshot;
    outputParent: string;
    trustedRoot: string;
    privateKey: string;
    trusted: { keyId: string; publicKey: string };
}): Promise<VerifiedReleaseSnapshot & { stagingRoot: string; buildId: string }> {
    validKeyId(input.trusted.keyId);
    validateReleaseSigningKeyPair(input.privateKey, input.trusted.publicKey);
    const stagingResult = await produceMonitorNativeStagingManifest({
        snapshot: input.snapshot,
        outputParent: input.outputParent,
        trustedRoot: input.trustedRoot,
    });
    const publication = await publishSelectedRelease({
        selection: input.snapshot.selection(),
        staging: stagingResult.staging,
        manifest: stagingResult.manifest,
        privateKey: input.privateKey,
        trusted: input.trusted,
    });
    await beforeRereadHook?.(publication.root);
    const verified = await verifyReleaseSnapshot(
        publication.root,
        input.snapshot.selection(),
        input.trusted,
    );
    if (
        verified.archiveSha256 !== publication.archiveSha256 ||
        verified.manifestSha256 !== publication.manifestSha256 ||
        verified.envelopeSha256 !== publication.envelopeSha256 ||
        verified.manifest.buildId !== stagingResult.staging.buildId
    ) throw new Error("published release reread differs from publication tuple");
    return {
        ...verified,
        stagingRoot: stagingResult.staging.root,
        buildId: stagingResult.staging.buildId,
    };
}
