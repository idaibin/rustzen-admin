import type { ModuleApiRoute } from "@/api/module-contract";

export const insightsAPIContract = {
    collectionPolicy: { method: "GET", path: "/api/insights/collection-policy" },
    updateCollectionPolicy: { method: "PUT", path: "/api/insights/collection-policy" },
    overview: { method: "GET", path: "/api/insights/overview" },
    events: { method: "GET", path: "/api/insights/events" },
} as const satisfies Record<string, ModuleApiRoute>;
