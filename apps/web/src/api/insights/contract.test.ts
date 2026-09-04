import { expect, test } from "bun:test";

import { insightsAPIContract } from "./contract";

test("insights transport inventory maps every protected ModuleRouter route", () => {
    expect(Object.values(insightsAPIContract)).toEqual([
        { method: "GET", path: "/api/insights/collection-policy" },
        { method: "PUT", path: "/api/insights/collection-policy" },
        { method: "GET", path: "/api/insights/overview" },
        { method: "GET", path: "/api/insights/events" },
    ]);
});

test("insights transport inventory excludes public tracker routes", () => {
    const contractRoutes = new Set(
        Object.values(insightsAPIContract).map((route) => `${route.method} ${route.path}`),
    );

    expect(contractRoutes.has("GET /api/insights/tracker.js")).toBe(false);
    expect(contractRoutes.has("POST /api/insights/track")).toBe(false);
});
