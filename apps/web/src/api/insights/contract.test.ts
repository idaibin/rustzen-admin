import { expect, test } from "bun:test";

import { insightsAPIContract } from "./contract";

test("insights transport inventory maps every non-preflight ModuleRouter route", () => {
    expect(Object.values(insightsAPIContract)).toEqual([
        { method: "GET", path: "/api/insights/tracker.js" },
        { method: "POST", path: "/api/insights/track" },
        { method: "GET", path: "/api/insights/collection-policy" },
        { method: "PUT", path: "/api/insights/collection-policy" },
        { method: "GET", path: "/api/insights/overview" },
        { method: "GET", path: "/api/insights/events" },
    ]);
});
