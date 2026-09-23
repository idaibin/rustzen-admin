import { isAbsolute } from "node:path";

import { parseWebBinding, type WebBinding } from "../distribution/selected-web-binding.ts";

export type Inventory = {
    schemaVersion: number;
    preset: string;
    compositionId: string;
    generatedRoot: string;
    outputDirectory: string;
    selectedRoutes: string[];
    publicAssets: string[];
    emittedFiles: string[];
    fileInventory: { path: string; size: number; sha256: string }[];
    moduleIds: string[];
    binding: WebBinding;
};

export function parseInventory(value: unknown): Inventory {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected Web inventory must be an object");
    const record = value as Record<string, unknown>;
    const keys = ["schemaVersion", "preset", "compositionId", "generatedRoot", "outputDirectory", "selectedRoutes", "publicAssets", "emittedFiles", "fileInventory", "moduleIds", "binding"];
    if (Object.keys(record).length !== keys.length || keys.some((key) => !(key in record)))
        throw new Error("selected Web inventory has an invalid schema");
    if (record.schemaVersion !== 2 || typeof record.preset !== "string" || typeof record.compositionId !== "string" || typeof record.generatedRoot !== "string" || typeof record.outputDirectory !== "string" || ![record.selectedRoutes, record.publicAssets, record.emittedFiles, record.moduleIds].every(strings))
        throw new Error("selected Web inventory has invalid field types");
    if (!Array.isArray(record.fileInventory))
        throw new Error("selected Web inventory has invalid file inventory");
    const fileInventory = record.fileInventory.map(fileEntry);
    if (new Set(fileInventory.map((file) => file.path)).size !== fileInventory.length || !isSorted(fileInventory.map((file) => file.path)))
        throw new Error("selected Web inventory file entries must be sorted and unique");
    return { ...(record as Omit<Inventory, "binding" | "fileInventory">), fileInventory, binding: parseWebBinding(record.binding) };
}

export function assertSafeRelativePath(path: string, label: string) {
    if (!path || isAbsolute(path) || path.split(/[\\/]/).some((part) => part === ".."))
        throw new Error(`selected Web inventory ${label} contains a path escape`);
}

function strings(value: unknown): value is string[] {
    return Array.isArray(value) && value.every((item) => typeof item === "string");
}
function fileEntry(value: unknown) {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected Web inventory has invalid file entry");
    const file = value as Record<string, unknown>;
    if (Object.keys(file).length !== 3 || typeof file.path !== "string" || typeof file.size !== "number" || !Number.isSafeInteger(file.size) || file.size < 0 || typeof file.sha256 !== "string" || !/^[a-f0-9]{64}$/.test(file.sha256))
        throw new Error("selected Web inventory has invalid file entry");
    assertSafeRelativePath(file.path, "fileInventory");
    return { path: file.path, size: file.size, sha256: file.sha256 };
}
function isSorted(paths: string[]) {
    return paths.every((path, index) => index === 0 || (paths[index - 1] ?? "") < path);
}
