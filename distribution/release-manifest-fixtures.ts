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
import {
    WEB_BINDING_SLOT,
    canonicalBindingBytes,
    createWebBinding,
    readWebFiles,
    stampIndex,
} from "./selected-web-binding.ts";
import { resolveSelection } from "./resolver.ts";
import { selectedWebRoutes } from "../scripts/distribution-web-inventory-policy.ts";

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

export async function releaseFixture(
    kind: "server" | "agent" = "server",
    requestedSelection?: unknown,
) {
    const root = await mkdtemp(join(tmpdir(), "rz-manifest-"));
    const artifactRoot = join(root, "artifact");
    const webOutputRoot = join(root, "web");
    const webRoot = join(webOutputRoot, "dist");
    const apiRoot = join(root, "api");
    const schemaRoot = join(root, "schema");
    const configRoot = join(root, "selected-config");
    const nativeRoot = join(root, "native");
    const protocolRoot = join(root, "protocol");
    const selection = requestedSelection ?? (
        kind === "server"
            ? monitorSelection
            : { preset: "node-agent", target: monitorSelection.target }
    );
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
    const serverSelection = kind === "server" ? selection : monitorSelection;
    await produceSchemaContract(
        serverSelection,
        join(import.meta.dir, ".."),
        schemaRoot,
    );
    await mkdir(apiRoot, { recursive: true });
    await writeFile(
        join(apiRoot, "api.json"),
        canonicalJson(completeSelectedApiContractForTest(serverSelection)),
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
        const index = `<meta name="rustzen-web-binding" content="${WEB_BINDING_SLOT}" /><main>monitor</main>`;
        await writeFile(join(webRoot, "index.html"), index);
        await writeFixtureWebPolicyFiles(webRoot, selection);
        await writeFixtureWebBinding(webRoot, selection);
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

export async function writeFixtureWebPolicyFiles(
    webRoot: string,
    selection: unknown,
) {
    await mkdir(join(webRoot, "assets"), { recursive: true });
    await writeFile(
        join(webRoot, "assets", "main.js"),
        [
            "/api/auth/login",
            "/api/auth/me",
            "/api/monitor/",
            "/api/system/users",
            "/api/system/roles",
            "/api/system/menus/options",
            "/monitoring/overview",
            ...(resolveSelection(selection).preset === "monitor-notify"
                ? [
                      "/api/notifications/stream",
                      "/api/notifications/unread-count",
                      "Message center",
                  ]
                : []),
        ].join("\n"),
    );
    await writeFile(join(webRoot, "rustzen.png"), "fixture-logo");
}

export async function writeFixtureWebBinding(webRoot: string, selection: unknown) {
    const plan = resolveSelection(selection);
    const files = await readWebFiles(webRoot);
    const index = files.find((file) => file.path === "index.html");
    if (!index) throw new Error("fixture Web index is missing");
    const selectedApiBytes = new TextEncoder().encode("fixture-selected-api");
    const binding = createWebBinding({
        compositionId: plan.compositionId,
        selectedApiBytes,
        files,
    });
    await writeFile(join(webRoot, "index.html"), stampIndex(index.bytes, binding.webDigest));
    await writeFile(join(webRoot, "..", "binding.json"), canonicalBindingBytes(binding));
    await writeFile(join(webRoot, "..", "api.ts"), selectedApiBytes);
    const finalFiles = await readWebFiles(webRoot);
    await writeFile(
        join(webRoot, "..", "inventory.json"),
        JSON.stringify({
            schemaVersion: 2,
            preset: plan.preset,
            compositionId: plan.compositionId,
            generatedRoot: `apps/web/.selected-web/${plan.compositionId}`,
            outputDirectory: `target/distributions/${plan.compositionId}/web/dist`,
            selectedRoutes: selectedWebRoutes(plan),
            publicAssets: ["rustzen.png"],
            emittedFiles: finalFiles.map((file) => file.path),
            fileInventory: finalFiles.map(({ path, size, sha256 }) => ({ path, size, sha256 })),
            moduleIds: [
                `apps/web/.selected-web/${plan.compositionId}/index.tsx`,
            ],
            binding,
        }),
    );
}

export async function serverManifestFixture(
    selection = monitorSelection,
    binaries?: { admin: string; monitor: string },
) {
    const fixture = await releaseFixture("server", selection);
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
        selectedRoutes: selectedWebRoutes(resolveSelection(selection)),
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
    const fixture = await releaseFixture(kind, selection);
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
