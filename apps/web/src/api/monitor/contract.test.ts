import { expect, test } from "bun:test";

import { monitorAPIContract } from "./contract";

test("monitor client declares only the controller monitoring routes", () => {
    expect(Object.values(monitorAPIContract).map((route) => route.path)).toEqual([
        "/api/monitor/overview",
        "/api/monitor/nodes",
        "/api/monitor/nodes/{node_id}",
        "/api/monitor/nodes/{node_id}/metrics",
        "/api/monitor/nodes/{node_id}/alert-settings",
        "/api/monitor/nodes/{node_id}/alert-settings",
        "/api/monitor/nodes/{node_id}/alert-settings",
        "/api/monitor/incidents",
        "/api/monitor/incidents/{id}",
        "/api/monitor/alert-settings",
        "/api/monitor/alert-settings",
        "/api/monitor/daily-summaries",
    ]);
    expect(Object.values(monitorAPIContract).every((route) => !route.path.includes("checks"))).toBe(
        true,
    );
});
