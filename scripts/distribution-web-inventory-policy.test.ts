import { describe, expect, test } from "bun:test";

import { allowedWebPackages } from "./distribution-web-allowed-packages";
import {
    assertArrayEqual,
    assertModuleIds,
    assertSafeRelativePath,
    parseInventory,
} from "./distribution-web-inventory-policy";

const inventory = {
    schemaVersion: 1,
    preset: "monitor-notify",
    compositionId: "selection-1",
    generatedRoot: "apps/web/.selected-web/selection-1",
    outputDirectory: "target/distributions/selection-1/web/dist",
    selectedRoutes: ["index.tsx"],
    publicAssets: ["rustzen.png"],
    emittedFiles: ["index.html"],
    moduleIds: ["apps/web/.selected-web/selection-1/index.tsx"],
};

describe("selected Web inventory policy", () => {
    test("accepts the exact inventory schema and rejects extra fields", () => {
        expect(parseInventory(inventory)).toEqual(inventory);
        expect(() => parseInventory({ ...inventory, unexpected: true })).toThrow("invalid schema");
    });

    test("rejects path escapes and repeated inventory entries", () => {
        expect(() => assertSafeRelativePath("../outside", "fixture")).toThrow("path escape");
        expect(() => assertArrayEqual(["same", "same"], ["same", "same"], "fixture")).toThrow(
            "repeats entries",
        );
    });

    test("admits notification modules only for notification selections", () => {
        const ids = [
            "apps/web/.selected-web/selection-1/index.tsx",
            "apps/web/src/api/notifications/api.ts",
        ];
        expect(() => assertModuleIds(ids, "selection-1", false)).toThrow("unclassified");
        expect(() => assertModuleIds(ids, "selection-1", true)).not.toThrow();
    });

    test("keeps the package allowlist explicit", () => {
        expect(allowedWebPackages()).toContain("@tanstack/react-query");
        expect(allowedWebPackages()).not.toContain("axios");
    });
});
