import { expect, test } from "bun:test";

import { monitorAPI } from "./api";
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

test("Monitoring route reads preserve typed HTTP failures for route-local states", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () =>
        new Response(JSON.stringify({ code: 40002, message: "fixture forbidden", data: null }), {
            status: 403,
            headers: { "content-type": "application/json" },
        })) as unknown as typeof fetch;
    try {
        await expect(monitorAPI.overview()).rejects.toMatchObject({
            name: "ApiRequestError",
            status: 403,
        });
        await expect(monitorAPI.nodes()).rejects.toMatchObject({
            name: "ApiRequestError",
            status: 403,
        });
        await expect(monitorAPI.incidents({ current: 1, pageSize: 20 })).rejects.toMatchObject({
            name: "ApiRequestError",
            status: 403,
        });
        await expect(monitorAPI.dailySummaries({ current: 1, pageSize: 20 })).rejects.toMatchObject(
            {
                name: "ApiRequestError",
                status: 403,
            },
        );
    } finally {
        globalThis.fetch = originalFetch;
    }
});
