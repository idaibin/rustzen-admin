import { expect, test } from "bun:test";
import { resolveSelection } from "./resolver.ts";
import { expectedComposition } from "./source-build-plan.ts";
import { selectedServerInventory } from "./selected-server-inventory.ts";

const target = "x86_64-unknown-linux-musl";

test("selected server inventories remain exact and separate from Agent witnesses", () => {
    expect(selectedServerInventory(resolveSelection({ preset: "analytics", target }))).toEqual({
        binaries: ["bin/rz-admin", "bin/rz-insights"], hasAgentWitness: false,
    });
    for (const preset of ["monitor", "monitor-notify"]) {
        expect(selectedServerInventory(resolveSelection({ preset, target }))).toEqual({
            binaries: ["bin/rz-admin", "bin/rz-monitor"], hasAgentWitness: true,
        });
    }
    for (const preset of ["reports", "full", "node-agent"]) {
        expect(() => selectedServerInventory(resolveSelection({ preset, target }))).toThrow();
    }
});

test("selected server inventory fails closed when an accepted identity drifts", () => {
    const analytics = expectedComposition("analytics");
    for (const changed of [
        { ...analytics, preset: "monitor" },
        { ...analytics, artifactClass: "node-agent" as const },
        { ...analytics, compositionId: "0".repeat(64) },
        { ...analytics, capabilities: [...analytics.capabilities, "monitor"] },
        { ...analytics, capabilities: analytics.capabilities.slice().reverse() },
    ]) {
        expect(() => selectedServerInventory(changed)).toThrow(
            "does not support this exact closure",
        );
    }
});
