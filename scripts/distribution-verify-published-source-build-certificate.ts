import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { verifyAnalyticsContainerExport, verifyContainerExport } from "../distribution/container-export-validator.ts";
import { issueSourceBuildCertificate } from "../distribution/source-build-issuer.ts";
import { verifyPublishedSourceBuildCertificate } from "../distribution/published-source-build-certificate.ts";
import { readReleaseKeyFile } from "../distribution/release-key-file.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const required = ["--selection", "--export-root", "--expected-source-identity", "--release-root", "--public-key", "--key-id", "--certificate"];
const args = Bun.argv.slice(2);
if (args.length % 2 !== 0 || required.some((flag) => args.filter((arg) => arg === flag).length !== 1) || args.filter((arg) => arg === "--evidence").length > 1)
    throw new Error(`usage: ${required.join(" ")} [--evidence <host-synthetic|linux-amd64-buildkit>]`);
if (args.some((arg, index) => index % 2 === 0 ? !required.includes(arg) && arg !== "--evidence" : !arg || arg.startsWith("--")))
    throw new Error("published source-build certificate arguments are invalid");
const evidence = args[args.indexOf("--evidence") + 1] ?? "host-synthetic";
if (evidence !== "host-synthetic" && evidence !== "linux-amd64-buildkit")
    throw new Error("published certificate evidence must be host-synthetic or linux-amd64-buildkit");
const value = (flag: string) => args[args.indexOf(flag) + 1]!;
const repositoryPath = (flag: string) => {
    const path = resolve(root, value(flag));
    if (relative(root, path).startsWith("..")) throw new Error(`${flag} must be beneath repository root`);
    return path;
};
const selection = await Bun.file(repositoryPath("--selection")).json();
const trusted = { keyId: value("--key-id"), publicKey: await readReleaseKeyFile(resolve(value("--public-key")), "public") };
if (evidence === "linux-amd64-buildkit" && (selection as { preset?: unknown }).preset !== "analytics")
    throw new Error("linux-amd64-buildkit evidence is verified only for the analytics selection");
const releaseVersion = await readWorkspaceVersion(root);
const snapshot = evidence === "linux-amd64-buildkit"
    ? await verifyAnalyticsContainerExport(repositoryPath("--export-root"), selection, value("--expected-source-identity"), releaseVersion)
    : await verifyContainerExport(repositoryPath("--export-root"), selection, value("--expected-source-identity"), releaseVersion);
const issued = await issueSourceBuildCertificate({ snapshot, releaseRoot: repositoryPath("--release-root"), trustedRoot: root, trusted });
console.log(canonicalJson(await verifyPublishedSourceBuildCertificate({ issued, certificatePath: repositoryPath("--certificate") })));
