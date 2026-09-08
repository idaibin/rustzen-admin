import { join, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";
import {
    assertCanonicalPath,
    assertSafeRelativePath,
    assertSelectedApiSource,
    assertSelectedWebSnapshot,
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
assertSafeRelativePath(inventory.generatedRoot, "generatedRoot");
assertSafeRelativePath(inventory.outputDirectory, "outputDirectory");
inventory.selectedRoutes.forEach((path) => assertSafeRelativePath(path, "selectedRoutes"));
inventory.publicAssets.forEach((path) => assertSafeRelativePath(path, "publicAssets"));
inventory.emittedFiles.forEach((path) => assertSafeRelativePath(path, "emittedFiles"));

const files = await listFiles(repositoryRoot, distRoot);
if (!inventory.publicAssets.every((asset) => files.includes(asset)))
    throw new Error("selected Web output is missing a declared public asset");
if (files.includes("__rustzen_admin_marker__.json"))
    throw new Error("selected Web output copied an unselected public asset");

await assertSelectedApiSource(generatedApiSource);
const textFiles = files.filter((file) => /\.(?:html|js|css|map)$/.test(file));
const outputText = (
    await Promise.all(textFiles.map((file) => Bun.file(join(distRoot, file)).text()))
).join("\n");
assertSelectedWebSnapshot(inventory, selection, files, outputText);

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
