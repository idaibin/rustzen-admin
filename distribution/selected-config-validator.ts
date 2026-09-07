import goldenOwners from "./fixtures/monitor-config-owners.json";
import { canonicalJson } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";

export type SelectedConfigContract = {
    artifactClass: "server" | "node-agent";
    compositionId: string;
    preset: "monitor" | "monitor-notify" | "node-agent";
    owners: Record<string, unknown>;
};

export function completeSelectedConfigForTest(
    selectionInput: unknown,
): SelectedConfigContract {
    const plan = supportedPlan(selectionInput);
    return {
        artifactClass: plan.artifactClass,
        compositionId: plan.compositionId,
        preset: plan.preset as "monitor" | "monitor-notify" | "node-agent",
        owners:
            plan.artifactClass === "server"
                ? {
                      access: structuredClone(goldenOwners.access),
                      monitor: structuredClone(goldenOwners.monitor),
                  }
                : {
                      "monitor-agent": structuredClone(
                          goldenOwners["monitor-agent"],
                      ),
                  },
    };
}

export function parseSelectedConfig(
    value: unknown,
    selectionInput: unknown,
): SelectedConfigContract {
    const expected = completeSelectedConfigForTest(selectionInput);
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected config artifact must be an object");
    if (canonicalJson(value) !== canonicalJson(expected))
        throw new Error(
            "selected config artifact differs from reviewed descriptors",
        );
    return expected;
}

export function parseSelectedConfigBytes(
    bytes: Uint8Array,
    selectionInput: unknown,
) {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    let value: unknown;
    try {
        value = JSON.parse(text);
    } catch {
        throw new Error("selected config artifact is not JSON");
    }
    const contract = parseSelectedConfig(value, selectionInput);
    if (text !== canonicalJson(contract))
        throw new Error("selected config artifact is not canonical");
    return contract;
}

function supportedPlan(selectionInput: unknown) {
    const plan = resolveSelection(selectionInput);
    const valid =
        plan.artifactClass === "server"
            ? (plan.preset === "monitor" || plan.preset === "monitor-notify") &&
              canonicalJson(plan.configOwners) ===
                  canonicalJson(["access", "monitor"])
            : plan.artifactClass === "node-agent" &&
              plan.preset === "node-agent" &&
              canonicalJson(plan.configOwners) ===
                  canonicalJson(["monitor-agent"]);
    if (!valid)
        throw new Error(
            "selected config supports only monitor/monitor-notify server or node-agent",
        );
    return plan;
}
