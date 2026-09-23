import { resolve } from "node:path";

import { canonicalJson } from "../distribution/release-manifest-core.ts";
import {
    verifyAnalyticsContainerExport,
    verifyContainerExport,
} from "../distribution/container-export-validator.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const args = parse(Bun.argv.slice(2));
const selectionPath = args.get("--selection");
const exportRoot = args.get("--export-root");
const expectedSourceIdentity = args.get("--expected-source-identity");
const evidence = args.get("--evidence") ?? "host-synthetic";
if (!selectionPath || !exportRoot || !expectedSourceIdentity || (args.size !== 3 && args.size !== 4))
    throw new Error("usage: --selection <selection.json> --export-root <container-export-root> --expected-source-identity <source-identity> [--evidence <host-synthetic|linux-amd64-buildkit>]");
if (evidence !== "host-synthetic" && evidence !== "linux-amd64-buildkit")
    throw new Error("container export evidence must be host-synthetic or linux-amd64-buildkit");
const selection = await Bun.file(resolve(root, selectionPath)).json();
if (evidence === "linux-amd64-buildkit" && (selection as { preset?: unknown }).preset !== "analytics")
    throw new Error("linux-amd64-buildkit evidence is validated for the analytics selection");
const releaseVersion = await readWorkspaceVersion(root);
const snapshot = evidence === "linux-amd64-buildkit"
    ? await verifyAnalyticsContainerExport(resolve(root, exportRoot), selection, expectedSourceIdentity, releaseVersion)
    : await verifyContainerExport(resolve(root, exportRoot), selection, expectedSourceIdentity, releaseVersion);
console.log(canonicalJson({
    compositionId: snapshot.manifest().compositionId,
    evidence,
    files: snapshot.paths().length,
    sourceIdentity: snapshot.recordedProvenance().sourceIdentityInput,
    verified: true,
}));

function parse(values: string[]) {
    const allowed = new Set(["--selection", "--export-root", "--expected-source-identity", "--evidence"]);
    if (values.length !== 6 && values.length !== 8) return new Map<string, string>();
    const result = new Map<string, string>();
    for (let index = 0; index < values.length; index += 2) {
        const name = values[index], value = values[index + 1];
        if (!name || !value || !allowed.has(name) || result.has(name)) return new Map<string, string>();
        result.set(name, value);
    }
    return result;
}
