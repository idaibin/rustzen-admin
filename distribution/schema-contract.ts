import { lstat, mkdir, rm, writeFile } from "node:fs/promises";
import { basename, dirname, join, resolve } from "node:path";
import { readSingleArtifactFile } from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import { resolveSelection } from "./resolver.ts";

const sources = {
    admin: "apps/admin/migrations/sqlite-monitor/0001_init.sql",
    "admin-notifications":
        "apps/admin/migrations/sqlite-notifications/0001_notifications.sql",
    monitor: "apps/monitor/migrations/0001_init.sql",
    "monitor-notifications":
        "apps/monitor/migrations-notifications/0001_notification_outbox.sql",
} as const;
type Owner = keyof typeof sources;
export type SchemaContract = {
    compositionId: string;
    preset: "monitor" | "monitor-notify";
    owners: Partial<Record<Owner, { dataContractId: string; schemaSha256: string }>>;
};

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
    for (const owner of plan.schemaOwners as Owner[]) {
        const source = resolve(repositoryRoot, sources[owner]);
        const schemaSha256 = (
            await readSingleArtifactFile(dirname(source), basename(source))
        ).entry.sha256;
        owners[owner] = {
            schemaSha256,
            dataContractId: sha256(
                canonicalJson({ owner, version: 1, schemaSha256 }),
            ),
        };
    }
    return owners;
}

export function parseSchemaContract(
    value: unknown,
    selectionInput: unknown,
): SchemaContract {
    const plan = selectedSchemaPlan(selectionInput);
    if (!value || typeof value !== "object" || Array.isArray(value))
        throw new Error("selected schema artifact must be an object");
    const record = value as any;
    exactKeys(record, ["compositionId", "owners", "preset"]);
    if (
        record.compositionId !== plan.compositionId ||
        record.preset !== plan.preset
    )
        throw new Error("selected schema artifact selection mismatch");
    exactKeys(record.owners, [...plan.schemaOwners].sort());
    for (const owner of plan.schemaOwners as Owner[]) {
        exactKeys(record.owners[owner], ["dataContractId", "schemaSha256"]);
        const schemaSha256 = hash(record.owners[owner].schemaSha256);
        const expected = sha256(
            canonicalJson({ owner, version: 1, schemaSha256 }),
        );
        if (record.owners[owner].dataContractId !== expected)
            throw new Error("data contract ID differs from schema descriptor");
    }
    return record as SchemaContract;
}

function selectedSchemaPlan(selectionInput: unknown) {
    const plan = resolveSelection(selectionInput);
    const supportedOwners =
        plan.preset === "monitor"
            ? ["admin", "monitor"]
            : plan.preset === "monitor-notify"
              ? ["admin", "admin-notifications", "monitor", "monitor-notifications"]
              : null;
    if (
        supportedOwners === null ||
        plan.artifactClass !== "server" ||
        canonicalJson(plan.schemaOwners) !== canonicalJson(supportedOwners)
    )
        throw new Error("schema contract supports only monitor server compositions");
    return plan;
}
function exactKeys(value: any, keys: string[]) {
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
