import { relative, resolve } from "node:path";
import { canonicalJson } from "../distribution/release-manifest-core.ts";
import { verifyContainerExport } from "../distribution/container-export-validator.ts";
import { issueSourceBuildCertificate } from "../distribution/source-build-issuer.ts";
import { publishSourceBuildCertificate } from "../distribution/source-build-publisher.ts";
import { readReleaseKeyFile } from "../distribution/release-key-file.ts";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";

const root = resolve(import.meta.dir, "..");
const required = ["--selection", "--export-root", "--expected-source-identity", "--release-root", "--public-key", "--key-id"];
const args = Bun.argv.slice(2);
if (args.length !== required.length * 2 || required.some((flag) => args.filter((x) => x === flag).length !== 1))
    throw new Error(`usage: ${required.join(" ")}`);
if (args.some((value, index) => index % 2 === 0 ? !required.includes(value) : !value || value.startsWith("--")))
    throw new Error("source-build certificate arguments are invalid");
const value = (flag: string) => args[args.indexOf(flag) + 1]!;
const path = (flag: string) => {
    const result = resolve(root, value(flag));
    if (relative(root, result).startsWith("..")) throw new Error(`${flag} must be beneath repository root`);
    return result;
};
const selection = await Bun.file(path("--selection")).json();
const trusted = { keyId: value("--key-id"), publicKey: await readReleaseKeyFile(resolve(value("--public-key")), "public") };
const snapshot = await verifyContainerExport(path("--export-root"), selection, value("--expected-source-identity"), await readWorkspaceVersion(root));
const issued = await issueSourceBuildCertificate({ snapshot, releaseRoot: path("--release-root"), trustedRoot: root, trusted });
const published = await publishSourceBuildCertificate({ issued });
console.log(canonicalJson({ path: published.path, buildId: published.certificate.certifiedLayers.build.buildId }));
