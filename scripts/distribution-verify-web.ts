import { join, resolve } from "node:path";

import { resolveSelection } from "../distribution/resolver.ts";
import {
    assertCanonicalPath,
    assertSafeRelativePath,
    assertSelectedWebSnapshot,
    parseInventory,
} from "./distribution-web-inventory-policy.ts";
import { assertSelectedApiSource } from "./distribution-selected-api-policy.ts";
import {
    canonicalBindingBytes,
    parseWebBinding,
    readWebFiles,
    verifyWebBinding,
} from "../distribution/selected-web-binding.ts";
import {
    assertNotificationDeliveryClosure,
    assertSelectedApiAuthority,
} from "./distribution-notification-delivery-closure.ts";

const repositoryRoot = resolve(import.meta.dir, "..");
const [flag, selectionPath] = Bun.argv.slice(2);
if (flag !== "--selection" || !selectionPath)
    throw new Error("usage: bun scripts/distribution-verify-web.ts --selection <selection.json>");

const selection = resolveSelection(await Bun.file(resolve(repositoryRoot, selectionPath)).json());
if (!["monitor", "monitor-notify", "analytics"].includes(selection.preset))
    throw new Error("selected Web verifier does not support this composition");

const distributionRoot = join(repositoryRoot, "target/distributions", selection.compositionId);
const outputRoot = join(distributionRoot, "web");
const distRoot = join(outputRoot, "dist");
const generatedDirectory = join(repositoryRoot, "apps/web/.selected-web", selection.compositionId);
const generatedApiSource = join(generatedDirectory, "api.ts");
const selectedApiSource = join(
    repositoryRoot,
    "apps/web/src/distribution",
    selection.preset === "analytics"
        ? "analytics-api.ts"
        : selection.preset === "monitor-notify"
          ? "monitor-notify-api.ts"
          : "monitor-api.ts",
);
const retainedApiSource = join(outputRoot, "api.ts");
const bindingPath = join(outputRoot, "binding.json");
await Promise.all([
    assertCanonicalPath(repositoryRoot, distributionRoot, "distributionRoot"),
    assertCanonicalPath(repositoryRoot, outputRoot, "web outputRoot"),
    assertCanonicalPath(repositoryRoot, distRoot, "distRoot"),
    assertCanonicalPath(repositoryRoot, generatedDirectory, "generatedRoot"),
    assertCanonicalPath(repositoryRoot, generatedApiSource, "generated API source"),
    assertCanonicalPath(repositoryRoot, selectedApiSource, "selected API source"),
    assertCanonicalPath(repositoryRoot, retainedApiSource, "retained selected API source"),
    assertCanonicalPath(repositoryRoot, bindingPath, "binding descriptor"),
]);

const inventory = parseInventory(await Bun.file(join(outputRoot, "inventory.json")).json());
assertSafeRelativePath(inventory.generatedRoot, "generatedRoot");
assertSafeRelativePath(inventory.outputDirectory, "outputDirectory");
inventory.selectedRoutes.forEach((path) => assertSafeRelativePath(path, "selectedRoutes"));
inventory.publicAssets.forEach((path) => assertSafeRelativePath(path, "publicAssets"));
inventory.emittedFiles.forEach((path) => assertSafeRelativePath(path, "emittedFiles"));

const files = await readWebFiles(distRoot);
const filePaths = files.map((file) => file.path);
if (!inventory.publicAssets.every((asset) => filePaths.includes(asset)))
    throw new Error("selected Web output is missing a declared public asset");
if (filePaths.includes("__rustzen_admin_marker__.json"))
    throw new Error("selected Web output copied an unselected public asset");

await assertSelectedApiSource(generatedApiSource, selection.preset);
const authoritativeApiBytes = await Bun.file(selectedApiSource).bytes();
const generatedApiBytes = await Bun.file(generatedApiSource).bytes();
const retainedApiBytes = await Bun.file(retainedApiSource).bytes();
assertSelectedApiAuthority({
    authoritative: authoritativeApiBytes,
    generated: generatedApiBytes,
    retained: retainedApiBytes,
});
const binding = parseWebBinding(JSON.parse(await Bun.file(bindingPath).text()));
if (
    new TextDecoder().decode(canonicalBindingBytes(binding)) !==
    (await Bun.file(bindingPath).text())
)
    throw new Error("selected Web binding descriptor is not canonical");
if (JSON.stringify(binding) !== JSON.stringify(inventory.binding))
    throw new Error("selected Web binding descriptor differs from inventory");
verifyWebBinding({
    binding,
    compositionId: selection.compositionId,
    selectedApiBytes: retainedApiBytes,
    files,
});
const textFiles = files.filter((file) => /\.(?:html|js|css|map)$/.test(file.path));
const outputText = (
    await Promise.all(
        textFiles.map((file) => new TextDecoder("utf-8", { fatal: true }).decode(file.bytes)),
    )
).join("\n");
assertSelectedWebSnapshot(inventory, selection, files, outputText);
const retainedText = new TextDecoder().decode(retainedApiBytes);
if (selection.preset !== "analytics") {
    const generatedText = await Bun.file(
        generatedDirectory + "/routes/monitoring/incidents.tsx",
    ).text();
    assertNotificationDeliveryClosure({
        hasNotifications: selection.preset === "monitor-notify",
        retainedText,
        generatedText,
        outputText,
    });
}

console.log(
    JSON.stringify(
        {
            ok: true,
            preset: selection.preset,
            compositionId: selection.compositionId,
            outputDirectory: inventory.outputDirectory,
            emittedFiles: filePaths,
            moduleIds: inventory.moduleIds,
        },
        null,
        2,
    ),
);
