import goldenOwners from "./fixtures/monitor-api-owners.json";
import analyticsInsightsOwner from "./fixtures/analytics-api-owner.json";
import notificationOwner from "./fixtures/monitor-notify-api-owner.json";
import notificationMonitorOwner from "./fixtures/monitor-notify-monitor-api-owner.json";
import { canonicalJson } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import {
    isExactSupportedPlan,
    type SourceBuildPlan,
} from "./source-build-plan.ts";

type ApiOwner = {
    routes: Array<{
        access: string | { kind: string; capabilities?: string[] };
        method: string;
        path: string;
        operation?: string;
        permission?: string;
    }>;
    apiPrefix?: string;
    contractVersion?: number;
    menus?: Array<{ code: string; path: string; permission: string }>;
    module?: string;
    name?: string;
    version?: number;
};

export type SelectedApiContract = {
    compositionId: string;
    preset: "analytics" | "monitor" | "monitor-notify";
    owners: Record<string, ApiOwner>;
};

/** The API producer is intentionally limited to the reviewed server closures. */
export const supportsSelectedApiContract = (plan: SourceBuildPlan): boolean =>
    isExactSupportedPlan(plan, ["analytics", "monitor", "monitor-notify"]);

export function completeSelectedApiContractForTest(
    selectionInput: unknown,
): SelectedApiContract {
    const selection = supportedSelection(selectionInput);
    const owners = (
        selection.preset === "analytics"
            ? {
                  admin: structuredClone(goldenOwners.admin),
                  insights: structuredClone(analyticsInsightsOwner),
              }
            : structuredClone(goldenOwners)
    ) as Record<string, ApiOwner>;
    if (selection.preset === "monitor-notify") {
        owners.monitor = structuredClone(notificationMonitorOwner);
        owners.notifications = structuredClone(notificationOwner);
    }
    return {
        compositionId: selection.compositionId,
        preset: selection.preset,
        owners,
    };
}

export function parseSelectedApiContract(
    value: unknown,
    selectionInput: unknown,
): SelectedApiContract {
    const selection = supportedSelection(selectionInput);
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected API artifact must be an object");
    const record = value as Record<string, unknown>;
    if (
        canonicalJson(Object.keys(record).sort()) !==
        canonicalJson(["compositionId", "owners", "preset"])
    )
        throw new Error("selected API artifact fields are invalid");
    if (
        record.compositionId !== selection.compositionId ||
        record.preset !== selection.preset
    )
        throw new Error("selected API artifact selection mismatch");
    const expected = completeSelectedApiContractForTest(selectionInput);
    if (canonicalJson(record.owners) !== canonicalJson(expected.owners))
        throw new Error("selected API owners differ from reviewed baseline");
    return {
        compositionId: selection.compositionId,
        preset: expected.preset,
        owners: structuredClone(expected.owners),
    };
}

export function parseSelectedApiBytes(
    bytes: Uint8Array,
    selectionInput: unknown,
): SelectedApiContract {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    let value: unknown;
    try {
        value = JSON.parse(text);
    } catch {
        throw new Error("selected API artifact is not JSON");
    }
    const contract = parseSelectedApiContract(value, selectionInput);
    if (text !== canonicalJson(contract))
        throw new Error("selected API artifact is not canonical");
    return contract;
}

function supportedSelection(selectionInput: unknown) {
    const selection = resolveSelection(selectionInput);
    const expectedOwners =
        selection.preset === "analytics"
            ? ["admin", "insights"]
            : selection.preset === "monitor"
            ? ["admin", "monitor"]
            : selection.preset === "monitor-notify"
              ? [
                    "admin",
                    "admin-notifications",
                    "monitor",
                    "monitor-notifications",
                ]
              : null;
    if (
        !supportsSelectedApiContract(selection) ||
        expectedOwners === null ||
        selection.artifactClass !== "server" ||
        canonicalJson(selection.schemaOwners) !==
            canonicalJson(expectedOwners)
    )
        throw new Error("selected API contract supports only analytics or monitor server compositions");
    return {
        ...selection,
        preset: selection.preset as "analytics" | "monitor" | "monitor-notify",
    };
}
