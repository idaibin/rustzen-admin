import { parseNativeLayoutBytes } from "./native-layout.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { parseSelectedApiBytes } from "./selected-contract-validator.ts";
import { parseSelectedConfigBytes } from "./selected-config.ts";
import { parseSelectedProtocolBytes } from "./selected-protocol.ts";
import { parseWebBinding, verifyWebDigestBinding } from "./selected-web-binding.ts";
import { parseSchemaArtifactBytes } from "./schema-contract.ts";
import {
    readVerifiedNativeSource,
    type VerifiedNativeSource,
    type VerifiedNativeSourceState,
} from "./native-staging-source.ts";
import { resolveSelection } from "./resolver.ts";

export function verifyPublishedSource(
    source: VerifiedNativeSource,
) : VerifiedNativeSourceState {
    const state = readVerifiedNativeSource(source);
    const { files, digests, selection } = state;
    const plan = resolveSelection(selection);
    const artifactClass = plan.artifactClass;
    const paths = files.map((file) => file.entry.path);
    if (
        canonicalJson(paths) !== canonicalJson([...paths].sort(compareStagingPath)) ||
        new Set(paths).size !== paths.length ||
        files.some(
            (file) =>
                file.entry.type !== "file" ||
                !canonicalStagingPath(file.entry.path) ||
                file.entry.size !== file.bytes.byteLength ||
                sha256(file.bytes) !== file.entry.sha256 ||
                (file.entry.path.startsWith("bin/")
                    ? file.entry.mode !== "0755"
                    : file.entry.mode !== "0644"),
        )
    )
        throw new Error("staging verified source entries differ from bytes");

    const get = (path: string) => {
        const value = files.find((file) => file.entry.path === path);
        if (!value) throw new Error(`staging verified source is missing ${path}`);
        return value;
    };
    const binaries =
        artifactClass === "server"
            ? ["bin/rz-admin", "bin/rz-monitor"]
            : ["bin/rz-monitor-agent"];
    if (
        canonicalJson(paths.filter((path) => path.startsWith("bin/")).sort(compareStagingPath)) !==
        canonicalJson(binaries)
    )
        throw new Error("staging verified source binary inventory differs");

    const config = get("contracts/config/config.json");
    const native = get("contracts/native/native-layout.json");
    const protocol = get("contracts/protocol/protocol.json");
    parseSelectedConfigBytes(config.bytes, selection);
    const layout = parseNativeLayoutBytes(native.bytes, selection);
    parseSelectedProtocolBytes(protocol.bytes, selection);
    const units = files
        .filter((file) => file.entry.path.startsWith("systemd/"))
        .map((file) => ({ path: file.entry.path, sha256: file.entry.sha256 }))
        .sort((left, right) => compareStagingPath(left.path, right.path));
    if (canonicalJson(units) !== canonicalJson(layout.layout.units))
        throw new Error("staging verified source units differ from native layout");

    const fixed = [
        ...binaries,
        "contracts/config/config.json",
        "contracts/native/native-layout.json",
        "contracts/protocol/protocol.json",
        ...units.map((unit) => unit.path),
    ];
    if (artifactClass === "server") {
        parseSelectedApiBytes(get("contracts/api/api.json").bytes, selection);
        parseSchemaArtifactBytes(get("contracts/schema/schema.json").bytes, selection);
        const binding = parseWebBinding(
            JSON.parse(
                new TextDecoder("utf-8", { fatal: true }).decode(
                    get("contracts/web/binding.json").bytes,
                ),
            ),
        );
        verifyWebDigestBinding({
            binding,
            compositionId: plan.compositionId,
            files: files
                .filter((file) => file.entry.path.startsWith("web/"))
                .map((file) => ({
                    ...file.entry,
                    path: file.entry.path.slice(4),
                    bytes: file.bytes,
                })),
        });
        fixed.push(
            "contracts/api/api.json",
            "contracts/schema/schema.json",
            "contracts/web/binding.json",
        );
        if (!paths.includes("web/index.html"))
            throw new Error("staging verified source Web is empty");
        if (paths.some((path) => !fixed.includes(path) && !path.startsWith("web/")))
            throw new Error("staging verified source inventory differs");
    } else if (paths.some((path) => !fixed.includes(path))) {
        throw new Error("staging verified source inventory differs");
    }
    if (
        digests.configDigest !== config.entry.sha256 ||
        digests.nativeLayoutDigest !== native.entry.sha256 ||
        digests.protocolArtifactDigest !== protocol.entry.sha256 ||
        (artifactClass === "server" &&
            (digests.apiDigest !== get("contracts/api/api.json").entry.sha256 ||
                digests.schemaDigest !== get("contracts/schema/schema.json").entry.sha256))
    )
        throw new Error("staging verified source digests differ from contracts");
    return state;
}

export function canonicalStagingPath(path: string) {
    return (
        !!path &&
        !path.startsWith("/") &&
        !path.includes("\\") &&
        !path.includes("\0") &&
        !path.split("/").some((part) => !part || part === "." || part === "..")
    );
}

export function compareStagingPath(left: string, right: string) {
    return left < right ? -1 : left > right ? 1 : 0;
}
