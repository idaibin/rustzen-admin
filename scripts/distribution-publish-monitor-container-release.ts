import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { publishMonitorContainerRelease } from "../distribution/container-export-release.ts";
import { verifyContainerExport } from "../distribution/container-export-validator.ts";
import { readReleaseKeyFile } from "../distribution/release-key-file.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const required = [
    "--selection", "--export-root", "--expected-source-identity",
    "--output-base", "--private-key", "--public-key", "--key-id",
];
const args = Bun.argv.slice(2);
if (args.length !== required.length * 2 ||
    required.some((flag) => args.filter((x) => x === flag).length !== 1))
    throw new Error(`usage: ${required.join(" ")}`);
if (args.some((arg, index) => index % 2 === 0 ? !required.includes(arg) : !arg || arg.startsWith("--")))
    throw new Error("monitor container release arguments are invalid");
const value = (flag: string) => args[args.indexOf(flag) + 1]!;
const repositoryPath = (flag: string) => {
    const path = resolve(root, value(flag));
    if (relative(root, path).startsWith(".."))
        throw new Error(`${flag} must be beneath repository root`);
    return path;
};
const selection = await Bun.file(repositoryPath("--selection")).json();
const snapshot = await verifyContainerExport(
    repositoryPath("--export-root"),
    selection,
    value("--expected-source-identity"),
    await readWorkspaceVersion(root),
);
const privateKey = await readReleaseKeyFile(resolve(value("--private-key")), "private");
const publicKey = await readReleaseKeyFile(resolve(value("--public-key")), "public");
const result = await publishMonitorContainerRelease({
    snapshot,
    outputParent: repositoryPath("--output-base"),
    trustedRoot: root,
    privateKey,
    trusted: { keyId: value("--key-id"), publicKey },
});
console.log(canonicalJson({
    root: result.root,
    buildId: result.buildId,
    archiveSha256: result.archiveSha256,
    manifestSha256: result.manifestSha256,
    envelopeSha256: result.envelopeSha256,
}));
