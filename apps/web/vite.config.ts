import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

import tailwindcss from "@tailwindcss/vite";
import { tanstackRouter } from "@tanstack/router-plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig, lazyPlugins } from "vite-plus";

// Vite+ and Vite 8 expose compatible plugins through distinct type identities.
type VitePlusPluginList = NonNullable<ReturnType<typeof lazyPlugins>>;

const WEB_DEV_PORT = 9800;
const BACKEND_PORT = 9801;
const selectedPreset = process.env.RUSTZEN_WEB_PRESET;
const selectedRoot = process.env.RUSTZEN_WEB_SELECTED_ROOT;
const selectedOutput = process.env.RUSTZEN_WEB_OUTPUT_DIR;
const selectedViteInventory = process.env.RUSTZEN_WEB_VITE_INVENTORY;
const isSelectedBuild =
    ["monitor", "monitor-notify", "analytics", "reports"].includes(selectedPreset ?? "") &&
    Boolean(selectedRoot) &&
    Boolean(selectedOutput);

if (selectedPreset && !isSelectedBuild) {
    throw new Error("selected Web build preset is unsupported");
}

const selectedPath = (...segments: string[]) => resolve(selectedRoot!, ...segments);
const selectedInventoryPlugin = isSelectedBuild
    ? {
          name: "rustzen-selected-web-inventory",
          generateBundle(
              this: { getModuleIds: () => Iterable<string> },
              _options: unknown,
              bundle: object,
          ) {
              writeFileSync(
                  selectedViteInventory!,
                  `${JSON.stringify(
                      {
                          emittedFiles: Object.keys(bundle).sort(),
                          moduleIds: [...this.getModuleIds()].sort(),
                      },
                      null,
                      2,
                  )}\n`,
              );
          },
      }
    : null;

// https://vite.dev/config/
export default defineConfig({
    lint: { options: { typeAware: true, typeCheck: true } },
    fmt: { sortImports: {} },
    staged: {
        "*": "vp check --fix",
    },
    plugins: lazyPlugins(
        () =>
            [
                tanstackRouter(
                    isSelectedBuild
                        ? {
                              autoCodeSplitting: true,
                              routesDirectory: selectedPath("routes"),
                              generatedRouteTree: selectedPath("routeTree.gen.ts"),
                              tmpDir: selectedPath(".tanstack"),
                          }
                        : { autoCodeSplitting: true },
                ),
                viteReact(),
                tailwindcss(),
                ...(selectedInventoryPlugin ? [selectedInventoryPlugin] : []),
            ] as unknown as VitePlusPluginList,
    ),
    resolve: {
        tsconfigPaths: true,
        alias: isSelectedBuild
            ? [
                  { find: /^@\/api$/, replacement: selectedPath("api.ts") },
                  {
                      find: /^@\/components\/layout$/,
                      replacement: selectedPath("layout.tsx"),
                  },
                  {
                      find: /^@\/store\/useAuthStore$/,
                      replacement: selectedPath("auth-store.ts"),
                  },
                  {
                      find: /^@\/api\/system\/menu\/query-options$/,
                      replacement: selectedPath("menu-query-options.ts"),
                  },
              ]
            : undefined,
    },
    server: {
        host: "0.0.0.0",
        port: WEB_DEV_PORT,
        open: false,
        allowedHosts: ["host.docker.internal", "terminal.local"],
        proxy: {
            "/api": {
                target: `http://127.0.0.1:${BACKEND_PORT}`,
                changeOrigin: true,
            },
            "/resources": {
                target: `http://127.0.0.1:${BACKEND_PORT}`,
                changeOrigin: true,
            },
        },
    },
    publicDir: isSelectedBuild ? selectedPath("public") : "public",
    build: isSelectedBuild
        ? {
              outDir: selectedOutput,
              emptyOutDir: true,
              rollupOptions: { input: selectedPath("index.html") },
          }
        : undefined,
});
