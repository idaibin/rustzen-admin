import { resolve } from "node:path";

import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceContainerExport } from "../distribution/container-export.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const args = values(Bun.argv.slice(2), new Set(["--selection", "--output-root"]));
const selectionPath = args.get("--selection");
const outputRoot = args.get("--output-root");
if (!selectionPath || !outputRoot || args.size !== 2)
    throw new Error("usage: --selection <selection.json> --output-root <container-output-root>");
const targetTriple = process.env.RUSTZEN_CONTAINER_TARGET_TRIPLE;
const sourceIdentity = process.env.RUSTZEN_CONTAINER_SOURCE_IDENTITY;
const commands = process.env.RUSTZEN_CONTAINER_BUILD_COMMANDS;
const evidence = process.env.RUSTZEN_CONTAINER_EVIDENCE;
if (!targetTriple || !sourceIdentity || !commands || evidence !== "linux-amd64-buildkit")
    throw new Error("container export requires linux-amd64-buildkit, target, source identity and build command inputs");
const rustc = Bun.spawnSync(["rustc", "-Vv"], { stdout: "pipe", stderr: "pipe" });
if (rustc.exitCode !== 0) throw new Error("rustc -Vv failed in container build stage");
const produced = await produceContainerExport({
    selection: await Bun.file(resolve(root, selectionPath)).json(),
    outputRoot,
    targetTriple,
    sourceIdentity,
    buildCommands: JSON.parse(commands),
    rustcVv: new TextDecoder().decode(rustc.stdout),
    releaseVersion: await readWorkspaceVersion(root),
    evidence,
});
console.log(canonicalJson({
    compositionId: produced.manifest.compositionId,
    manifest: "release/output-manifest.json",
    provenance: "release/container-provenance.json",
}));

function values(input: string[], allowed: Set<string>) {
    if (input.length % 2 !== 0) return new Map<string, string>();
    const result = new Map<string, string>();
    for (let index = 0; index < input.length; index += 2) {
        const name = input[index], value = input[index + 1];
        if (!name || !value || !allowed.has(name) || result.has(name))
            return new Map<string, string>();
        result.set(name, value);
    }
    return result;
}
