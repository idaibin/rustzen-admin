import { cp, mkdir, rm } from "node:fs/promises";
import { dirname, join, relative, resolve, sep } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";
import { supportsSelectedWeb } from "../distribution/selected-web-producer.ts";
import {
    WEB_BINDING_SLOT,
    canonicalBindingBytes,
    createWebBinding,
    readWebFiles,
    stampIndex,
    verifyWebBinding,
} from "../distribution/selected-web-binding.ts";
import { createSelectedWebBootstrap } from "../distribution/selected-web-bootstrap.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const webRoot = join(repositoryRoot, "apps/web");
const [flag, selectionPath] = Bun.argv.slice(2);

if (flag !== "--selection" || !selectionPath) {
    throw new Error("usage: bun scripts/distribution-build-web.ts --selection <selection.json>");
}

const selection = resolveSelection(await Bun.file(resolve(repositoryRoot, selectionPath)).json());
if (!supportsSelectedWeb(selection))
    throw new Error("selected Web producer does not support this exact composition");
const hasNotifications = selection.capabilities.includes("notifications");
const isAnalytics = selection.preset === "analytics";
const isReports = selection.preset === "reports";

const outputRoot = join(repositoryRoot, "target/distributions", selection.compositionId, "web");
const generatedRoot = join(webRoot, ".selected-web", selection.compositionId);
const routeRoot = join(generatedRoot, "routes");
const reportsRouteHelpers = [
    "apps/web/src/routes/reports/-schedule-permissions.ts",
    "apps/web/src/routes/reports/-schedule-toggle.ts",
    "apps/web/src/routes/reports/-runs/live-frame.tsx",
    "apps/web/src/routes/reports/-runs/retry-run-button.tsx",
    "apps/web/src/routes/reports/-runs/retry-run-state.ts",
    "apps/web/src/routes/reports/-runs/run-details.tsx",
    "apps/web/src/routes/reports/-runs/run-dialog.tsx",
    "apps/web/src/routes/reports/-runs/status.ts",
    "apps/web/src/routes/reports/-templates/delete-flow-dialog.tsx",
    "apps/web/src/routes/reports/-templates/flow-dialog.tsx",
    "apps/web/src/routes/reports/-templates/schedule-columns.tsx",
    "apps/web/src/routes/reports/-templates/schedule-dialog.tsx",
    "apps/web/src/routes/reports/-templates/schedule-panel.tsx",
    "apps/web/src/routes/reports/-templates/schedule-save-state.ts",
    "apps/web/src/routes/reports/-templates/schedule-toggle.tsx",
    "apps/web/src/routes/reports/-templates/schedule-utils.tsx",
    "apps/web/src/routes/reports/-templates/target-dialog.tsx",
    "apps/web/src/routes/reports/-templates/templates-content.tsx",
];
const sourceRoutes = [
    ...selection.webRoots,
    ...(isAnalytics ? ["apps/web/src/routes/analytics/-event-target.ts"] : []),
    ...(isReports ? reportsRouteHelpers : []),
    ...(!isAnalytics && !isReports
        ? [
              "apps/web/src/routes/monitoring/-global-alert-settings.tsx",
              "apps/web/src/routes/monitoring/-incident-drawer.tsx",
              "apps/web/src/routes/monitoring/-node-details.tsx",
              "apps/web/src/routes/monitoring/-node-alert-policy.tsx",
              "apps/web/src/routes/monitoring/-node-onboarding.tsx",
              "apps/web/src/routes/monitoring/-save-state.ts",
          ]
        : []),
    "apps/web/src/routes/system/-role-actions.tsx",
    "apps/web/src/routes/system/-role-delete-state.ts",
    "apps/web/src/routes/system/-role-dialog.tsx",
    "apps/web/src/routes/system/-role-permission-picker.tsx",
    "apps/web/src/routes/system/-user-actions.tsx",
    "apps/web/src/routes/system/-user-dialog.tsx",
];
const selectedRoutes = [
    "index.tsx",
    ...sourceRoutes.map((source) => source.replace("apps/web/src/routes/", "")),
].sort();
const publicAssets = ["rustzen.png"];
const viteInventoryPath = join(generatedRoot, "vite-inventory.json");

