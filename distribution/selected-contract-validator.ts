import goldenOwners from "./fixtures/monitor-api-owners.json";
import { canonicalJson } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";

export type SelectedApiContract = {
    compositionId: string;
    preset: "monitor";
    owners: typeof goldenOwners;
};

export function completeSelectedApiContractForTest(
    selectionInput: unknown,
): SelectedApiContract {
    const selection = monitorSelection(selectionInput);
    return {
        compositionId: selection.compositionId,
        preset: "monitor",
        owners: structuredClone(goldenOwners),
    };
}

export function parseSelectedApiContract(
    value: unknown,
    selectionInput: unknown,
): SelectedApiContract {
    const selection = monitorSelection(selectionInput);
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
        record.preset !== "monitor"
    )
        throw new Error("selected API artifact selection mismatch");
    if (canonicalJson(record.owners) !== canonicalJson(goldenOwners))
        throw new Error("selected API owners differ from reviewed baseline");
    return {
        compositionId: selection.compositionId,
        preset: "monitor",
        owners: structuredClone(goldenOwners),
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

function monitorSelection(selectionInput: unknown) {
    const selection = resolveSelection(selectionInput);
    if (
        selection.preset !== "monitor" ||
        selection.artifactClass !== "server" ||
        canonicalJson(selection.schemaOwners) !==
            canonicalJson(["admin", "monitor"])
    )
        throw new Error("selected API contract supports only monitor server");
    return selection;
}
