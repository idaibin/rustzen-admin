import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { verifyAnalyticsContainerExport, verifyContainerExport } from "../distribution/container-export-validator.ts";
import { issueSourceBuildCertificate } from "../distribution/source-build-issuer.ts";
import { publishSourceBuildCertificate } from "../distribution/source-build-publisher.ts";
import { readReleaseKeyFile } from "../distribution/release-key-file.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const required = ["--selection", "--export-root", "--expected-source-identity", "--release-root", "--public-key", "--key-id"];
const usage = `usage: ${required.join(" ")} [--evidence <host-synthetic|linux-amd64-buildkit>]`;
const args = Bun.argv.slice(2);
if (args.length % 2 !== 0) throw new Error(usage);
const seen = new Map<string, string>();
for (let index = 0; index < args.length; index += 2) {
    const name = args[index], value = args[index + 1];
    if ((!required.includes(name) && name !== "--evidence") || seen.has(name) || !value || value.startsWith("--"))
        throw new Error("source-build certificate arguments are invalid");
    seen.set(name, value);
}
if (!required.every((flag) => seen.has(flag))) throw new Error(usage);
const evidence = seen.get("--evidence") ?? "host-synthetic";
if (evidence !== "host-synthetic" && evidence !== "linux-amd64-buildkit")
    throw new Error("source-build certificate evidence must be host-synthetic or linux-amd64-buildkit");
const value = (flag: string) => seen.get(flag)!;
const path = (flag: string) => {
    const result = resolve(root, value(flag));
    if (relative(root, result).startsWith("..")) throw new Error(`${flag} must be beneath repository root`);
    return result;
};
const selection = await Bun.file(path("--selection")).json();
if (evidence === "linux-amd64-buildkit" && (selection as { preset?: unknown }).preset !== "analytics")
    throw new Error("linux-amd64-buildkit evidence is certified only for the analytics selection");
const trusted = { keyId: value("--key-id"), publicKey: await readReleaseKeyFile(resolve(value("--public-key")), "public") };
const releaseVersion = await readWorkspaceVersion(root);
const snapshot = evidence === "linux-amd64-buildkit"
    ? await verifyAnalyticsContainerExport(path("--export-root"), selection, value("--expected-source-identity"), releaseVersion)
    : await verifyContainerExport(path("--export-root"), selection, value("--expected-source-identity"), releaseVersion);
const issued = await issueSourceBuildCertificate({ snapshot, releaseRoot: path("--release-root"), trustedRoot: root, trusted });
const published = await publishSourceBuildCertificate({ issued });
console.log(canonicalJson({ path: published.path, buildId: published.certificate.certifiedLayers.build.buildId }));
