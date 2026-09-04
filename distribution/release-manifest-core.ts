import { resolveSelection } from "./resolver.ts";
import type {
    BuildInputs,
    Digest,
    FileEntry,
} from "./release-manifest-types.ts";
const HEX = /^[a-f0-9]{64}$/;
/** Canonical JSON uses UTF-16 code-unit key ordering, matching ECMAScript/JCS. */
export function canonicalJson(value: unknown): string {
    if (value === null || typeof value === "boolean")
        return JSON.stringify(value);
    if (typeof value === "string") {
        assertScalarUnicode(value);
        return JSON.stringify(value);
    }
    if (typeof value === "number") {
        if (!Number.isSafeInteger(value))
            throw new Error("canonical JSON permits safe integers only");
        return String(value);
    }
    if (Array.isArray(value)) return `[${value.map(canonicalJson).join(",")}]`;
    if (!value || typeof value !== "object")
        throw new Error("canonical JSON rejects unsupported values");
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record);
    keys.forEach(assertScalarUnicode);
    return `{${keys
        .sort((a, b) => (a < b ? -1 : a > b ? 1 : 0))
        .map((k) => `${JSON.stringify(k)}:${canonicalJson(record[k])}`)
        .join(",")}}`;
}
export const sha256 = (bytes: string | Uint8Array): string =>
    new Bun.CryptoHasher("sha256").update(bytes).digest("hex");
export const validHash = (value: string): string => {
    if (!HEX.test(value)) throw new Error("value must be lowercase sha256");
    return value;
};
export const nonempty = (value: unknown, label: string): string => {
    if (typeof value !== "string" || !value)
        throw new Error(`${label} must be nonempty`);
    return value;
};
export const sortedStrings = (value: unknown, label: string): string[] => {
    if (
        !Array.isArray(value) ||
        value.some((item) => typeof item !== "string" || !item)
    )
        throw new Error(`${label} must be nonempty strings`);
    const result = [...value].sort();
    if (
        new Set(result).size !== result.length ||
        canonicalJson(result) !== canonicalJson(value)
    )
        throw new Error(`${label} must be sorted and unique`);
    return result as string[];
};
export const selectionDigest = (selection: unknown): Digest => ({
    sha256: sha256(canonicalJson(resolveSelection(selection))),
    source: "resolved-selection",
});
export const deriveBuildId = (
    selection: unknown,
    inputs: BuildInputs,
    apiDigest?: string,
): string =>
    sha256(
        canonicalJson({
            plan: resolveSelection(selection),
            releaseVersion: nonempty(inputs.releaseVersion, "releaseVersion"),
            sourceIdentity: nonempty(inputs.sourceIdentity, "sourceIdentity"),
            toolchain: nonempty(inputs.toolchain, "toolchain"),
            selectedRoutes: sortedStrings(
                inputs.selectedRoutes,
                "selectedRoutes",
            ),
            schemaDigest: validHash(inputs.schemaDigest),
            configDigest: validHash(inputs.configDigest),
            ...(apiDigest === undefined
                ? {}
                : { apiDigest: validHash(apiDigest) }),
            protocolId:
                inputs.protocolId === undefined
                    ? undefined
                    : validHash(inputs.protocolId),
        }),
    );
export const filesDigest = (files: FileEntry[]): string =>
    sha256(canonicalJson(files.map(({ path, sha256 }) => ({ path, sha256 }))));

function assertScalarUnicode(value: string): void {
    for (let index = 0; index < value.length; index++) {
        const unit = value.charCodeAt(index);
        if (unit >= 0xd800 && unit <= 0xdbff) {
            const next = value.charCodeAt(++index);
            if (!Number.isInteger(next) || next < 0xdc00 || next > 0xdfff)
                throw new Error("canonical JSON rejects isolated surrogate");
        } else if (unit >= 0xdc00 && unit <= 0xdfff)
            throw new Error("canonical JSON rejects isolated surrogate");
    }
}
