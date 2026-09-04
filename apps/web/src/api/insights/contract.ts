import type { ModuleApiRoute } from "@/api/module-contract";

export const insightsAPIContract = {
    trackerScript: { method: "GET", path: "/api/insights/tracker.js" },
    track: { method: "POST", path: "/api/insights/track" },
    collectionPolicy: { method: "GET", path: "/api/insights/collection-policy" },
    updateCollectionPolicy: { method: "PUT", path: "/api/insights/collection-policy" },
    overview: { method: "GET", path: "/api/insights/overview" },
    events: { method: "GET", path: "/api/insights/events" },
} as const satisfies Record<string, ModuleApiRoute>;
