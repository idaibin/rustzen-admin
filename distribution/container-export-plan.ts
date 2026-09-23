import { isExactSupportedPlan, type SourceBuildPlan } from "./source-build-plan.ts";

export type ContainerServerPlan = SourceBuildPlan & {
    preset: "monitor" | "monitor-notify" | "analytics";
    artifactClass: "server";
};

export function isReviewedContainerServerPlan(plan: SourceBuildPlan): plan is ContainerServerPlan {
    return isExactSupportedPlan(plan, ["monitor", "monitor-notify", "analytics"]);
}

export function reviewedContainerServerPlan(plan: SourceBuildPlan): asserts plan is ContainerServerPlan {
    if (!isReviewedContainerServerPlan(plan))
        throw new Error("container export supports only reviewed server selections");
}
