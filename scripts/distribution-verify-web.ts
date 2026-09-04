import { lstat, readdir, realpath } from "node:fs/promises";
import { isAbsolute, join, relative, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";

type Inventory = {
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

const repositoryRoot = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath)
    throw new Error("usage: bun scripts/distribution-verify-web.ts --selection <selection.json>");

const selection = resolveSelection(await Bun.file(resolve(repositoryRoot, selectionPath)).json());
if (selection.preset !== "monitor")
    throw new Error("selected Web verifier currently supports only the monitor preset");

const distributionRoot = join(repositoryRoot, "target/distributions", selection.compositionId);
const outputRoot = join(distributionRoot, "web");
const distRoot = join(outputRoot, "dist");
const generatedDirectory = join(repositoryRoot, "apps/web/.selected-web", selection.compositionId);
const generatedApiSource = join(generatedDirectory, "api.ts");
const selectedApiSource = join(repositoryRoot, "apps/web/src/distribution/monitor-api.ts");
await Promise.all([
    assertCanonicalPath(distributionRoot, "distributionRoot"),
    assertCanonicalPath(outputRoot, "web outputRoot"),
    assertCanonicalPath(distRoot, "distRoot"),
    assertCanonicalPath(generatedDirectory, "generatedRoot"),
    assertCanonicalPath(generatedApiSource, "generated API source"),
    assertCanonicalPath(selectedApiSource, "selected API source"),
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
        "monitoring/-node-details.tsx",
        "monitoring/-node-onboarding-command.ts",
        "monitoring/-node-onboarding.tsx",
        "system/-role-delete-state.ts",
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

const files = await listFiles(distRoot);
assertArrayEqual(inventory.emittedFiles, files, "emittedFiles");
if (!inventory.publicAssets.every((asset) => files.includes(asset)))
    throw new Error("selected Web output is missing a declared public asset");
if (files.includes("__rustzen_admin_marker__.json"))
    throw new Error("selected Web output copied an unselected public asset");

assertModuleIds(inventory.moduleIds, selection.compositionId);
await assertSelectedApiSource(generatedApiSource);
const textFiles = files.filter((file) => /\.(?:html|js|css|map)$/.test(file));
const outputText = (await Promise.all(textFiles.map((file) => Bun.file(join(distRoot, file)).text()))).join("\n");
const forbiddenText = ["/api/insights", "/api/reports", "/api/manage", "/api/system/status", "/analytics/", "/reports/", "/manage/deploy", "/manage/task", "/manage/log", "ReactQueryDevtools", "TanStackRouterDevtools"];
const requiredText = ["/api/auth/login", "/api/auth/me", "/api/monitor/", "/api/system/users", "/api/system/roles", "/api/system/menus/options", "/monitoring/overview"];
const violations = forbiddenText.filter((entry) => outputText.includes(entry));
const missing = requiredText.filter((entry) => !outputText.includes(entry));
if (violations.length || missing.length)
    throw new Error(`selected Web text inventory mismatch: forbidden=${violations.join(",") || "none"}; missing=${missing.join(",") || "none"}`);

console.log(JSON.stringify({ ok: true, preset: selection.preset, compositionId: selection.compositionId, outputDirectory: inventory.outputDirectory, emittedFiles: files, moduleIds: inventory.moduleIds }, null, 2));

function parseInventory(value: unknown): Inventory {
    if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("selected Web inventory must be an object");
    const record = value as Record<string, unknown>;
    const keys = ["schemaVersion", "preset", "compositionId", "generatedRoot", "outputDirectory", "selectedRoutes", "publicAssets", "emittedFiles", "moduleIds"];
    if (Object.keys(record).length !== keys.length || keys.some((key) => !(key in record))) throw new Error("selected Web inventory has an invalid schema");
    if (record.schemaVersion !== 1 || typeof record.preset !== "string" || typeof record.compositionId !== "string" || typeof record.generatedRoot !== "string" || typeof record.outputDirectory !== "string" || ![record.selectedRoutes, record.publicAssets, record.emittedFiles, record.moduleIds].every((field) => Array.isArray(field) && field.every((item) => typeof item === "string"))) throw new Error("selected Web inventory has invalid field types");
    return record as unknown as Inventory;
}

function assertEqual(actual: string, expected: string, label: string) {
    if (actual !== expected) throw new Error(`selected Web inventory ${label} mismatch`);
}

function assertArrayEqual(actual: string[], expected: string[], label: string) {
    if (JSON.stringify([...actual].sort()) !== JSON.stringify([...expected].sort())) throw new Error(`selected Web inventory ${label} mismatch`);
    if (new Set(actual).size !== actual.length) throw new Error(`selected Web inventory ${label} repeats entries`);
}

function assertSafeRelativePath(path: string, label: string) {
    if (!path || isAbsolute(path) || path.split(/[\\/]/).some((part) => part === "..")) throw new Error(`selected Web inventory ${label} contains a path escape`);
}

async function assertCanonicalPath(path: string, label: string) {
    const root = resolve(repositoryRoot);
    const expected = resolve(path);
    const relativePath = relative(root, expected);
    if (relativePath === "" || isAbsolute(relativePath) || relativePath.split(/[\\/]/).some((part) => part === ".."))
        throw new Error(`selected Web ${label} escapes the repository root`);
    const rootState = await lstat(root);
    if (rootState.isSymbolicLink()) throw new Error("selected Web repository root must not be a symlink");
    let current = root;
    for (const part of relativePath.split(/[\\/]/)) {
        current = join(current, part);
        const state = await lstat(current);
        if (state.isSymbolicLink()) throw new Error(`selected Web ${label} contains a symlink: ${current}`);
    }
    if (await realpath(expected) !== expected)
        throw new Error(`selected Web ${label} is not at its canonical path`);
}

async function listFiles(directory: string, root = directory): Promise<string[]> {
    await assertCanonicalPath(directory, "output traversal");
    const entries = await readdir(directory, { withFileTypes: true });
    const nested = await Promise.all(entries.map(async (entry) => {
        const path = join(directory, entry.name);
        const state = await lstat(path);
        if (entry.isSymbolicLink() || state.isSymbolicLink()) throw new Error(`selected Web output contains symlink: ${path}`);
        if (entry.isDirectory()) return listFiles(path, root);
        if (!entry.isFile()) throw new Error(`selected Web output contains unsupported entry: ${path}`);
        return [relative(root, path).replaceAll("\\", "/")];
    }));
    return nested.flat().sort();
}

function assertModuleIds(moduleIds: string[], compositionId: string) {
    if (new Set(moduleIds).size !== moduleIds.length) throw new Error("selected Web module IDs repeat");
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
    const allowedNodeModules = new Set([
        "@ant-design/colors", "@ant-design/cssinjs", "@ant-design/cssinjs-utils", "@ant-design/fast-color", "@ant-design/icons", "@ant-design/icons-svg", "@ant-design/pro-components", "@ant-design/react-slick", "@babel/runtime", "@ctrl/tinycolor", "@dnd-kit/accessibility", "@dnd-kit/core", "@dnd-kit/modifiers", "@dnd-kit/sortable", "@dnd-kit/utilities", "@emotion/hash", "@emotion/unitless", "@reduxjs/toolkit", "@tanstack/history", "@tanstack/query-core", "@tanstack/react-query", "@tanstack/react-router", "@tanstack/react-store", "@tanstack/store", "@umijs/route-utils",
        "antd", "clsx", "cn", "compute-scroll-into-view", "d3-array", "d3-color", "d3-format", "d3-interpolate", "d3-path", "d3-scale", "d3-shape", "d3-time", "d3-time-format", "dayjs", "decimal.js-light", "dequal", "es-toolkit", "eventemitter3", "immer", "internmap", "is-mobile", "json2mq", "lodash-es", "path-to-regexp", "react", "react-dom", "react-is", "react-redux", "recharts", "redux", "redux-thunk", "reselect", "safe-stable-stringify", "scheduler", "scroll-into-view-if-needed", "seroval", "seroval-plugins", "string-convert", "stylis", "swr", "throttle-debounce", "tiny-invariant", "use-sync-external-store", "victory-vendor", "zustand"
    ]);
    for (const name of ["async-validator", "cascader", "checkbox", "collapse", "color-picker", "context", "dialog", "drawer", "dropdown", "form", "image", "input", "input-number", "mentions", "menu", "mini-decimal", "motion", "mutate-observer", "notification", "overflow", "pagination", "picker", "portal", "progress", "qrcode", "rate", "resize-observer", "segmented", "select", "slider", "steps", "switch", "table", "tabs", "tooltip", "tour", "tree", "tree-select", "trigger", "upload", "util", "virtual-list"])
        allowedNodeModules.add(`@rc-component/${name}`);
    let generatedCount = 0;
    for (const rawId of moduleIds) {
        const id = rawId.split("?", 1)[0];
        if (!id || id.includes("\\") || isAbsolute(id) || id.startsWith(".") || id.split("/").some((part) => part === ".."))
            throw new Error(`selected Web module inventory contains an unsafe module ID: ${rawId}`);
        if (id.startsWith("\0")) {
            if (id !== "\0vite/preload-helper.js") throw new Error(`selected Web module inventory contains an unknown virtual ID: ${rawId}`);
            continue;
        }
        if (id.startsWith(generatedPrefix)) {
            generatedCount += 1;
            continue;
        }
        if (id.startsWith("apps/web/.selected-web/"))
            throw new Error(`selected Web module inventory contains another composition source: ${rawId}`);
        if (allowedSourceFiles.has(id) || allowedSourceDirectories.some((directory) => id.startsWith(directory))) continue;
        const nodeModulesPrefix = "apps/web/node_modules/";
        if (id.startsWith(nodeModulesPrefix)) {
            const parts = id.slice(nodeModulesPrefix.length).split("/");
            const packageName = parts[0]?.startsWith("@") ? `${parts[0]}/${parts[1] ?? ""}` : parts[0];
            if (packageName && allowedNodeModules.has(packageName)) continue;
            throw new Error(`selected Web module inventory contains an unknown node_modules package: ${rawId}`);
        }
        throw new Error(`selected Web module inventory contains an unclassified module ID: ${rawId}`);
    }
    if (!generatedCount) throw new Error("selected Web module inventory has no generated route source");
}

async function assertSelectedApiSource(apiPath: string) {
    const apiSource = await Bun.file(apiPath).text();
    if (/\/(?:api)\/\s*["'`]\s*\+/.test(apiSource))
        throw new Error("selected Web API adapter dynamically constructs an API namespace");
    if (/(?:insights|reports|manage|system\/status)/.test(apiSource))
        throw new Error("selected Web API adapter names an excluded capability");
}
