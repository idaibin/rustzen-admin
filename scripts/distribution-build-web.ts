import { cp, lstat, mkdir, readdir, rm } from "node:fs/promises";
import { dirname, join, relative, resolve, sep } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const webRoot = join(repositoryRoot, "apps/web");
const [flag, selectionPath] = Bun.argv.slice(2);

if (flag !== "--selection" || !selectionPath) {
    throw new Error("usage: bun scripts/distribution-build-web.ts --selection <selection.json>");
}

const selection = resolveSelection(await Bun.file(resolve(repositoryRoot, selectionPath)).json());
if (selection.preset !== "monitor") {
    throw new Error("selected Web producer currently supports only the monitor preset");
}

const outputRoot = join(repositoryRoot, "target/distributions", selection.compositionId, "web");
const generatedRoot = join(webRoot, ".selected-web", selection.compositionId);
const routeRoot = join(generatedRoot, "routes");
const sourceRoutes = [
    ...selection.webRoots,
    "apps/web/src/routes/monitoring/-global-alert-settings.tsx",
    "apps/web/src/routes/monitoring/-node-details.tsx",
    "apps/web/src/routes/monitoring/-node-onboarding.tsx",
    "apps/web/src/routes/monitoring/-node-onboarding-command.ts",
    "apps/web/src/routes/system/-role-delete-state.ts",
];
const selectedRoutes = ["index.tsx", ...sourceRoutes.map((source) => source.replace("apps/web/src/routes/", ""))].sort();
const publicAssets = ["rustzen.png"];
const viteInventoryPath = join(generatedRoot, "vite-inventory.json");

const copyRoute = async (source: string) => {
    const relativeRoute = relative(join(webRoot, "src/routes"), join(repositoryRoot, source));
    const destination = join(routeRoot, relativeRoute);
    await mkdir(dirname(destination), { recursive: true });
    const content = await Bun.file(join(repositoryRoot, source)).text();
    await Bun.write(destination, content);
};

await rm(outputRoot, { recursive: true, force: true });
await rm(generatedRoot, { recursive: true, force: true });
await Promise.all(sourceRoutes.map(copyRoute));
await mkdir(join(generatedRoot, "public"), { recursive: true });
await Bun.write(
    join(routeRoot, "index.tsx"),
    `import { createFileRoute, redirect } from "@tanstack/react-router";\n\nexport const Route = createFileRoute("/")({ beforeLoad: () => { throw redirect({ to: "/monitoring/overview" }); } });\n`,
);
await cp(join(webRoot, "public/rustzen.png"), join(generatedRoot, "public/rustzen.png"));
await Bun.write(join(generatedRoot, "api.ts"), await Bun.file(join(webRoot, "src/distribution/monitor-api.ts")).text());
await Bun.write(join(generatedRoot, "layout.tsx"), await Bun.file(join(webRoot, "src/distribution/monitor-layout.tsx")).text());
await Bun.write(join(generatedRoot, "auth-store.ts"), await Bun.file(join(webRoot, "src/distribution/monitor-auth-store.ts")).text());
await Bun.write(join(generatedRoot, "menu-query-options.ts"), await Bun.file(join(webRoot, "src/distribution/monitor-menu-query-options.ts")).text());
await Bun.write(
    join(generatedRoot, "main.tsx"),
    (await Bun.file(join(webRoot, "src/main.tsx")).text()).replace(
        'from "./routeTree.gen"',
        'from "./routeTree.gen"',
    ),
);

const command = ["bun", "run", "vp", "build"];
const build = Bun.spawnSync(command, {
    cwd: webRoot,
    env: {
        ...process.env,
        NODE_ENV: "production",
        RUSTZEN_WEB_PRESET: "monitor",
        RUSTZEN_WEB_SELECTED_ROOT: generatedRoot,
        RUSTZEN_WEB_OUTPUT_DIR: join(outputRoot, "dist"),
        RUSTZEN_WEB_VITE_INVENTORY: viteInventoryPath,
    },
    stdout: "inherit",
    stderr: "inherit",
});
if (build.exitCode !== 0) process.exit(build.exitCode ?? 1);

const viteInventory = await Bun.file(viteInventoryPath).json();
const emittedFiles = await listFiles(join(outputRoot, "dist"));

const output = {
    schemaVersion: 1,
    preset: selection.preset,
    compositionId: selection.compositionId,
    generatedRoot: relative(repositoryRoot, generatedRoot),
    outputDirectory: relative(repositoryRoot, join(outputRoot, "dist")),
    selectedRoutes,
    publicAssets,
    emittedFiles,
    moduleIds: normalizeModuleIds(viteInventory.moduleIds),
};
await Bun.write(join(outputRoot, "inventory.json"), `${JSON.stringify(output, null, 2)}\n`);
console.log(JSON.stringify(output, null, 2));

async function listFiles(directory: string, root = directory): Promise<string[]> {
    const entries = await readdir(directory, { withFileTypes: true });
    const nested = await Promise.all(
        entries.map(async (entry) => {
            const path = join(directory, entry.name);
            if (entry.isSymbolicLink()) throw new Error(`selected Web output contains symlink: ${path}`);
            if (entry.isDirectory()) return listFiles(path, root);
            if (!entry.isFile()) throw new Error(`selected Web output contains unsupported entry: ${path}`);
            const state = await lstat(path);
            if (state.isSymbolicLink()) throw new Error(`selected Web output contains symlink: ${path}`);
            return [relative(root, path).replaceAll("\\", "/")];
        }),
    );
    return nested.flat().sort();
}

function normalizeModuleIds(moduleIds: unknown): string[] {
    if (!Array.isArray(moduleIds) || moduleIds.some((id) => typeof id !== "string"))
        throw new Error("Vite did not emit a valid module ID inventory");
    return moduleIds
        .map((id) => (id.startsWith(repositoryRoot + sep) ? relative(repositoryRoot, id) : id))
        .sort();
}
