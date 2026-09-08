import {
    publishNativeStagingBytes,
    type StagingResult,
} from "./native-staging.ts";
import { fromContainerSnapshot } from "./native-staging-source.ts";
import { readVerifiedNativeSource } from "./native-staging-source.ts";
import { produceReleaseManifest, type ReleaseManifest } from "./release-manifest.ts";
import { resolveSelection } from "./resolver.ts";
import { type VerifiedContainerExportSnapshot } from "./container-export-validator.ts";

/** Builds Monitor staging only from the retained verified container snapshot bytes. */
export async function produceMonitorNativeStagingManifest(input: {
    snapshot: VerifiedContainerExportSnapshot;
    outputParent: string;
    trustedRoot: string;
}): Promise<{ staging: StagingResult; manifest: ReleaseManifest }> {
    const selection = input.snapshot.selection();
    const plan = resolveSelection(selection);
    if (plan.preset !== "monitor" || plan.artifactClass !== "server")
        throw new Error("container native staging supports only Monitor server snapshots");
    const source = fromContainerSnapshot(input.snapshot);
    const captured = readVerifiedNativeSource(source);
    const staging = await publishNativeStagingBytes({
        outputParent: input.outputParent,
        trustedRoot: input.trustedRoot,
        source,
    });
    return {
        staging,
        manifest: await produceReleaseManifest({
            ...captured.buildInputs,
            selection: captured.selection,
            staging,
        }),
    };
}
