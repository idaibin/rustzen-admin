import { lstat, readdir, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve } from "node:path";

import { allowedWebPackages } from "./distribution-web-allowed-packages.ts";
import type { WebFile } from "../distribution/selected-web-binding.ts";
export {
    assertSafeRelativePath,
    parseInventory,
    type Inventory,
} from "./distribution-web-inventory-schema.ts";
import { assertSafeRelativePath, type Inventory } from "./distribution-web-inventory-schema.ts";

export function assertEqual(actual: string, expected: string, label: string) {
    if (actual !== expected) throw new Error(`selected Web inventory ${label} mismatch`);
}

export function assertArrayEqual(actual: string[], expected: string[], label: string) {
    if (JSON.stringify([...actual].sort()) !== JSON.stringify([...expected].sort()))
        throw new Error(`selected Web inventory ${label} mismatch`);
    if (new Set(actual).size !== actual.length)
        throw new Error(`selected Web inventory ${label} repeats entries`);
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

export function assertModuleIds(moduleIds: string[], compositionId: string, preset: string) {
    const hasNotifications = preset === "monitor-notify";
    const isAnalytics = preset === "analytics";
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
        "apps/web/src/api/installation/",
        ...(!isAnalytics ? ["apps/web/src/api/monitor/"] : []),
        ...(isAnalytics ? ["apps/web/src/api/insights/"] : []),
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
        if (!hasNotifications && notificationOnlyMonitorOwners.has(id))
            throw new Error(
                `pure selected Web module inventory contains a notification-only Monitor owner: ${rawId}`,
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
    for (const owner of requiredModuleOwners(preset)) {
        if (!moduleIds.some((rawId) => rawId.split("?", 1)[0] === owner))
            throw new Error(`selected Web module inventory is missing required owner: ${owner}`);
    }
}

const notificationOnlyMonitorOwners = new Set([
    "apps/web/src/api/monitor/api.ts",
    "apps/web/src/api/monitor/notification-contract.ts",
]);

export function requiredModuleOwners(preset: string) {
    const hasNotifications = preset === "monitor-notify";
    return [
        "apps/web/src/api/installation/api.ts",
        ...(preset === "analytics"
            ? []
            : [
                  hasNotifications
                      ? "apps/web/src/api/monitor/api.ts"
                      : "apps/web/src/api/monitor/core-api.ts",
              ]),
        "apps/web/src/api/request.ts",
        ...(hasNotifications ? ["apps/web/src/api/notifications/api.ts"] : []),
    ];
}

/** Shared selected-Monitor inventory policy for producer verification and exported snapshots. */
export function assertSelectedWebSnapshot(
    inventory: Inventory,
    selection: { preset: string; compositionId: string; webRoots: string[] },
    emittedFiles: WebFile[],
    outputText: string,
) {
    const routes = selectedWebRoutes(selection);
    const hasNotifications = selection.preset === "monitor-notify";
    const isAnalytics = selection.preset === "analytics";
    const forbidden = [
        ...(!isAnalytics
            ? ["/api/insights", "/analytics/"]
            : [
                  "/api/monitor",
                  "/monitoring/",
                  "/api/insights/collection-policy",
                  "/api/insights/track",
                  "/api/insights/tracker.js",
              ]),
        "/api/reports",
        "/api/manage",
        "/api/system/status",
        "/reports/",
        "/manage/deploy",
        "/manage/task",
        "/manage/log",
        "ReactQueryDevtools",
        "TanStackRouterDevtools",
        ...(!hasNotifications
            ? ["/api/notifications", "Message center", "消息中心", "__rustzen_admin_marker__.json"]
            : []),
    ];
    const required = [
        "/api/auth/login",
        "/api/auth/me",
        ...(isAnalytics
            ? ["/api/insights/overview", "/api/insights/events", "/analytics/overview"]
            : ["/api/monitor/", "/monitoring/overview"]),
        "/api/system/users",
        "/api/system/roles",
        "/api/system/menus/options",
        ...(hasNotifications
            ? ["/api/notifications/stream", "/api/notifications/unread-count", "Message center"]
            : []),
    ];

    assertEqual(inventory.preset, selection.preset, "preset");
    assertEqual(inventory.compositionId, selection.compositionId, "compositionId");
    assertEqual(
        inventory.generatedRoot,
        `apps/web/.selected-web/${selection.compositionId}`,
        "generatedRoot",
    );
    assertEqual(
        inventory.outputDirectory,
        `target/distributions/${selection.compositionId}/web/dist`,
        "outputDirectory",
    );
    assertArrayEqual(inventory.selectedRoutes, routes, "selectedRoutes");
    assertArrayEqual(inventory.publicAssets, ["rustzen.png"], "publicAssets");
    const emittedPaths = emittedFiles.map((file) => file.path);
    assertArrayEqual(inventory.emittedFiles, emittedPaths, "emittedFiles");
    const actualFiles = emittedFiles.map(({ path, size, sha256 }) => ({ path, size, sha256 }));
    if (JSON.stringify(inventory.fileInventory) !== JSON.stringify(actualFiles))
        throw new Error("selected Web inventory file digest mismatch");
    if (inventory.binding.compositionId !== selection.compositionId)
        throw new Error("selected Web inventory binding composition mismatch");
    assertModuleIds(inventory.moduleIds, selection.compositionId, selection.preset);
    for (const asset of inventory.publicAssets) {
        if (!inventory.emittedFiles.includes(asset) || !emittedPaths.includes(asset))
            throw new Error(`selected Web public asset is absent from emitted output: ${asset}`);
    }
    for (const value of forbidden) {
        if (outputText.includes(value))
            throw new Error(`selected Web output contains excluded text: ${value}`);
    }
    for (const value of required) {
        if (!outputText.includes(value))
            throw new Error(`selected Web output is missing required text: ${value}`);
    }
}

export function selectedWebRoutes(selection: { webRoots: string[] }) {
    const monitor = selection.webRoots.some((route) => route.includes("/monitoring"));
    return [
        "index.tsx",
        ...selection.webRoots.map((route) => route.replace("apps/web/src/routes/", "")),
        ...(monitor
            ? [
                  "monitoring/-global-alert-settings.tsx",
                  "monitoring/-incident-drawer.tsx",
                  "monitoring/-node-details.tsx",
                  "monitoring/-node-alert-policy.tsx",
                  "monitoring/-node-onboarding.tsx",
                  "monitoring/-save-state.ts",
              ]
            : ["analytics/-event-target.ts"]),
        "system/-role-actions.tsx",
        "system/-role-delete-state.ts",
        "system/-role-dialog.tsx",
        "system/-role-permission-picker.tsx",
        "system/-user-actions.tsx",
        "system/-user-dialog.tsx",
    ].sort();
}
