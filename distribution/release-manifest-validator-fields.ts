import { canonicalJson, nonempty, validHash } from "./release-manifest-core.ts";
import type { BinaryDigest, Digest, DigestSource, FileEntry } from "./release-manifest-types.ts";

function string(value: unknown, label: string): string {
    if (typeof value !== "string") throw new Error(`${label} must be a string`);
    return value;
}

export function object(value: unknown, label: string): Record<string, unknown> {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error(`${label} must be an object`);
    return value as Record<string, unknown>;
}
export function onlyKeys(record: Record<string, unknown>, keys: string[]) {
    const allowed = new Set(keys);
    for (const key of Object.keys(record))
        if (!allowed.has(key))
            throw new Error(`unknown manifest field: ${key}`);
}
export function required(value: unknown, label: string): string {
    if (value === undefined) throw new Error(`${label} is required`);
    return nonempty(value, label);
}
export function digestRecord(
    value: unknown,
    source: DigestSource,
    label: string,
): Digest {
    const record = object(value, label);
    onlyKeys(
        record,
        source === "binary-file"
            ? ["path", "sha256", "source"]
            : ["sha256", "source"],
    );
    if (record.source !== source)
        throw new Error(`${label} requires ${source} source`);
    const path =
        source === "binary-file"
            ? checkedPath(nonempty(record.path, `${label}.path`))
            : undefined;
    return {
        sha256: validHash(nonempty(record.sha256, `${label}.sha256`)),
        source,
        ...(path ? { path } : {}),
    };
}
export function binaryDigests(value: unknown): BinaryDigest[] {
    if (!Array.isArray(value) || !value.length)
        throw new Error("binaryDigests must be nonempty");
    const result = value.map((entry) =>
        digestRecord(entry, "binary-file", "binaryDigests"),
    );
    if (new Set(result.map((item) => item.path)).size !== result.length)
        throw new Error("binaryDigests repeat paths");
    return result as BinaryDigest[];
}
export function hashMap(value: unknown, label: string): Record<string, string> {
    const record = object(value, label);
    const keys = Object.keys(record).sort();
    if (!keys.length) throw new Error(`${label} must not be empty`);
    for (const key of keys) {
        if (!key) throw new Error(`${label} has empty owner`);
        validHash(nonempty(record[key], `${label}.${key}`));
    }
    return Object.fromEntries(keys.map((key) => [key, record[key] as string]));
}
export function fileEntries(value: unknown): FileEntry[] {
    if (!Array.isArray(value) || !value.length)
        throw new Error("files must be nonempty");
    const result = value
        .map((entry) => {
            const record = object(entry, "file");
            onlyKeys(record, ["path", "type", "mode", "size", "sha256"]);
            const path = checkedPath(nonempty(record.path, "file.path"));
            if (record.type !== "file")
                throw new Error("file type must be file");
            const mode = record.mode;
            if (path.startsWith("bin/") ? mode !== "0755" : mode !== "0644")
                throw new Error("file mode is invalid");
            if (
                typeof record.size !== "number" ||
                !Number.isSafeInteger(record.size) ||
                record.size < 0
            )
                throw new Error("file size is invalid");
            return {
                path,
                type: "file" as const,
                mode,
                size: record.size,
                sha256: validHash(nonempty(record.sha256, "file.sha256")),
            } as FileEntry;
        })
        .sort((left, right) =>
            left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
        );
    if (
        new Set(result.map((entry) => entry.path)).size !== result.length ||
        canonicalJson(result) !== canonicalJson(value)
    )
        throw new Error("files must be sorted and unique");
    return result;
}


export function checkedPath(path: string): string {
    if (
        path.includes("\\") ||
        path.includes("\0") ||
        path.startsWith("/") ||
        [
            "manifest.json",
            "release-manifest.json",
            "signature.json",
            "envelope.json",
            "signature-envelope.json",
        ].includes(path) ||
        path.split("/").some((part) => !part || part === "." || part === "..")
    )
        throw new Error("file path is not canonical POSIX");
    return path;
}
