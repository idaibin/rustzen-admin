import { join, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";
import {
    assertArrayEqual,
    assertCanonicalPath,
    assertEqual,
    assertModuleIds,
    assertSafeRelativePath,
    assertSelectedApiSource,
    listFiles,
    parseInventory,
} from "./distribution-web-inventory-policy.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath)
    throw new Error("usage: bun scripts/distribution-verify-web.ts --selection <selection.json>");

const selection = resolveSelection(await Bun.file(resolve(repositoryRoot, selectionPath)).json());
if (!["monitor", "monitor-notify"].includes(selection.preset))
    throw new Error("selected Web verifier currently supports only monitor compositions");
const hasNotifications = selection.capabilities.includes("notifications");

const distributionRoot = join(repositoryRoot, "target/distributions", selection.compositionId);
const outputRoot = join(distributionRoot, "web");
const distRoot = join(outputRoot, "dist");
const generatedDirectory = join(repositoryRoot, "apps/web/.selected-web", selection.compositionId);
const generatedApiSource = join(generatedDirectory, "api.ts");
const selectedApiSource = join(repositoryRoot, "apps/web/src/distribution/monitor-api.ts");
await Promise.all([
    assertCanonicalPath(repositoryRoot, distributionRoot, "distributionRoot"),
    assertCanonicalPath(repositoryRoot, outputRoot, "web outputRoot"),
    assertCanonicalPath(repositoryRoot, distRoot, "distRoot"),
    assertCanonicalPath(repositoryRoot, generatedDirectory, "generatedRoot"),
    assertCanonicalPath(repositoryRoot, generatedApiSource, "generated API source"),
    assertCanonicalPath(repositoryRoot, selectedApiSource, "selected API source"),
]);

const inventory = parseInventory(await Bun.file(join(outputRoot, "inventory.json")).json());
const expected = {
    preset: selection.preset,
    compositionId: selection.compositionId,
    generatedRoot: `apps/web/.selected-web/${selection.compositionId}`,
    outputDirectory: `target/distributions/${selection.compositionId}/web/dist`,
    selectedRoutes: [
        "index.tsx",
        ...selection.webRoots.map((route) => route.replace("apps/web/src/routes/", "")),
        "monitoring/-global-alert-settings.tsx",
        "monitoring/-incident-drawer.tsx",
        "monitoring/-node-details.tsx",
        "monitoring/-node-onboarding.tsx",
        "monitoring/-save-state.ts",
        "system/-role-actions.tsx",
        "system/-role-delete-state.ts",
        "system/-role-dialog.tsx",
        "system/-role-permission-picker.tsx",
        "system/-user-actions.tsx",
        "system/-user-dialog.tsx",
    ].sort(),
    publicAssets: ["rustzen.png"],
};

assertEqual(inventory.preset, expected.preset, "preset");
assertEqual(inventory.compositionId, expected.compositionId, "compositionId");
assertEqual(inventory.generatedRoot, expected.generatedRoot, "generatedRoot");
assertEqual(inventory.outputDirectory, expected.outputDirectory, "outputDirectory");
assertArrayEqual(inventory.selectedRoutes, expected.selectedRoutes, "selectedRoutes");
assertArrayEqual(inventory.publicAssets, expected.publicAssets, "publicAssets");
assertSafeRelativePath(inventory.generatedRoot, "generatedRoot");
assertSafeRelativePath(inventory.outputDirectory, "outputDirectory");
inventory.selectedRoutes.forEach((path) => assertSafeRelativePath(path, "selectedRoutes"));
inventory.publicAssets.forEach((path) => assertSafeRelativePath(path, "publicAssets"));
inventory.emittedFiles.forEach((path) => assertSafeRelativePath(path, "emittedFiles"));

const files = await listFiles(repositoryRoot, distRoot);
assertArrayEqual(inventory.emittedFiles, files, "emittedFiles");
if (!inventory.publicAssets.every((asset) => files.includes(asset)))
    throw new Error("selected Web output is missing a declared public asset");
if (files.includes("__rustzen_admin_marker__.json"))
    throw new Error("selected Web output copied an unselected public asset");

assertModuleIds(inventory.moduleIds, selection.compositionId, hasNotifications);
await assertSelectedApiSource(generatedApiSource);
const textFiles = files.filter((file) => /\.(?:html|js|css|map)$/.test(file));
const outputText = (
    await Promise.all(textFiles.map((file) => Bun.file(join(distRoot, file)).text()))
).join("\n");
const forbiddenText = [
    "/api/insights",
    "/api/reports",
    "/api/manage",
    "/api/system/status",
    "/analytics/",
    "/reports/",
    "/manage/deploy",
    "/manage/task",
    "/manage/log",
    "ReactQueryDevtools",
    "TanStackRouterDevtools",
    ...(!hasNotifications ? ["/api/notifications", "Message center", "消息中心"] : []),
];
const requiredText = [
    "/api/auth/login",
    "/api/auth/me",
    "/api/monitor/",
    "/api/system/users",
    "/api/system/roles",
    "/api/system/menus/options",
    "/monitoring/overview",
    ...(hasNotifications
        ? ["/api/notifications/stream", "/api/notifications/unread-count", "Message center"]
        : []),
];
const violations = forbiddenText.filter((entry) => outputText.includes(entry));
const missing = requiredText.filter((entry) => !outputText.includes(entry));
if (violations.length || missing.length)
    throw new Error(
        `selected Web text inventory mismatch: forbidden=${violations.join(",") || "none"}; missing=${missing.join(",") || "none"}`,
    );

console.log(
    JSON.stringify(
        {
            ok: true,
            preset: selection.preset,
            compositionId: selection.compositionId,
            outputDirectory: inventory.outputDirectory,
            emittedFiles: files,
            moduleIds: inventory.moduleIds,
        },
        null,
        2,
    ),
);
