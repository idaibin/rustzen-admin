import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

import { distributionCatalog, resolveSelection } from "./resolver.ts";
import {
    selectedCargoBuilds,
    selectedServiceCargoBuilds,
    supportsSelectedCargo,
    supportsSelectedServiceCargo,
} from "./selected-cargo-producer.ts";
import {
    assertCompleteReadinessInventory,
    auditSourceBuildReadiness,
} from "./source-build-readiness.ts";

const fixture = (preset: string) =>
    JSON.parse(readFileSync(join(import.meta.dir, "fixtures", `${preset}.json`), "utf8"));

describe("P8 source/build certification admission", () => {
    test("has a canonical fixture and explicit assessment for every named preset", () => {
        assertCompleteReadinessInventory();
        for (const preset of Object.keys(distributionCatalog.presets)) {
            const plan = resolveSelection(fixture(preset));
            expect(plan.preset).toBe(preset);
        }
    });

    test("admits only the three selections with complete source/build producer families", () => {
        for (const preset of ["monitor", "monitor-notify", "node-agent"]) {
            const audit = auditSourceBuildReadiness(fixture(preset));
            expect(audit.admissionReady).toBeTrue();
            expect(audit.missingProducers).toEqual([]);
            expect(audit.certified).toBeFalse();
        }
        expect(auditSourceBuildReadiness(fixture("node-agent")).requiredProducers).toEqual([
            "cargo",
            "config",
            "native-layout",
            "protocol",
        ]);
    });

    test("fails closed for incomplete official and test-only selections", () => {
        for (const preset of ["full", "analytics", "reports", "current-full-regression"]) {
            const audit = auditSourceBuildReadiness(fixture(preset));
            expect(audit.admissionReady).toBeFalse();
            expect(audit.blockers.length).toBeGreaterThan(0);
            expect(audit.certified).toBeFalse();
        }
        expect(auditSourceBuildReadiness(fixture("full")).missingProducers).toContain("web");
        expect(auditSourceBuildReadiness(fixture("monitor-notify")).missingProducers).toEqual([]);
    });

    test("tracks the Analytics service build without admitting the incomplete server family", () => {
        const plan = resolveSelection(fixture("analytics"));
        expect(supportsSelectedServiceCargo(plan)).toBeTrue();
        expect(selectedServiceCargoBuilds(plan)).toEqual([
            [
                "cargo",
                "build",
                "-p",
                "rustzen-insights",
                "--no-default-features",
                "--features",
                "selected-distribution",
                "--bin",
                "rz-insights",
            ],
        ]);
        expect(supportsSelectedCargo(plan)).toBeFalse();
        expect(() => selectedCargoBuilds(plan)).toThrow();
        const audit = auditSourceBuildReadiness(fixture("analytics"));
        expect(audit.availableProducers).toEqual(["web", "api", "schema", "config", "native-layout"]);
        expect(audit.missingProducers).toEqual(["cargo", "protocol"]);
        expect(audit.admissionReady).toBeFalse();
        const insights = distributionCatalog.capabilities.find(({ id }) => id === "insights");
        expect(insights?.packageTargets[0]?.reason).toContain(
            "Insights service Cargo, selected Admin/Insights schema, selected Web/API/config and native-layout producers are implemented",
        );
    });

    test("does not infer custom readiness from an identical capability closure", () => {
        const audit = auditSourceBuildReadiness({
            schemaVersion: 1,
            preset: "custom",
            artifactClass: "server",
            capabilities: ["access", "monitor", "notifications"],
            target: "x86_64-unknown-linux-musl",
        });
        expect(audit.compositionId).toBe(
            auditSourceBuildReadiness(fixture("monitor-notify")).compositionId,
        );
        expect(audit.admissionReady).toBeFalse();
        expect(audit.availableProducers).toEqual([]);
        expect(audit.blockers).toContain(
            "custom selections require an explicit composition-specific assessment",
        );
    });

    test("fails closed when a named composition no longer matches producer closure", () => {
        const catalog = structuredClone(distributionCatalog);
        catalog.presets.monitor.capabilities = ["access"];
        const audit = auditSourceBuildReadiness(fixture("monitor"), catalog);
        expect(audit.admissionReady).toBeFalse();
        expect(audit.missingProducers).toEqual([
            "cargo",
            "web",
            "api",
            "schema",
            "config",
            "native-layout",
            "protocol",
        ]);
    });

    test("fails closed when a producer receives a mutated composition identity", () => {
        const plan = resolveSelection(fixture("monitor"));
        expect(supportsSelectedCargo(plan)).toBeTrue();
        expect(supportsSelectedCargo({ ...plan, compositionId: "0".repeat(64) })).toBeFalse();
    });

    test("rejects a forged named catalog whose Agent closure changes", () => {
        const catalog = structuredClone(distributionCatalog);
        const agent = catalog.capabilities.find((capability) => capability.id === "monitor-agent");
        if (!agent) throw new Error("monitor-agent capability is required for this test");
        agent.dependsOn = ["agent-extra"];
        catalog.capabilities.push({ ...structuredClone(agent), id: "agent-extra", dependsOn: [] });
        expect(() => auditSourceBuildReadiness(fixture("node-agent"), catalog)).toThrow(
            "node-agent must resolve exactly monitor-agent",
        );
    });

    test("does not trust a same-name catalog preset with forged capabilities", () => {
        const catalog = structuredClone(distributionCatalog);
        catalog.presets["monitor-notify"].capabilities = ["access", "monitor"];
        const audit = auditSourceBuildReadiness(fixture("monitor-notify"), catalog);
        expect(audit.compositionId).toBe(
            auditSourceBuildReadiness(fixture("monitor")).compositionId,
        );
        expect(audit.admissionReady).toBeFalse();
        expect(audit.missingProducers).toContain("native-layout");
    });
});
