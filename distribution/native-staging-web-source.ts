import { join } from "node:path";
import {
    assertSelectedWebSnapshot,
    parseInventory,
} from "../scripts/distribution-web-inventory-policy.ts";
import {
    readArtifactFileTree,
    type ArtifactFile,
} from "./release-manifest-artifacts.ts";
import { canonicalJson } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import { parseWebBinding, verifyWebBinding } from "./selected-web-binding.ts";

export async function readVerifiedWebSource(
    webRoot: string,
    selection: unknown,
): Promise<{
    descriptor: ArtifactFile;
    selectedRoutes: string[];
    web: ArtifactFile[];
}> {
    const output = await readArtifactFileTree(join(webRoot, ".."));
    const getOutput = (path: string) => requiredFile(output, path);
    const descriptor = getOutput("binding.json");
    const selectedApi = getOutput("api.ts");
    const inventory = parseInventory(json(getOutput("inventory.json").bytes));
    const web = output
        .filter((file) => file.entry.path.startsWith("dist/"))
        .map((file) => ({
            ...file,
            entry: { ...file.entry, path: file.entry.path.slice(5) },
        }));
    if (!web.length) throw new Error("staging Web root must not be empty");
    const fixedOutput = [
        "api.ts",
        "binding.json",
        "inventory.json",
        ...web.map((file) => `dist/${file.entry.path}`),
    ].sort(comparePath);
    if (
        canonicalJson(output.map((file) => file.entry.path)) !==
        canonicalJson(fixedOutput)
    ) {
        throw new Error("staging selected Web output inventory differs");
    }
    const binding = parseWebBinding(json(descriptor.bytes));
    if (
        new TextDecoder("utf-8", { fatal: true }).decode(descriptor.bytes) !==
        canonicalJson(binding)
    ) {
        throw new Error("staging selected Web binding is not canonical");
    }
    const plan = resolveSelection(selection);
    const webFiles = web.map((file) => ({ ...file.entry, bytes: file.bytes }));
    verifyWebBinding({
        binding,
        compositionId: plan.compositionId,
        selectedApiBytes: selectedApi.bytes,
        files: webFiles,
    });
    const outputText = web
        .filter((file) => /\.(?:html|js|css|map)$/.test(file.entry.path))
        .map((file) =>
            new TextDecoder("utf-8", { fatal: true }).decode(file.bytes),
        )
        .join("\n");
    assertSelectedWebSnapshot(inventory, plan, webFiles, outputText);
    if (canonicalJson(binding) !== canonicalJson(inventory.binding)) {
        throw new Error("staging selected Web binding differs from inventory");
    }
    return { descriptor, selectedRoutes: inventory.selectedRoutes, web };
}

function requiredFile(files: ArtifactFile[], path: string) {
    const file = files.find((candidate) => candidate.entry.path === path);
    if (!file) throw new Error(`staging selected Web output is missing ${path}`);
    return file;
}

function comparePath(left: string, right: string) {
    return left < right ? -1 : left > right ? 1 : 0;
}

function json(bytes: Uint8Array): unknown {
    try {
        return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes));
    } catch {
        throw new Error("staging selected Web output is invalid JSON");
    }
}
