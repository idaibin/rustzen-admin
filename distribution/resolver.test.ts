import { describe, expect, test } from "bun:test";
import { existsSync } from "node:fs";
import { resolve } from "node:path";

import { distributionCatalog, resolveSelection, validateCatalog } from "./resolver.ts";

describe("distribution selection", () => {
    test("monitor resolves to access and monitor only", () => {
        expect(resolveSelection({ preset: "monitor" }).capabilities).toEqual(["access", "monitor"]);
    });

    test("monitor-notify selects its notification pressure configuration owner", () => {
        expect(resolveSelection({ preset: "monitor-notify" }).configOwners).toEqual([
            "access",
            "monitor",
            "notifications",
        ]);
    });

    test("selected Web roots exist", () => {
        for (const root of resolveSelection({ preset: "monitor" }).webRoots) {
            expect(existsSync(resolve(import.meta.dir, "..", root))).toBeTrue();
        }
    });

    test("full is the exact declared production closure", () => {
        expect(resolveSelection({ preset: "full" }).capabilities).toEqual([
            "access",
            "admin-console",
            "insights",
            "log-console",
            "monitor",
            "notifications",
            "release-ui",
            "reports",
            "task-admin",
        ]);
    });

    test("full cannot omit a server capability from the catalog", () => {
        const incomplete = structuredClone(distributionCatalog);
        incomplete.presets.full.capabilities = incomplete.presets.full.capabilities.filter(
            (capability) => capability !== "notifications",
        );
        expect(() => validateCatalog(incomplete)).toThrow("every server capability");
    });

    test("rejects unknown catalog fields", () => {
        const malformed = structuredClone(distributionCatalog) as unknown as Record<
            string,
            unknown
        >;
        malformed.unexpected = true;
        expect(() => validateCatalog(malformed as typeof distributionCatalog)).toThrow(
            "unknown catalog field",
        );
    });

    test("custom resolves dependencies and produces a sorted closure", () => {
        const plan = resolveSelection({
            preset: "custom",
            capabilities: ["reports", "admin-console"],
        });
        expect(plan.capabilities).toEqual(["access", "admin-console", "reports"]);
        expect(plan.services).toEqual(["admin", "reports"]);
    });

    test("does not mix the node-agent and server classes", () => {
        expect(() =>
            resolveSelection({
                preset: "custom",
                artifactClass: "server",
                capabilities: ["monitor-agent"],
            }),
        ).toThrow("not valid for server");
        expect(() =>
            resolveSelection({
                preset: "custom",
                artifactClass: "node-agent",
                capabilities: ["monitor-agent", "access"],
            }),
        ).toThrow("not valid for node-agent");
    });

    test("rejects unknown selection fields and dependency cycles", () => {
        expect(() => resolveSelection({ preset: "monitor", unexpected: true })).toThrow(
            "unknown selection field",
        );
        const cyclic = structuredClone(distributionCatalog);
        const access = cyclic.capabilities.find((capability) => capability.id === "access");
        if (!access) throw new Error("test catalog is missing access");
        access.dependsOn = ["monitor"];
        expect(() => validateCatalog(cyclic)).toThrow("dependency cycle");
    });

    test("keeps the incomplete regression fixture test-only", () => {
        expect(() =>
            resolveSelection({ preset: "current-full-regression", releaseClass: "production" }),
        ).toThrow("fixed artifactClass and releaseClass");
        expect(() => resolveSelection({ preset: "full", capabilities: ["access"] })).toThrow(
            "named presets do not accept explicit capabilities",
        );
    });

    test("composition identity ignores preset aliases and source version", () => {
        const preset = resolveSelection({ preset: "monitor", sourceVersion: "old-source" });
        const custom = resolveSelection({
            preset: "custom",
            capabilities: ["monitor"],
            sourceVersion: "new-source",
        });
        expect(custom.compositionId).toBe(preset.compositionId);
        expect(resolveSelection({ preset: "monitor" })).toEqual(
            resolveSelection({ preset: "monitor" }),
        );
        expect(
            resolveSelection({ preset: "monitor", target: "aarch64-unknown-linux-gnu" })
                .compositionId,
        ).toBe(preset.compositionId);
    });

    test("release gate remains closed until producer verification exists", () => {
        expect(resolveSelection({ preset: "monitor" }).producerReadiness.ready).toBeFalse();
    });

    test("node-agent preset contains only the collector", () => {
        const plan = resolveSelection({ preset: "node-agent" });
        expect(plan.capabilities).toEqual(["monitor-agent"]);
        expect(plan.webRoots).toEqual([]);
        expect(plan.schemaOwners).toEqual([]);
    });

    test("retains all selected blockers when capabilities share a binary", () => {
        for (const preset of ["monitor-notify", "full"]) {
            const plan = resolveSelection({ preset });
            for (const capability of distributionCatalog.capabilities) {
                if (!plan.capabilities.includes(capability.id)) continue;
                for (const target of capability.packageTargets)
                    expect(plan.producerReadiness.blockers).toContain(target.reason);
            }
        }
    });
});
