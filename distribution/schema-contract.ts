import { lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { readArtifactFiles, readSingleArtifactFile } from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";
import {
    isExactSupportedPlan,
    type SourceBuildPlan,
} from "./source-build-plan.ts";

type Owner =
    | "admin"
    | "admin-notifications"
    | "insights"
    | "monitor"
    | "monitor-notifications";
const sources: Record<"analytics" | "monitor", Partial<Record<Owner, string>>> = {
    analytics: {
        admin: "apps/admin/migrations/sqlite-analytics/0001_init.sql",
        insights: "apps/insights/migrations",
    },
    monitor: {
        admin: "apps/admin/migrations/sqlite-monitor/0001_init.sql",
        "admin-notifications":
            "apps/admin/migrations/sqlite-notifications/0001_notifications.sql",
        monitor: "apps/monitor/migrations/0001_init.sql",
        "monitor-notifications":
            "apps/monitor/migrations-notifications/0001_notification_outbox.sql",
    },
};
export type SchemaContract = {
    compositionId: string;
    preset: "analytics" | "monitor" | "monitor-notify";
    owners: Partial<Record<Owner, { dataContractId: string; schemaSha256: string }>>;
};

/** Fresh-install SQL schemas are selected only for the reviewed server closures. */
export const supportsSelectedSchema = (plan: SourceBuildPlan): boolean =>
    isExactSupportedPlan(plan, ["analytics", "monitor", "monitor-notify"]);

export async function produceSchemaContract(
    selectionInput: unknown,
    repositoryRoot: string,
    outputRoot: string,
) {
    const plan = selectedSchemaPlan(selectionInput);
    await rejectSymlink(outputRoot);
    const owners = await schemaOwners(selectionInput, repositoryRoot);
    const contract = parseSchemaContract(
        { compositionId: plan.compositionId, preset: plan.preset, owners },
        selectionInput,
    );
    await rm(outputRoot, { recursive: true, force: true });
    await mkdir(outputRoot, { recursive: true });
    await writeFile(join(outputRoot, "schema.json"), canonicalJson(contract), {
        mode: 0o644,
    });
    return readSchemaContract(outputRoot, selectionInput);
}

export async function readSchemaContract(
    root: string,
    selectionInput: unknown,
    repositoryRoot = resolve(import.meta.dir, ".."),
): Promise<{ contract: SchemaContract; sha256: string }> {
    const file = await readSingleArtifactFile(root, "schema.json");
    return parseSchemaBytes(file.bytes, selectionInput, repositoryRoot);
}

export async function parseSchemaBytes(
    bytes: Uint8Array,
    selectionInput: unknown,
    repositoryRoot = resolve(import.meta.dir, ".."),
): Promise<{ contract: SchemaContract; sha256: string }> {
    const parsed = parseSchemaArtifactBytes(bytes, selectionInput);
    const expected = await schemaOwners(selectionInput, repositoryRoot);
    if (canonicalJson(parsed.contract.owners) !== canonicalJson(expected))
        throw new Error(
            "selected schema artifact differs from fresh-install SQL",
        );
    return parsed;
}

/** Parses only the already-stable artifact bytes; it never reads repository state. */
export function parseSchemaArtifactBytes(
    bytes: Uint8Array,
    selectionInput: unknown,
): { contract: SchemaContract; sha256: string } {
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    let value: unknown;
    try {
        value = JSON.parse(text);
    } catch {
        throw new Error("selected schema artifact is not JSON");
    }
    const contract = parseSchemaContract(value, selectionInput);
    if (text !== canonicalJson(contract))
        throw new Error("selected schema artifact is not canonical");
    return { contract, sha256: sha256(bytes) };
}

async function schemaOwners(selectionInput: unknown, repositoryRoot: string) {
    const plan = selectedSchemaPlan(selectionInput);
    const owners = {} as SchemaContract["owners"];
    const presetSources = plan.preset === "analytics" ? sources.analytics : sources.monitor;
    for (const owner of plan.schemaOwners as Owner[]) {
        const relative = presetSources[owner];
        if (!relative) throw new Error(`selected schema source is unavailable: ${owner}`);
        const source = resolve(repositoryRoot, relative);
        const schemaSha256 = await schemaSourceDigest(source);
        owners[owner] = {
            schemaSha256,
            dataContractId: sha256(
                canonicalJson({ owner, version: 1, schemaSha256 }),
            ),
        };
    }
    return owners;
}

/** Single-file sources keep their file digest; directory sources bind the exact ordered migration set. */
async function schemaSourceDigest(source: string): Promise<string> {
    const stat = await lstat(source);
    if (stat.isSymbolicLink())
        throw new Error("selected schema source is a symlink");
    if (stat.isFile()) {
        return (await readSingleArtifactFile(dirname(source), basename(source))).entry.sha256;
    }
    if (!stat.isDirectory())
        throw new Error("selected schema source is unavailable");
    const files = await readArtifactFiles(source);
    if (!files.length || files.some((file) => !/^[0-9]{4}_[a-z0-9_]+\.sql$/.test(file.path)))
        throw new Error("selected schema source inventory is invalid");
    return sha256(canonicalJson(files.map((file) => ({ name: file.path, sha256: file.sha256 }))));
}

export function parseSchemaContract(
    value: unknown,
    selectionInput: unknown,
): SchemaContract {
    const plan = selectedSchemaPlan(selectionInput);
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected schema artifact must be an object");
    const record = value as Record<string, unknown>;
    exactKeys(record, ["compositionId", "owners", "preset"]);
    if (
        record.compositionId !== plan.compositionId ||
        record.preset !== plan.preset
    )
        throw new Error("selected schema artifact selection mismatch");
    exactKeys(record.owners, [...plan.schemaOwners].sort());
    for (const owner of plan.schemaOwners as Owner[]) {
        const descriptor = record.owners as Record<string, unknown>;
        exactKeys(descriptor[owner], ["dataContractId", "schemaSha256"]);
        const fields = descriptor[owner] as Record<string, unknown>;
        const schemaSha256 = hash(fields.schemaSha256);
        const expected = sha256(
            canonicalJson({ owner, version: 1, schemaSha256 }),
        );
        if (fields.dataContractId !== expected)
            throw new Error("data contract ID differs from schema descriptor");
    }
    return record as SchemaContract;
}

function selectedSchemaPlan(selectionInput: unknown) {
    const plan = resolveSelection(selectionInput);
    const supportedOwners =
        plan.preset === "analytics"
            ? ["admin", "insights"]
            : plan.preset === "monitor"
            ? ["admin", "monitor"]
            : plan.preset === "monitor-notify"
              ? ["admin", "admin-notifications", "monitor", "monitor-notifications"]
              : null;
    if (
        !supportsSelectedSchema(plan) ||
        supportedOwners === null ||
        plan.artifactClass !== "server" ||
        canonicalJson(plan.schemaOwners) !== canonicalJson(supportedOwners)
    )
        throw new Error("schema contract supports only analytics or monitor server compositions");
    return plan;
}
function exactKeys(value: unknown, keys: string[]) {
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("schema contract field must be an object");
    if (canonicalJson(Object.keys(value).sort()) !== canonicalJson(keys))
        throw new Error("selected schema artifact fields are invalid");
}
function hash(value: unknown): string {
    if (typeof value !== "string" || !/^[a-f0-9]{64}$/.test(value))
        throw new Error("schema digest must be lowercase sha256");
    return value;
}

async function rejectSymlink(path: string) {
    try {
        if ((await lstat(path)).isSymbolicLink())
            throw new Error("schema contract output root must not be symlink");
    } catch (error: unknown) {
        if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
    }
}
