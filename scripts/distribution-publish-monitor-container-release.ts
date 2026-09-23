import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { publishMonitorContainerRelease } from "../distribution/container-export-release.ts";
import { verifyAnalyticsContainerExport, verifyContainerExport } from "../distribution/container-export-validator.ts";
import { readReleaseKeyFile } from "../distribution/release-key-file.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const required = [
    "--selection", "--export-root", "--expected-source-identity",
    "--output-base", "--private-key", "--public-key", "--key-id",
];
const usage = `usage: ${required.join(" ")} [--evidence <host-synthetic|linux-amd64-buildkit>]`;
const args = Bun.argv.slice(2);
if (args.length % 2 !== 0) throw new Error(usage);
const seen = new Map<string, string>();
for (let index = 0; index < args.length; index += 2) {
    const name = args[index], value = args[index + 1];
    if ((!required.includes(name) && name !== "--evidence") || seen.has(name) || !value || value.startsWith("--"))
        throw new Error("monitor container release arguments are invalid");
    seen.set(name, value);
}
if (!required.every((flag) => seen.has(flag))) throw new Error(usage);
const evidence = seen.get("--evidence") ?? "host-synthetic";
if (evidence !== "host-synthetic" && evidence !== "linux-amd64-buildkit")
    throw new Error("container release evidence must be host-synthetic or linux-amd64-buildkit");
const value = (flag: string) => seen.get(flag)!;
const repositoryPath = (flag: string) => {
    const path = resolve(root, value(flag));
    if (relative(root, path).startsWith(".."))
        throw new Error(`${flag} must be beneath repository root`);
    return path;
};
const selection = await Bun.file(repositoryPath("--selection")).json();
if (evidence === "linux-amd64-buildkit" && (selection as { preset?: unknown }).preset !== "analytics")
    throw new Error("linux-amd64-buildkit evidence is published only for the analytics selection");
const snapshot = evidence === "linux-amd64-buildkit"
    ? await verifyAnalyticsContainerExport(
        repositoryPath("--export-root"),
        selection,
        value("--expected-source-identity"),
        await readWorkspaceVersion(root),
    )
    : await verifyContainerExport(
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
