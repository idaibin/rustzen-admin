import { resolve } from "node:path";
import { readWorkspaceVersion } from "../distribution/workspace-version.ts";
import { verifyContainerExport, type VerifiedContainerExportSnapshot } from "../distribution/container-export-validator.ts";
import { parseSelectedApiBytes } from "../distribution/selected-contract-validator.ts";
import { parseSchemaArtifactBytes } from "../distribution/schema-contract.ts";
import { parseInventory } from "./distribution-web-inventory-schema.ts";
import { resolveSelection } from "../distribution/resolver.ts";
import { json } from "./monitor-load-admission.ts";

const selectionPath = "distribution/fixtures/monitor.json";

/** Verifies the issued source-build certificate before reading the same verified export. */
export async function signedSourceBuild(input: {
    exportRoot: string;
    expectedSourceIdentity: string;
    releaseResult: string;
    certificate: string;
    publicKey: string;
    adminSha256: string;
}) {
    const release = await json(input.releaseResult), root = String(release.root);
    const envelope = await json(`${root}/signature-envelope.json`);
    const key = (envelope.payload as Record<string, unknown>)?.keyId;
    if (!root || typeof key !== "string") throw Error("release verifier input differs");
    const result = Bun.spawnSync(["pnpm", "dlx", "bun@1.3.14", "scripts/distribution-verify-published-source-build-certificate.ts", "--selection", selectionPath, "--export-root", input.exportRoot, "--expected-source-identity", input.expectedSourceIdentity, "--release-root", root, "--public-key", input.publicKey, "--key-id", key, "--certificate", input.certificate], { cwd: import.meta.dir + "/..", stdout: "pipe", stderr: "pipe" });
    if (result.exitCode) throw Error(new TextDecoder().decode(result.stderr));
    const verified = JSON.parse(new TextDecoder().decode(result.stdout)) as Record<string, any>;
    if (verified.binaryDigests?.find((x: { path?: unknown }) => x.path === "bin/rz-admin")?.sha256 !== input.adminSha256)
        throw Error("signed admin binary differs");
    const snapshot = await verifyContainerExport(
        resolve(import.meta.dir, "..", input.exportRoot),
        await Bun.file(resolve(import.meta.dir, "..", selectionPath)).json(),
        input.expectedSourceIdentity,
        await readWorkspaceVersion(resolve(import.meta.dir, "..")),
    );
    return { verified, snapshot };
}

/**
 * Derives the pure-Monitor absence claim exclusively from a verified export snapshot.
 * Any notifications/SSE owner in the retained API, Web, schema, or service inventories
 * is a contract violation; callers cannot supply an absence boolean.
 */
export function pureMonitorAbsence(snapshot: VerifiedContainerExportSnapshot) {
    const selectionInput = snapshot.selection();
    const selection = resolveSelection(selectionInput);
    if (selection.preset !== "monitor" || selection.artifactClass !== "server")
        throw Error("pure Monitor absence requires Monitor server selection");
    const api = parseSelectedApiBytes(snapshot.artifact("release/contracts/api/api.json").bytes, selectionInput);
    const schema = parseSchemaArtifactBytes(snapshot.artifact("release/contracts/schema/schema.json").bytes, selectionInput).contract;
    const web = parseInventory(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(snapshot.artifact("release/web/inventory.json").bytes)));
    const serviceList = [...selection.services];
    const values = {
        api: Object.entries(api.owners).flatMap(([owner, value]) => [owner, ...routes(value)]),
        web: web.selectedRoutes,
        schema: Object.keys(schema.owners),
        services: serviceList,
    };
    const forbidden = Object.fromEntries(Object.entries(values).map(([owner, entries]) => [owner, entries.filter(forbiddenOwner)]));
    if (Object.values(forbidden).some((entries) => entries.length))
        throw Error(`pure Monitor export retains notifications/SSE owner: ${JSON.stringify(forbidden)}`);
    return { apiInventory: values.api.length, webInventory: values.web.length, schemaInventory: values.schema.length, serviceList, forbidden };
}

function routes(value: unknown): string[] {
    if (!value || typeof value !== "object" || Array.isArray(value)) return [];
    const entries = (value as Record<string, unknown>).routes;
    return Array.isArray(entries) ? entries.flatMap(item => typeof (item as Record<string, unknown>)?.path === "string" ? [(item as Record<string, string>).path] : []) : [];
}
function forbiddenOwner(value: string) { return /(?:^|[/_.-])(?:notifications?|sse)(?:$|[/_.-])/i.test(value); }
