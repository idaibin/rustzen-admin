import {
    publishNativeStagingBytes,
    type StagingResult,
} from "./native-staging.ts";
import { fromContainerSnapshot } from "./native-staging-source.ts";
import { readVerifiedNativeSource } from "./native-staging-source.ts";
import {
    produceReleaseManifest,
    type ReleaseManifest,
} from "./release-manifest.ts";
import { resolveSelection } from "./resolver.ts";
import { reviewedContainerServerPlan } from "./container-export-plan.ts";
import { selectedServerSyntheticExportPlan } from "./selected-server-synthetic-export-plan.ts";
import { type VerifiedContainerExportSnapshot } from "./container-export-validator.ts";

/** Builds reviewed server staging only from the retained verified container snapshot bytes. */
export async function produceMonitorNativeStagingManifest(input: {
    snapshot: VerifiedContainerExportSnapshot;
    outputParent: string;
    trustedRoot: string;
}): Promise<{ staging: StagingResult; manifest: ReleaseManifest }> {
    const selection = input.snapshot.selection();
    const plan = resolveSelection(selection);
    reviewedContainerServerPlan(plan);
    return produceSelectedServerSyntheticNativeStagingManifest(input);
}

/** Produces a release-manifest-shaped captured staging result for exact host-synthetic servers. */
export async function produceSelectedServerSyntheticNativeStagingManifest(input: {
    snapshot: VerifiedContainerExportSnapshot;
    outputParent: string;
    trustedRoot: string;
}): Promise<{ staging: StagingResult; manifest: ReleaseManifest }> {
    const selection = input.snapshot.selection();
    selectedServerSyntheticExportPlan(resolveSelection(selection));
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
