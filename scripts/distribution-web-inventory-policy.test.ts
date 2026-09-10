import { describe, expect, test } from "bun:test";

import { allowedWebPackages } from "./distribution-web-allowed-packages";
import {
    assertArrayEqual,
    assertModuleIds,
    assertSafeRelativePath,
    parseInventory,
} from "./distribution-web-inventory-policy";
import { assertSelectedApiText } from "./distribution-selected-api-policy";

const compositionId = "1".repeat(64);
const digest = "2".repeat(64);
const inventory = {
    schemaVersion: 2,
    preset: "monitor-notify",
    compositionId,
    generatedRoot: `apps/web/.selected-web/${compositionId}`,
    outputDirectory: `target/distributions/${compositionId}/web/dist`,
    selectedRoutes: ["index.tsx"],
    publicAssets: ["rustzen.png"],
    emittedFiles: ["index.html"],
    fileInventory: [{ path: "index.html", size: 123, sha256: digest }],
    moduleIds: [`apps/web/.selected-web/${compositionId}/index.tsx`],
    binding: {
        bindingVersion: 1,
        compositionId,
        selectedApiDigest: digest,
        webDigest: "3".repeat(64),
    },
};

describe("selected Web inventory policy", () => {
    test("accepts the exact inventory schema and rejects extra fields", () => {
        expect(parseInventory(inventory)).toEqual(inventory);
        expect(() => parseInventory({ ...inventory, unexpected: true })).toThrow("invalid schema");
        expect(() => parseInventory({ ...inventory, schemaVersion: 1 })).toThrow("field types");
        expect(() => parseInventory({ ...inventory, binding: undefined })).toThrow();
        expect(() =>
            parseInventory({
                ...inventory,
                fileInventory: [...inventory.fileInventory, inventory.fileInventory[0]],
            }),
        ).toThrow("sorted and unique");
        expect(() =>
            parseInventory({
                ...inventory,
                fileInventory: [
                    { path: "z.js", size: 1, sha256: digest },
                    { path: "a.js", size: 1, sha256: digest },
                ],
            }),
        ).toThrow("sorted and unique");
        expect(() =>
            parseInventory({
                ...inventory,
                fileInventory: [{ path: "index.html", size: 1, sha256: "wrong" }],
            }),
        ).toThrow("invalid file entry");
    });

    test("rejects path escapes and repeated inventory entries", () => {
        expect(() => assertSafeRelativePath("../outside", "fixture")).toThrow("path escape");
        expect(() => assertArrayEqual(["same", "same"], ["same", "same"], "fixture")).toThrow(
            "repeats entries",
        );
    });

    test("admits notification modules only for notification selections", () => {
        const pureIds = [
            "apps/web/.selected-web/selection-1/index.tsx",
            "apps/web/src/api/installation/api.ts",
            "apps/web/src/api/request.ts",
            "apps/web/src/api/monitor/core-api.ts",
            "apps/web/src/api/notifications/api.ts",
        ];
        expect(() => assertModuleIds(pureIds, "selection-1", "monitor")).toThrow("unclassified");
        expect(() =>
            assertModuleIds(
                [...pureIds, "apps/web/src/api/monitor/api.ts"],
                "selection-1",
                "monitor-notify",
            ),
        ).not.toThrow();
    });

    test("requires the API owner selected by the pure or notify template", () => {
        const base = (ids: string[]) => [
            `apps/web/.selected-web/${compositionId}/index.tsx`,
            "apps/web/src/api/installation/api.ts",
            "apps/web/src/api/request.ts",
            ...ids,
        ];
        expect(() =>
            assertModuleIds(
                base(["apps/web/src/api/monitor/core-api.ts"]),
                compositionId,
                "monitor",
            ),
        ).not.toThrow();
        expect(() => assertModuleIds(base([]), compositionId, "monitor")).toThrow(
            "missing required owner",
        );
        expect(() =>
            assertModuleIds(
                base([
                    "apps/web/src/api/monitor/core-api.ts",
                    "apps/web/src/api/monitor/api.ts",
                    "apps/web/src/api/notifications/api.ts",
                ]),
                compositionId,
                "monitor-notify",
            ),
        ).not.toThrow();
        expect(() =>
            assertModuleIds(
                base([
                    "apps/web/src/api/monitor/core-api.ts",
                    "apps/web/src/api/notifications/api.ts",
                ]),
                compositionId,
                "monitor-notify",
            ),
        ).toThrow("missing required owner");
    });

    test("rejects notification-only Monitor owners from a complete pure selection", () => {
        const pure = [
            `apps/web/.selected-web/${compositionId}/index.tsx`,
            "apps/web/src/api/installation/api.ts",
            "apps/web/src/api/monitor/core-api.ts",
            "apps/web/src/api/request.ts",
        ];
        for (const owner of [
            "apps/web/src/api/monitor/api.ts",
            "apps/web/src/api/monitor/notification-contract.ts",
        ]) {
            expect(() => assertModuleIds([...pure, owner], compositionId, "monitor")).toThrow(
                "notification-only Monitor owner",
            );
        }
    });

    test("keeps the package allowlist explicit", () => {
        expect(allowedWebPackages()).toContain("@tanstack/react-query");
        expect(allowedWebPackages()).not.toContain("axios");
    });

    test("locks the Analytics adapter to two static GET routes", () => {
        const exact = `const insightsAPI = {\n    overview: () => apiRequest<X>({ url: "/api/insights/overview", method: "GET" }),\n    events: () => apiRequest<X>({ url: "/api/insights/events", method: "GET" }),\n};`;
        expect(() => assertSelectedApiText(exact, "analytics")).not.toThrow();
        for (const mutation of [
            exact.replace(
                "\n};",
                '\n    export: () => apiRequest<X>({ url: "/api/insights/export", method: "GET" }),\n};',
            ),
            exact.replace('method: "GET"', 'method: "DELETE"'),
            exact.replace('url: "/api/insights/events"', "url: `/api/insights/${suffix}`"),
            `const write = () => apiRequest({ url: "/api/insights/export", method: "POST" });\n${exact.replace("\n};", "\n    export: write,\n};")}`,
            `const writeApi = {};\n${exact.replace("{\n", "{\n    ...writeApi,\n")}`,
            `const writeApi = () => {};\n${exact.replace("{\n", "{\n    writeApi,\n")}`,
            exact.replace("{\n", "{\n    writeApi() {},\n"),
            `const writeApi = () => {};\n${exact.replace("{\n", "{\n     writeApi,\n")}`,
            `const writeApi = () => {};\n${exact.replace("{\n", "{\n\twriteApi,\n")}`,
        ])
            expect(() => assertSelectedApiText(mutation, "analytics")).toThrow();
    });
});
