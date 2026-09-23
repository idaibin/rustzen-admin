import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { produceMonitorNativeStagingManifest } from "../distribution/container-export-native-staging.ts";
import { verifyContainerExport } from "../distribution/container-export-validator.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const args = Bun.argv.slice(2);
const required = ["--selection", "--export-root", "--expected-source-identity", "--output-base"];
if (args.length !== required.length * 2 || required.some((flag) => args.filter((value) => value === flag).length !== 1))
    throw new Error("usage: --selection <path> --export-root <path> --expected-source-identity <text> --output-base <path>");
const value = (flag: string) => args[args.indexOf(flag) + 1];
if (args.some((arg, index) => index % 2 === 0 ? !required.includes(arg) : !arg || arg.startsWith("--")))
    throw new Error("container native staging arguments are invalid");
const outputParent = resolve(root, value("--output-base"));
if (relative(root, outputParent).startsWith(".."))
    throw new Error("output base must be beneath repository root");
const snapshot = await verifyContainerExport(
    resolve(root, value("--export-root")),
    await Bun.file(resolve(root, value("--selection"))).json(),
    value("--expected-source-identity"),
    await readWorkspaceVersion(root),
);
const result = await produceMonitorNativeStagingManifest({
    snapshot,
    outputParent,
    trustedRoot: root,
});
console.log(canonicalJson({ buildId: result.staging.buildId, files: result.staging.files.length, root: result.staging.root, manifest: result.manifest }));
