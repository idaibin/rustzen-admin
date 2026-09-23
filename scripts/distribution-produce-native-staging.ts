import { join, relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceNativeStaging } from "../distribution/native-staging.ts";
import { resolveSelection } from "../distribution/resolver.ts";

const root = resolve(import.meta.dir, "..");
const args = Bun.argv.slice(2);
const value = (name: string) => args[args.indexOf(name) + 1];
const selectionPath = value("--selection");
const binaryRoot = value("--binary-root");
const configRoot = value("--config-root");
const nativeRoot = value("--native-root");
const protocolRoot = value("--protocol-root");
const outputBase = value("--output-base") ?? "target/distributions";
const releaseVersion = value("--release-version");
const sourceIdentity = value("--source-identity");
const toolchain = value("--toolchain");
const selectedRoutes = value("--selected-routes");
if (
    !selectionPath ||
    !binaryRoot ||
    !configRoot ||
    !nativeRoot ||
    !protocolRoot ||
    !releaseVersion ||
    !sourceIdentity ||
    !toolchain ||
    !selectedRoutes
)
    throw new Error(
        "usage: --selection --binary-root --config-root --native-root --protocol-root --release-version --source-identity --toolchain --selected-routes [--output-base --web-root --api-root --schema-root]",
    );
const selection = await Bun.file(resolve(root, selectionPath)).json();
const resolvedOutputBase = resolve(root, outputBase);
if (relative(root, resolvedOutputBase).startsWith(".."))
    throw new Error("output base must be beneath repository root");
const plan = resolveSelection(selection);
const result = await produceNativeStaging({
    selection,
    outputParent: resolvedOutputBase,
    trustedRoot: root,
    releaseVersion,
    sourceIdentity,
    toolchain,
    selectedRoutes: selectedRoutes.split(",").filter(Boolean).sort(),
    binaryRoot,
    configRoot,
    nativeRoot,
    protocolRoot,
    webRoot: value("--web-root"),
    apiRoot: value("--api-root"),
    schemaRoot: value("--schema-root"),
});
console.log(
    canonicalJson({
        buildId: result.buildId,
        compositionId: plan.compositionId,
        files: result.files.length,
        root: result.root,
        sha256: result.sha256,
    }),
);
