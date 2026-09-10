import { isExactSupportedPlan, type SourceBuildPlan } from "./source-build-plan.ts";

export type ContainerServerPlan = SourceBuildPlan & {
    preset: "monitor" | "monitor-notify";
    artifactClass: "server";
};

export function isReviewedContainerServerPlan(plan: SourceBuildPlan): plan is ContainerServerPlan {
    return isExactSupportedPlan(plan, ["monitor", "monitor-notify"]);
}

export function reviewedContainerServerPlan(plan: SourceBuildPlan): asserts plan is ContainerServerPlan {
    if (!isReviewedContainerServerPlan(plan))
        throw new Error("container export supports only reviewed monitor server selections");
}

export function containerBuildCommands(plan: ContainerServerPlan): string[][] {
    const notify = plan.preset === "monitor-notify";
    const base = ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl"];
    return [
        [...base, "-p", "rustzen-admin", "--no-default-features", "--features", notify ? "monitor-distribution,notifications" : "monitor-distribution"],
        [...base, "-p", "rustzen-monitor", "--no-default-features", "--features", notify ? "notifications" : "controller", "--bin", "rz-monitor"],
        [...base, "-p", "rustzen-monitor", "--no-default-features", "--features", "agent", "--bin", "rz-monitor-agent"],
    ];
}
