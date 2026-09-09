import { describe, expect, test } from "bun:test";

import { compareManifestRoutes } from "./worker-contract-verifier.mjs";

const manifest = (routes, apiPrefix = "/api/insights") => ({ apiPrefix, routes });

describe("manifest/frontend route contract comparison", () => {
    test("requires every protected manifest route in the client contract", () => {
        const result = compareManifestRoutes(
            "insights",
            manifest([
                { method: "GET", path: "/overview", access: "protected" },
                { method: "GET", path: "/events", access: "protected" },
            ]),
            {
                overview: { method: "GET", path: "/api/insights/overview" },
                stale: { method: "GET", path: "/api/insights/stale" },
            },
        );

        expect(result.missing).toEqual([
            { key: "GET /api/insights/stale", name: "stale" },
        ]);
        expect(result.extra).toEqual([
            { key: "GET /api/insights/events", access: "protected" },
        ]);
    });

    test("allows only the explicit public tracker routes", () => {
        const result = compareManifestRoutes(
            "insights",
            manifest([
                { method: "GET", path: "/tracker.js", access: "public" },
                { method: "OPTIONS", path: "/track", access: "public" },
                { method: "POST", path: "/track", access: "public" },
                { method: "GET", path: "/overview", access: "protected" },
            ]),
            { overview: { method: "GET", path: "/api/insights/overview" } },
        );

        expect(result.extra).toEqual([]);
        expect(result.invalidAllowlist).toEqual([]);
        expect(result.clientPublicRoutes).toEqual([]);
    });

    test("allows the Monitor agent-report route as an explicit public application route", () => {
        const result = compareManifestRoutes(
            "monitor",
            manifest(
                [{ method: "POST", path: "/agent-reports", access: "public" }],
                "/api/monitor",
            ),
            {},
        );

        expect(result.extra).toEqual([]);
        expect(result.invalidAllowlist).toEqual([]);
        expect(result.clientPublicRoutes).toEqual([]);
    });

    test("rejects a public route that is not explicitly allowlisted", () => {
        const result = compareManifestRoutes(
            "insights",
            manifest([{ method: "GET", path: "/public-status", access: "public" }]),
            {},
        );

        expect(result.extra).toEqual([
            { key: "GET /api/insights/public-status", access: "public" },
        ]);
    });

    test("does not hide an unknown Monitor public route behind the agent-report route", () => {
        const result = compareManifestRoutes(
            "monitor",
            manifest(
                [{ method: "POST", path: "/public-status", access: "public" }],
                "/api/monitor",
            ),
            {},
        );

        expect(result.extra).toEqual([
            { key: "POST /api/monitor/public-status", access: "public" },
        ]);
    });

    test("reports agent-report access drift", () => {
        const result = compareManifestRoutes(
            "monitor",
            manifest(
                [{ method: "POST", path: "/agent-reports", access: "protected" }],
                "/api/monitor",
            ),
            {},
        );

        expect(result.invalidAllowlist).toEqual([
            { key: "POST /api/monitor/agent-reports", access: "protected" },
        ]);
        expect(result.clientPublicRoutes).toEqual([]);
    });

    test("does not let the handwritten client contract consume a public exception", () => {
        const result = compareManifestRoutes(
            "insights",
            manifest([{ method: "GET", path: "/tracker.js", access: "public" }]),
            { tracker: { method: "GET", path: "/api/insights/tracker.js" } },
        );

        expect(result.clientPublicRoutes).toEqual([
            { key: "GET /api/insights/tracker.js", name: "tracker" },
        ]);
    });
});

test("does not allowlist notification delivery for Monitor or Reports", () => {
    for (const [module, prefix] of [["monitor", "/api/monitor"], ["reports", "/api/reports"]]) {
        const result = compareManifestRoutes(module, manifest([{ method: "GET", path: "/notification-delivery", access: "public" }], prefix), {});
        expect(result.extra).toEqual([{ key: `GET ${prefix}/notification-delivery`, access: "public" }]);
    }
});
