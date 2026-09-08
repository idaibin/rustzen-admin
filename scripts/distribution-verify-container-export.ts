import { resolve } from "node:path";

import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { verifyContainerExport } from "../distribution/container-export-validator.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const args = parse(Bun.argv.slice(2));
const selectionPath = args.get("--selection");
const exportRoot = args.get("--export-root");
const expectedSourceIdentity = args.get("--expected-source-identity");
if (!selectionPath || !exportRoot || !expectedSourceIdentity || args.size !== 3)
    throw new Error("usage: --selection <selection.json> --export-root <container-export-root> --expected-source-identity <source-identity>");
const snapshot = await verifyContainerExport(
    resolve(root, exportRoot),
    await Bun.file(resolve(root, selectionPath)).json(),
    expectedSourceIdentity,
    await readWorkspaceVersion(root),
);
console.log(canonicalJson({
    compositionId: snapshot.manifest().compositionId,
    files: snapshot.paths().length,
    sourceIdentity: snapshot.recordedProvenance().sourceIdentityInput,
    verified: true,
}));

function parse(values: string[]) {
    const allowed = new Set(["--selection", "--export-root", "--expected-source-identity"]);
    if (values.length !== 6) return new Map<string, string>();
    const result = new Map<string, string>();
    for (let index = 0; index < values.length; index += 2) {
        const name = values[index], value = values[index + 1];
        if (!name || !value || !allowed.has(name) || result.has(name)) return new Map<string, string>();
        result.set(name, value);
    }
    return result;
}
