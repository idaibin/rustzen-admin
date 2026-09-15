import { canonicalJson } from "./release-manifest-core.ts";

export type SourceBuildPlan = {
    preset: string;
    artifactClass: "server" | "node-agent";
    compositionId: string;
    capabilities: string[];
};

type SupportedComposition = {
    preset: "monitor" | "monitor-notify" | "analytics" | "reports" | "node-agent";
    artifactClass: "server" | "node-agent";
    capabilities: string[];
    compositionId: string;
};

// These identities are reviewed producer contracts. Do not derive them from the
// mutable catalog: a catalog change must fail closed until every producer owner
// explicitly accepts the new closure and composition identity.
const supported: Record<SupportedComposition["preset"], SupportedComposition> = {
    monitor: {
        preset: "monitor",
        artifactClass: "server",
        capabilities: ["access", "monitor"],
        compositionId: "8957924886140f55fd0560d89f0c2acdac67cd95d14c09ac78d6f9fa18109d3b",
    },
    "monitor-notify": {
        preset: "monitor-notify",
        artifactClass: "server",
        capabilities: ["access", "monitor", "notifications"],
        compositionId: "0aac2acc2b282ed9f4c0e7b5ffff7866b78801b7cea1c77b273446128e86c36d",
    },
    analytics: {
        preset: "analytics",
        artifactClass: "server",
        capabilities: ["access", "insights"],
        compositionId: "62d09d09b3b0e94f88329179bf9a8c1fa84a984ba87df341911dab0e7fcf0a40",
    },
    reports: {
        preset: "reports",
        artifactClass: "server",
        capabilities: ["access", "reports"],
        compositionId: "e1037a3fe8a9c0829830eecc521570ef0b0c26bc030f7f6ee98a55bfff81cb56",
    },
    "node-agent": {
        preset: "node-agent",
        artifactClass: "node-agent",
        capabilities: ["monitor-agent"],
        compositionId: "5e526bfcee61a36b2c574c4b9cdf5e213445a1c796a06aeb84fbf4bdf012fc1c",
    },
};

export function isExactSupportedPlan(
    plan: SourceBuildPlan,
    presets: Array<keyof typeof supported>,
): boolean {
    const expected = presets.map((preset) => supported[preset]).find((candidate) =>
        candidate.preset === plan.preset &&
        candidate.artifactClass === plan.artifactClass &&
        candidate.compositionId === plan.compositionId &&
        canonicalJson(candidate.capabilities) === canonicalJson(plan.capabilities),
    );
    return expected !== undefined;
}

export function expectedComposition(
    preset: keyof typeof supported,
): SupportedComposition {
    return structuredClone(supported[preset]);
}
