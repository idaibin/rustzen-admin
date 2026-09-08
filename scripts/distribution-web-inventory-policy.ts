import { lstat, readdir, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve } from "node:path";

import { allowedWebPackages } from "./distribution-web-allowed-packages.ts";

export type Inventory = {
    schemaVersion: number;
    preset: string;
    compositionId: string;
    generatedRoot: string;
    outputDirectory: string;
    selectedRoutes: string[];
    publicAssets: string[];
    emittedFiles: string[];
    moduleIds: string[];
};

export function parseInventory(value: unknown): Inventory {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected Web inventory must be an object");
    const record = value as Record<string, unknown>;
    const keys = [
        "schemaVersion",
        "preset",
        "compositionId",
        "generatedRoot",
        "outputDirectory",
        "selectedRoutes",
        "publicAssets",
        "emittedFiles",
        "moduleIds",
    ];
    if (Object.keys(record).length !== keys.length || keys.some((key) => !(key in record)))
        throw new Error("selected Web inventory has an invalid schema");
    if (
        record.schemaVersion !== 1 ||
        typeof record.preset !== "string" ||
        typeof record.compositionId !== "string" ||
        typeof record.generatedRoot !== "string" ||
        typeof record.outputDirectory !== "string" ||
        ![record.selectedRoutes, record.publicAssets, record.emittedFiles, record.moduleIds].every(
            (field) => Array.isArray(field) && field.every((item) => typeof item === "string"),
        )
    )
        throw new Error("selected Web inventory has invalid field types");
    return record as unknown as Inventory;
}

export function assertEqual(actual: string, expected: string, label: string) {
    if (actual !== expected) throw new Error(`selected Web inventory ${label} mismatch`);
}

export function assertArrayEqual(actual: string[], expected: string[], label: string) {
    if (JSON.stringify([...actual].sort()) !== JSON.stringify([...expected].sort()))
        throw new Error(`selected Web inventory ${label} mismatch`);
    if (new Set(actual).size !== actual.length)
        throw new Error(`selected Web inventory ${label} repeats entries`);
}

export function assertSafeRelativePath(path: string, label: string) {
    if (!path || isAbsolute(path) || path.split(/[\\/]/).some((part) => part === ".."))
        throw new Error(`selected Web inventory ${label} contains a path escape`);
}

export async function assertCanonicalPath(repositoryRoot: string, path: string, label: string) {
    const root = resolve(repositoryRoot);
    const expected = resolve(path);
    const relativePath = relative(root, expected);
    if (
        relativePath === "" ||
        isAbsolute(relativePath) ||
        relativePath.split(/[\\/]/).some((part) => part === "..")
    )
        throw new Error(`selected Web ${label} escapes the repository root`);
    const rootState = await lstat(root);
    if (rootState.isSymbolicLink())
        throw new Error("selected Web repository root must not be a symlink");
    let current = root;
    for (const part of relativePath.split(/[\\/]/)) {
        current = join(current, part);
        const state = await lstat(current);
        if (state.isSymbolicLink())
            throw new Error(`selected Web ${label} contains a symlink: ${current}`);
    }
    if ((await realpath(expected)) !== expected)
        throw new Error(`selected Web ${label} is not at its canonical path`);
}

export async function listFiles(
    repositoryRoot: string,
    directory: string,
    root = directory,
): Promise<string[]> {
    await assertCanonicalPath(repositoryRoot, directory, "output traversal");
    const entries = await readdir(directory, { withFileTypes: true });
    const nested = await Promise.all(
        entries.map(async (entry) => {
            const path = join(directory, entry.name);
            const state = await lstat(path);
            if (entry.isSymbolicLink() || state.isSymbolicLink())
                throw new Error(`selected Web output contains symlink: ${path}`);
            if (entry.isDirectory()) return listFiles(repositoryRoot, path, root);
            if (!entry.isFile())
                throw new Error(`selected Web output contains unsupported entry: ${path}`);
            return [relative(root, path).replaceAll("\\", "/")];
        }),
    );
    return nested.flat().sort();
}

export function assertModuleIds(
    moduleIds: string[],
    compositionId: string,
    hasNotifications: boolean,
) {
    if (new Set(moduleIds).size !== moduleIds.length)
        throw new Error("selected Web module IDs repeat");
    const generatedPrefix = `apps/web/.selected-web/${compositionId}/`;
    const allowedSourceFiles = new Set([
        "apps/web/src/api/module-contract.ts",
        "apps/web/src/api/request.ts",
        "apps/web/src/api/runtime.ts",
        "apps/web/src/components/language-switch.tsx",
        "apps/web/src/components/theme-provider.tsx",
    ]);
    const allowedSourceDirectories = [
        "apps/web/src/api/monitor/",
        ...(hasNotifications
            ? ["apps/web/src/api/notifications/", "apps/web/src/notifications/"]
            : []),
        "apps/web/src/assets/",
        "apps/web/src/components/auth/",
        "apps/web/src/components/feedback/",
        "apps/web/src/components/page/",
        "apps/web/src/components/table/",
        "apps/web/src/components/user/",
        "apps/web/src/constant/",
        "apps/web/src/hooks/",
        "apps/web/src/lib/",
    ];
    const allowedNodeModules = allowedWebPackages();
    let generatedCount = 0;
    for (const rawId of moduleIds) {
        const id = rawId.split("?", 1)[0];
        if (
            !id ||
            id.includes("\\") ||
            isAbsolute(id) ||
            id.startsWith(".") ||
            id.split("/").some((part) => part === "..")
        )
            throw new Error(`selected Web module inventory contains an unsafe module ID: ${rawId}`);
        if (id.startsWith("\0")) {
            if (!["\0vite/preload-helper.js", "\0vite/modulepreload-polyfill.js"].includes(id))
                throw new Error(
                    `selected Web module inventory contains an unknown virtual ID: ${rawId}`,
                );
            continue;
        }
        if (id.startsWith(generatedPrefix)) {
            generatedCount += 1;
            continue;
        }
        if (id.startsWith("apps/web/.selected-web/"))
            throw new Error(
                `selected Web module inventory contains another composition source: ${rawId}`,
            );
        if (
            allowedSourceFiles.has(id) ||
            allowedSourceDirectories.some((directory) => id.startsWith(directory))
        )
            continue;
        const nodeModulesPrefix = "apps/web/node_modules/";
        if (id.startsWith(nodeModulesPrefix)) {
            const parts = id.slice(nodeModulesPrefix.length).split("/");
            const packageName = parts[0]?.startsWith("@")
                ? `${parts[0]}/${parts[1] ?? ""}`
                : parts[0];
            if (packageName && allowedNodeModules.has(packageName)) continue;
            throw new Error(
                `selected Web module inventory contains an unknown node_modules package: ${rawId}`,
            );
        }
        throw new Error(
            `selected Web module inventory contains an unclassified module ID: ${rawId}`,
        );
    }
    if (!generatedCount)
        throw new Error("selected Web module inventory has no generated route source");
}

export async function assertSelectedApiSource(apiPath: string) {
    const apiSource = await Bun.file(apiPath).text();
    if (/\/(?:api)\/\s*["'`]\s*\+/.test(apiSource))
        throw new Error("selected Web API adapter dynamically constructs an API namespace");
    if (/(?:insights|reports|manage|system\/status)/.test(apiSource))
        throw new Error("selected Web API adapter names an excluded capability");
}
