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
    selectionPath?: string;
}) {
    const release = await json(input.releaseResult), root = String(release.root);
    const envelope = await json(`${root}/signature-envelope.json`);
    const key = (envelope.payload as Record<string, unknown>)?.keyId;
    if (!root || typeof key !== "string") throw Error("release verifier input differs");
    const selected = input.selectionPath ?? selectionPath;
    const preset = resolveSelection(await Bun.file(resolve(import.meta.dir, "..", selected)).json()).preset;
    const evidence = preset === "analytics" ? ["--evidence", "linux-amd64-buildkit"] : [];
    let result: ReturnType<typeof Bun.spawnSync> | undefined;
    for (let attempt = 0; attempt < 3 && result?.exitCode !== 0; attempt += 1) {
        result = Bun.spawnSync(["pnpm", "dlx", "bun@1.3.14", "scripts/distribution-verify-published-source-build-certificate.ts", "--selection", selected, "--export-root", input.exportRoot, "--expected-source-identity", input.expectedSourceIdentity, "--release-root", root, "--public-key", input.publicKey, "--key-id", key, "--certificate", input.certificate, ...evidence], { cwd: import.meta.dir + "/..", stdout: "pipe", stderr: "pipe" });
        if (attempt < 2 && result.exitCode) await Bun.sleep(1000);
    }
    if (result!.exitCode) throw Error(`signed certificate verifier failed (${result.exitCode}) for selection ${selected}: ${new TextDecoder().decode(result!.stderr)}`);
    const verified = JSON.parse(new TextDecoder().decode(result.stdout)) as Record<string, any>;
    if (verified.binaryDigests?.find((x: { path?: unknown }) => x.path === "bin/rz-admin")?.sha256 !== input.adminSha256)
        throw Error("signed admin binary differs");
    const snapshot = await verifyContainerExport(
        resolve(import.meta.dir, "..", input.exportRoot),
        await Bun.file(resolve(import.meta.dir, "..", selected)).json(),
        input.expectedSourceIdentity,
        await readWorkspaceVersion(resolve(import.meta.dir, "..")),
    );
    return { verified, snapshot };
}

/** Derives the monitor-notify composition-presence claim from a verified export snapshot. */
export function notifyCompositionPresence(snapshot: VerifiedContainerExportSnapshot) {
    const selectionInput = snapshot.selection();
    const selection = resolveSelection(selectionInput);
    if (selection.preset !== "monitor-notify" || selection.artifactClass !== "server")
        throw Error("monitor-notify presence requires the monitor-notify server selection");
    const api = parseSelectedApiBytes(snapshot.artifact("release/contracts/api/api.json").bytes, selectionInput);
    const schema = parseSchemaArtifactBytes(snapshot.artifact("release/contracts/schema/schema.json").bytes, selectionInput).contract;
    const web = parseInventory(JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(snapshot.artifact("release/web/inventory.json").bytes)));
    const apiOwners = Object.keys(api.owners);
    const schemaOwners = Object.keys(schema.owners);
    // The API contract exposes one shared "notifications" owner; the schema and
    // the selected Web graph carry the two per-module notification ledgers/shell.
    if (!apiOwners.includes("notifications"))
        throw Error("monitor-notify export lacks the notifications API owner");
    for (const owner of ["admin-notifications", "monitor-notifications"])
        if (!schemaOwners.includes(owner))
            throw Error(`monitor-notify export lacks the notification schema owner: ${owner}`);
    const notificationRoutes = web.selectedRoutes.filter((route: string) => route.includes("notifications"));
    if (!notificationRoutes.length)
        throw Error("monitor-notify export lacks notification-owned Web routes");
    return { apiOwners: apiOwners.length, schemaOwners: schemaOwners.length, notificationWebRoutes: notificationRoutes.length };
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
