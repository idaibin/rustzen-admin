import { isExactSupportedPlan, type SourceBuildPlan } from "./source-build-plan.ts";
import { selectedServerInventory } from "./selected-server-inventory.ts";

export type SyntheticServerPlan = SourceBuildPlan & {
    preset: "analytics" | "monitor" | "monitor-notify";
    artifactClass: "server";
};

/** Host-only synthetic evidence plan; it is intentionally independent of Docker admission. */
export function selectedServerSyntheticExportPlan(plan: SourceBuildPlan & { target?: string }): SyntheticServerPlan {
    if (!isExactSupportedPlan(plan, ["analytics", "monitor", "monitor-notify"]) ||
        plan.target !== "x86_64-unknown-linux-musl")
        throw new Error("synthetic export supports only exact selected server closures");
    return plan as SyntheticServerPlan;
}

export function syntheticServerBuildCommands(plan: SourceBuildPlan): string[][] {
    const selected = selectedServerSyntheticExportPlan(plan);
    const base = ["env", "RUSTFLAGS=-C target-feature=+crt-static", "cargo", "build", "--release", "--target", "x86_64-unknown-linux-musl"];
    if (selected.preset === "analytics") return [
        [
            ...base,
            "-p",
            "rustzen-admin",
            "--no-default-features",
            "--features",
            "analytics-distribution",
            "--bin",
            "rz-admin",
        ],
        [...base, "-p", "rustzen-insights", "--no-default-features", "--features", "selected-distribution", "--bin", "rz-insights"],
    ];
    const notify = selected.preset === "monitor-notify";
    return [
        [...base, "-p", "rustzen-admin", "--no-default-features", "--features", notify ? "monitor-distribution,notifications" : "monitor-distribution"],
        [...base, "-p", "rustzen-monitor", "--no-default-features", "--features", notify ? "notifications" : "controller", "--bin", "rz-monitor"],
        [...base, "-p", "rustzen-monitor", "--no-default-features", "--features", "agent", "--bin", "rz-monitor-agent"],
    ];
}

export const syntheticServerInventory = (plan: SourceBuildPlan) =>
    selectedServerInventory(selectedServerSyntheticExportPlan(plan));
