import { chmod, copyFile, mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { completeSelectedApiContractForTest } from "./selected-contract.ts";
import { completeSelectedConfigForTest } from "./selected-config.ts";
import { produceNativeLayout } from "./native-layout.ts";
import { produceNativeStaging } from "./native-staging.ts";
import { canonicalJson, produceReleaseManifest } from "./release-manifest.ts";
import { produceSchemaContract } from "./schema-contract.ts";
import {
    produceSelectedProtocol,
    reviewedProtocolOutput,
} from "./selected-protocol.ts";

export const monitorSelection = {
    preset: "monitor",
    target: "x86_64-unknown-linux-musl",
};
export const h = (letter: string) => letter.repeat(64);
export const manifestInputs = {
    releaseVersion: "0.5.0",
    sourceIdentity: "git:abc",
    toolchain: "rustc-1.90",
    selectedRoutes: ["login", "monitor"],
};
export const manifestContractDigests = {
    apiDigest: h("a"),
    schemaDigest: h("b"),
    configDigest: h("c"),
    nativeLayoutDigest: h("d"),
    protocolArtifactDigest: h("e"),
};

export async function releaseFixture(kind: "server" | "agent" = "server") {
    const root = await mkdtemp(join(tmpdir(), "rz-manifest-"));
    const artifactRoot = join(root, "artifact");
    const webRoot = join(root, "web");
    const apiRoot = join(root, "api");
    const schemaRoot = join(root, "schema");
    const configRoot = join(root, "selected-config");
    const nativeRoot = join(root, "native");
    const protocolRoot = join(root, "protocol");
    const selection =
        kind === "server"
            ? monitorSelection
            : { preset: "node-agent", target: monitorSelection.target };
    await mkdir(configRoot, { recursive: true });
    await writeFile(
        join(configRoot, "config.json"),
        canonicalJson(completeSelectedConfigForTest(selection)),
    );
    await produceNativeLayout(selection, nativeRoot);
    await produceSelectedProtocol(
        selection,
        protocolRoot,
        reviewedProtocolOutput(),
        reviewedProtocolOutput(),
    );
    await produceSchemaContract(
        monitorSelection,
        join(import.meta.dir, ".."),
        schemaRoot,
    );
    await mkdir(apiRoot, { recursive: true });
    await writeFile(
        join(apiRoot, "api.json"),
        canonicalJson(completeSelectedApiContractForTest(monitorSelection)),
    );
    await mkdir(join(artifactRoot, "bin"), { recursive: true });
    await mkdir(join(artifactRoot, "config"), { recursive: true });
    await writeFile(join(artifactRoot, "config", "rz.env"), "PORT=3000\n");
    if (kind === "server") {
        for (const name of ["rz-admin", "rz-monitor"]) {
            await writeFile(join(artifactRoot, "bin", name), name);
            await chmod(join(artifactRoot, "bin", name), 0o755);
        }
        await mkdir(join(webRoot, "assets"), { recursive: true });
        await writeFile(join(webRoot, "index.html"), "<main>monitor</main>");
        await writeFile(join(webRoot, "assets", "main.js"), "monitor");
    } else {
        await writeFile(join(artifactRoot, "bin", "rz-monitor-agent"), "agent");
        await chmod(join(artifactRoot, "bin", "rz-monitor-agent"), 0o755);
    }
    return {
        root,
        artifactRoot,
        webRoot,
        apiRoot,
        schemaRoot,
        configRoot,
        nativeRoot,
        protocolRoot,
    };
}

export async function serverManifestFixture(
    selection = monitorSelection,
    binaries?: { admin: string; monitor: string },
) {
    const fixture = await releaseFixture();
    const binaryRoot = join(fixture.root, "staging-binary");
    await mkdir(join(binaryRoot, "bin"), { recursive: true });
    for (const name of ["rz-admin", "rz-monitor"]) {
        const source = name === "rz-admin" ? binaries?.admin : binaries?.monitor;
        if (source) await copyFile(source, join(binaryRoot, "bin", name));
        else await writeFile(join(binaryRoot, "bin", name), name);
        await chmod(join(binaryRoot, "bin", name), 0o755);
    }
    const staged = await produceNativeStaging({
        ...manifestInputs,
        selection,
        outputParent: join(fixture.root, "staged"),
        trustedRoot: fixture.root,
        binaryRoot,
        webRoot: fixture.webRoot,
        apiRoot: fixture.apiRoot,
        schemaRoot: fixture.schemaRoot,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    });
    const manifest = await produceReleaseManifest({
        ...manifestInputs,
        selection,
        staging: staged,
    });
    return { ...fixture, payloadRoot: staged.root, staging: staged, manifest };
}

export async function stagedPayloadFixture(
    kind: "server" | "agent",
    selection = kind === "server"
        ? monitorSelection
        : { preset: "node-agent", target: monitorSelection.target },
    binarySource?: string,
) {
    const fixture = await releaseFixture(kind);
    const binaryRoot = join(fixture.root, "staging-binary");
    const names =
        kind === "server" ? ["rz-admin", "rz-monitor"] : ["rz-monitor-agent"];
    await mkdir(join(binaryRoot, "bin"), { recursive: true });
    for (const name of names) {
        if (binarySource && name === "rz-monitor-agent") {
            await copyFile(binarySource, join(binaryRoot, "bin", name));
        } else {
            await writeFile(join(binaryRoot, "bin", name), name);
        }
        await chmod(join(binaryRoot, "bin", name), 0o755);
    }
    const staged = await produceNativeStaging({
        ...manifestInputs,
        selection,
        outputParent: join(fixture.root, "staged"),
        trustedRoot: fixture.root,
        binaryRoot,
        webRoot: kind === "server" ? fixture.webRoot : undefined,
        apiRoot: kind === "server" ? fixture.apiRoot : undefined,
        schemaRoot: kind === "server" ? fixture.schemaRoot : undefined,
        configRoot: fixture.configRoot,
        nativeRoot: fixture.nativeRoot,
        protocolRoot: fixture.protocolRoot,
    });
    return { ...fixture, payloadRoot: staged.root, staging: staged };
}

export async function agentManifestFixture(
    selection = { preset: "node-agent", target: monitorSelection.target },
    binarySource?: string,
) {
    const fixture = await stagedPayloadFixture("agent", selection, binarySource);
    const manifest = await produceReleaseManifest({
        ...manifestInputs,
        selection,
        staging: fixture.staging,
    });
    return { ...fixture, manifest };
}
