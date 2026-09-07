import goldenOwners from "./fixtures/monitor-api-owners.json";
import notificationOwner from "./fixtures/monitor-notify-api-owner.json";
import { canonicalJson } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";

export type SelectedApiContract = {
    compositionId: string;
    preset: "monitor" | "monitor-notify";
    owners: typeof goldenOwners & { notifications?: typeof notificationOwner };
};

export function completeSelectedApiContractForTest(
    selectionInput: unknown,
): SelectedApiContract {
    const selection = supportedSelection(selectionInput);
    const owners = structuredClone(goldenOwners) as SelectedApiContract["owners"];
    if (selection.preset === "monitor-notify")
        owners.notifications = structuredClone(notificationOwner);
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
        selection.preset === "monitor"
            ? ["admin", "monitor"]
            : selection.preset === "monitor-notify"
              ? ["admin", "admin-notifications", "monitor"]
              : null;
    if (
        expectedOwners === null ||
        selection.artifactClass !== "server" ||
        canonicalJson(selection.schemaOwners) !==
            canonicalJson(expectedOwners)
    )
        throw new Error("selected API contract supports only monitor server compositions");
    return {
        ...selection,
        preset: selection.preset as "monitor" | "monitor-notify",
    };
}
