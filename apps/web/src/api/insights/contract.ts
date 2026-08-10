import type { ModuleApiRoute } from "@/api/module-contract";

export const insightsAPIContract = {
    overview: { method: "GET", path: "/api/insights/overview" },
    events: { method: "GET", path: "/api/insights/events" },
    collectionPolicy: { method: "GET", path: "/api/insights/collection-policy" },
    updateCollectionPolicy: { method: "PUT", path: "/api/insights/collection-policy" },
} as const satisfies Record<string, ModuleApiRoute>;