const copyRoute = async (source: string) => {
    const relativeRoute = relative(join(webRoot, "src/routes"), join(repositoryRoot, source));
    const destination = join(routeRoot, relativeRoute);
    await mkdir(dirname(destination), { recursive: true });
    let content = await Bun.file(join(repositoryRoot, source)).text();
    if (relativeRoute === "monitoring/incidents.tsx" && !hasNotifications) {
        content = content
            .replace(
                'import { NotificationDeliveryCard } from "@/components/feedback/notification-delivery-card";\n',
                "",
            )
            .replace(
                /    const deliveryCard = \(\n        <NotificationDeliveryCard\n            queryKey=\{\["monitor", "notification-delivery"\]\}\n            queryFn=\{monitorAPI\.notificationDelivery\}\n        \/>\n    \);\n/,
                "",
            )
            .replaceAll("                {deliveryCard}\n", "")
            .replaceAll("            {deliveryCard}\n", "");
    }
    if (relativeRoute === "reports/runs.tsx" && !hasNotifications) {
        content = content
            .replace(
                'import { NotificationDeliveryCard } from "@/components/feedback/notification-delivery-card";\n',
                "",
            )
            .replace(
                /    const deliveryCard = \(\n        <NotificationDeliveryCard\n            queryKey=\{\["reports", "notification-delivery"\]\}\n            queryFn=\{reportsAPI\.notificationDelivery\}\n        \/>\n    \);\n/,
                "",
            )
            .replaceAll("                {deliveryCard}\n", "")
            .replaceAll("            {deliveryCard}\n", "");
    }
    if (relativeRoute === "__root.tsx" && !hasNotifications) {
        content = content
            .replace('import { NotificationShell } from "./-notifications-shell";\n', "")
            .replace(" headerActions={token ? <NotificationShell /> : null}", "");
    }
    if (
        relativeRoute === "monitoring/incidents.tsx" &&
        !hasNotifications &&
        /NotificationDeliveryCard|deliveryCard|notification-delivery/.test(content)
    )
        throw new Error("pure Monitor incidents retains notification delivery");
    if (
        relativeRoute === "reports/runs.tsx" &&
        !hasNotifications &&
        /NotificationDeliveryCard|deliveryCard|notification-delivery/.test(content)
    )
        throw new Error("pure Reports runs retains notification delivery");
    await Bun.write(destination, content);
};

await rm(outputRoot, { recursive: true, force: true });
await rm(generatedRoot, { recursive: true, force: true });
await Promise.all(sourceRoutes.map(copyRoute));
await mkdir(join(generatedRoot, "public"), { recursive: true });
await cp(join(webRoot, "src/style.css"), join(generatedRoot, "style.css"));
await cp(join(webRoot, "src/styles"), join(generatedRoot, "styles"), { recursive: true });
const landingRoute = isAnalytics
    ? "/analytics/overview"
    : isReports
      ? "/reports/templates"
      : "/monitoring/overview";
await Bun.write(
    join(routeRoot, "index.tsx"),
    `import { createFileRoute, redirect } from "@tanstack/react-router";\n\nexport const Route = createFileRoute("/")({ beforeLoad: () => { throw redirect({ to: "${landingRoute}" }); } });\n`,
);
await cp(join(webRoot, "public/rustzen.png"), join(generatedRoot, "public/rustzen.png"));
await Bun.write(
    join(generatedRoot, "api.ts"),
    await Bun.file(
        join(
            webRoot,
            isAnalytics
                ? "src/distribution/analytics-api.ts"
                : isReports
                  ? "src/distribution/reports-api.ts"
                  : hasNotifications
                    ? "src/distribution/monitor-notify-api.ts"
                    : "src/distribution/monitor-api.ts",
        ),
    ).text(),
);
const distributionPrefix = isAnalytics ? "analytics" : isReports ? "reports" : "monitor";
await Bun.write(
    join(generatedRoot, "layout.tsx"),
    await Bun.file(join(webRoot, `src/distribution/${distributionPrefix}-layout.tsx`)).text(),
);
await Bun.write(
    join(generatedRoot, "auth-store.ts"),
    await Bun.file(join(webRoot, `src/distribution/${distributionPrefix}-auth-store.ts`)).text(),
);
await Bun.write(
    join(generatedRoot, "menu-query-options.ts"),
    await Bun.file(
        join(webRoot, `src/distribution/${distributionPrefix}-menu-query-options.ts`),
    ).text(),
);
const distributionTitle = isAnalytics ? "Analytics" : isReports ? "Reports" : "Monitor";
await Bun.write(
    join(generatedRoot, "index.html"),
    `<!doctype html><html lang="en"><head><meta charset="UTF-8" /><link href="./style.css" rel="stylesheet" /><meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>Rustzen ${distributionTitle}</title></head><body><div id="root"></div><script type="module" src="./main.tsx"></script></body></html>`,
);
await Bun.write(
    join(generatedRoot, "main.tsx"),
    await Bun.file(join(webRoot, "src/main.tsx")).text(),
);

