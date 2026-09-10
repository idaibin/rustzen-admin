import {
    isExactSupportedPlan,
    type SourceBuildPlan,
} from "./source-build-plan.ts";

/** Cargo invocation is a source producer, so its supported closures are explicit. */
export const supportsSelectedCargo = (plan: SourceBuildPlan): boolean =>
    isExactSupportedPlan(plan, ["monitor", "monitor-notify", "node-agent"]);

/** Service-only steps may land before the complete multi-binary producer is admitted. */
export const supportsSelectedServiceCargo = (plan: SourceBuildPlan): boolean =>
    isExactSupportedPlan(plan, ["analytics"]);

export function selectedServiceCargoBuilds(plan: SourceBuildPlan): string[][] {
    if (!supportsSelectedServiceCargo(plan))
        throw new Error("selected service Cargo producer does not support this exact closure");
    return [[
        "cargo",
        "build",
        "-p",
        "rustzen-insights",
        "--no-default-features",
        "--features",
        "selected-distribution",
        "--bin",
        "rz-insights",
    ]];
}

export function selectedCargoBuilds(plan: SourceBuildPlan): string[][] {
    if (!supportsSelectedCargo(plan))
        throw new Error("selected Cargo producer supports only reviewed monitor or agent closures");
    if (plan.artifactClass === "node-agent")
        return [[
            "cargo",
            "build",
            "-p",
            "rustzen-monitor",
            "--no-default-features",
            "--features",
            "agent",
            "--bin",
            "rz-monitor-agent",
        ]];
    const notify = plan.preset === "monitor-notify";
    return [
        [
            "cargo",
            "build",
            "-p",
            "rustzen-admin",
            "--no-default-features",
            "--features",
            notify ? "monitor-distribution,notifications" : "monitor-distribution",
            "--bin",
            "rz-admin",
        ],
        [
            "cargo",
            "build",
            "-p",
            "rustzen-monitor",
            "--no-default-features",
            "--features",
            notify ? "notifications" : "controller",
            "--bin",
            "rz-monitor",
        ],
    ];
}
