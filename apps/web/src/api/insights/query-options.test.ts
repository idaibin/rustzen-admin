import { expect, test } from "bun:test";

import { insightsAPI } from "./api";
import { insightsQueryKeys, insightsQueryOptions } from "./query-options";

test("collection policy query uses the manage-only Insights read key", () => {
    expect(insightsQueryKeys.collectionPolicy()).toEqual(["insights", "collection-policy"]);
    expect(insightsQueryOptions.collectionPolicy()).toMatchObject({
        queryKey: ["insights", "collection-policy"],
        staleTime: 30_000,
    });
});

test("collection policy adapter reads the manage-only GET route", async () => {
    const originalFetch = globalThis.fetch;
    const calls: Array<[string, RequestInit | undefined]> = [];
    globalThis.fetch = (async (url, init) => {
        const requestUrl =
            typeof url === "string" ? url : url instanceof URL ? url.toString() : url.url;
        calls.push([requestUrl, init]);
        return new Response(
            JSON.stringify({
                code: 0,
                message: "Success",
                data: {
                    collectionEnabled: true,
                    projectConfigured: true,
                    allowedOrigins: ["https://app.example"],
                },
            }),
            { status: 200, headers: { "content-type": "application/json" } },
        );
    }) as typeof fetch;
    try {
        const policy = await insightsAPI.collectionPolicy();
        expect(policy).toEqual({
            collectionEnabled: true,
            projectConfigured: true,
            allowedOrigins: ["https://app.example"],
        });
        expect(calls).toHaveLength(1);
        expect(calls[0]?.[0]).toBe("/api/insights/collection-policy");
        expect(calls[0]?.[1]?.method).toBe("GET");
    } finally {
        globalThis.fetch = originalFetch;
    }
});