const command = ["bun", "run", "vp", "build"];
const build = Bun.spawnSync(command, {
    cwd: webRoot,
    env: {
        ...process.env,
        NODE_ENV: "production",
        RUSTZEN_WEB_PRESET: selection.preset,
        RUSTZEN_WEB_SELECTED_ROOT: generatedRoot,
        RUSTZEN_WEB_OUTPUT_DIR: join(outputRoot, "dist"),
        RUSTZEN_WEB_VITE_INVENTORY: viteInventoryPath,
        VITE_RUSTZEN_REPORTS_SELECTED: selection.capabilities.includes("reports")
            ? "true"
            : "false",
    },
    stdout: "inherit",
    stderr: "inherit",
});
if (build.exitCode !== 0) process.exit(build.exitCode ?? 1);

const viteInventory = await Bun.file(viteInventoryPath).json();
if (
    !viteInventory ||
    typeof viteInventory !== "object" ||
    !Array.isArray(viteInventory.emittedFiles)
)
    throw new Error("Vite did not emit a valid selected Web bundle inventory");
const mainEntry = viteInventory.emittedFiles.find(
    (file: unknown) => typeof file === "string" && /^assets\/index-[^/]+\.js$/.test(file),
);
if (typeof mainEntry !== "string") throw new Error("selected Web bundle has no main entry chunk");
const styleEntries = viteInventory.emittedFiles.filter(
    (file: unknown): file is string =>
        typeof file === "string" && /^assets\/[^/]+\.css$/.test(file),
);
if (styleEntries.length === 0) throw new Error("selected Web bundle has no stylesheet");
const styleLinks = styleEntries
    .sort()
    .map((file: string) => `<link rel="stylesheet" href="/${file}" />`)
    .join("");
const bootstrap = createSelectedWebBootstrap({
    entryPath: mainEntry,
    entryBytes: await Bun.file(join(outputRoot, "dist", mainEntry)).bytes(),
});
await rm(join(outputRoot, "dist", ".selected-web"), { recursive: true, force: true });
await Bun.write(
    join(outputRoot, "dist", "index.html"),
    `<!doctype html><html lang="en"><head><meta charset="UTF-8" />${styleLinks}<meta name="viewport" content="width=device-width, initial-scale=1.0" /><title>Rustzen ${distributionTitle}</title><meta name="rustzen-web-binding" content="${WEB_BINDING_SLOT}" /></head><body><div id="root"></div>${bootstrap.html}</body></html>`,
);
const selectedApiBytes = await Bun.file(join(generatedRoot, "api.ts")).bytes();
await Bun.write(join(outputRoot, "api.ts"), selectedApiBytes);
const preStampFiles = await readWebFiles(join(outputRoot, "dist"));
const binding = createWebBinding({
    compositionId: selection.compositionId,
    selectedApiBytes,
    files: preStampFiles,
});
const unstampedIndex = preStampFiles.find((file) => file.path === "index.html");
if (!unstampedIndex) throw new Error("selected Web bundle has no index.html");
await Bun.write(
    join(outputRoot, "dist", "index.html"),
    stampIndex(unstampedIndex.bytes, binding.webDigest),
);
const emittedFiles = await readWebFiles(join(outputRoot, "dist"));
verifyWebBinding({
    binding,
    compositionId: selection.compositionId,
    selectedApiBytes,
    files: emittedFiles,
});

const output = {
    schemaVersion: 2,
    preset: selection.preset,
    compositionId: selection.compositionId,
    generatedRoot: relative(repositoryRoot, generatedRoot),
    outputDirectory: relative(repositoryRoot, join(outputRoot, "dist")),
    selectedRoutes,
    publicAssets,
    emittedFiles: emittedFiles.map((file) => file.path),
    fileInventory: emittedFiles.map(({ path, size, sha256 }) => ({ path, size, sha256 })),
    moduleIds: normalizeModuleIds(viteInventory.moduleIds),
    binding,
};
await Bun.write(join(outputRoot, "inventory.json"), `${JSON.stringify(output, null, 2)}\n`);
await Bun.write(join(outputRoot, "binding.json"), canonicalBindingBytes(binding));
console.log(JSON.stringify(output, null, 2));

function normalizeModuleIds(moduleIds: unknown): string[] {
    if (!Array.isArray(moduleIds) || moduleIds.some((id) => typeof id !== "string"))
        throw new Error("Vite did not emit a valid module ID inventory");
    return moduleIds
        .map((id) => (id.startsWith(repositoryRoot + sep) ? relative(repositoryRoot, id) : id))
        .sort();
}
