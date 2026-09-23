import { mkdir, mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";
import {
    parseSchemaContract,
    produceSchemaContract,
    readSchemaContract,
} from "./schema-contract.ts";

const selection = { preset: "monitor", target: "x86_64-unknown-linux-musl" };
const notifySelection = {
    preset: "monitor-notify",
    target: "x86_64-unknown-linux-musl",
};
const analyticsSelection = {
    preset: "analytics",
    target: "x86_64-unknown-linux-musl",
};

test("analytics binds exactly the Admin and Insights fresh schemas", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-analytics-"));
    try {
        const result = await produceSchemaContract(
            analyticsSelection,
            join(import.meta.dir, ".."),
            join(root, "out"),
        );
        expect(result.contract.preset).toBe("analytics");
        expect(Object.keys(result.contract.owners)).toEqual(["admin", "insights"]);
        const mutations: Array<(value: Record<string, unknown>) => void> = [
            (value) => delete (value.owners as Record<string, unknown>).insights,
            (value) => ((value.owners as Record<string, unknown>).monitor = {}),
            (value) => (
                ((value.owners as Record<string, Record<string, unknown>>).admin.schemaSha256 =
                    "0".repeat(64))
            ),
        ];
        for (const mutate of mutations) {
            const value = structuredClone(result.contract) as Record<string, unknown>;
            mutate(value);
            expect(() => parseSchemaContract(value, analyticsSelection)).toThrow();
        }
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("monitor-notify binds the optional Admin inbox fragment", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-notify-"));
    try {
        const result = await produceSchemaContract(
            notifySelection,
            join(import.meta.dir, ".."),
            join(root, "out"),
        );
        expect(result.contract.preset).toBe("monitor-notify");
        expect(Object.keys(result.contract.owners)).toEqual([
            "admin",
            "admin-notifications",
            "monitor",
            "monitor-notifications",
        ]);
        const missing = structuredClone(result.contract);
        delete missing.owners["admin-notifications"];
        expect(() => parseSchemaContract(missing, notifySelection)).toThrow();
        const missingMonitor = structuredClone(result.contract);
        delete missingMonitor.owners["monitor-notifications"];
        expect(() => parseSchemaContract(missingMonitor, notifySelection)).toThrow();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("schema contract is derived from both fresh-install migrations", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-"));
    try {
        const out = join(root, "out");
        const first = await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            out,
        );
        const second = await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            out,
        );
        expect(first.sha256).toBe(second.sha256);
        expect(Object.keys(first.contract.owners)).toEqual([
            "admin",
            "monitor",
        ]);
        for (const value of Object.values(first.contract.owners)) {
            expect(value.schemaSha256).toMatch(/^[a-f0-9]{64}$/);
            expect(value.dataContractId).toMatch(/^[a-f0-9]{64}$/);
        }
        await expect(
            produceSchemaContract(
                { preset: "node-agent" },
                join(import.meta.dir, ".."),
                out,
            ),
        ).rejects.toThrow();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("schema contract rejects structural and identity mutations", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-"));
    try {
        const out = join(root, "out");
        const { contract } = await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            out,
        );
        const mutations: Array<(value: Record<string, unknown>) => void> = [
            (v) => (v.preset = "full"),
            (v) => (v.compositionId = "0".repeat(64)),
            (value) => delete (value.owners as Record<string, unknown>).admin,
            (value) => ((value.owners as Record<string, unknown>).extra = {}),
            (value) => (
                ((value.owners as Record<string, Record<string, unknown>>).admin.schemaSha256 =
                    "0".repeat(64))
            ),
            (value) => (
                ((value.owners as Record<string, Record<string, unknown>>).monitor.dataContractId =
                    "0".repeat(64))
            ),
            (value) => (value.extra = true),
        ];
        for (const mutate of mutations) {
            const value = structuredClone(contract) as Record<string, unknown>;
            mutate(value);
            expect(() => parseSchemaContract(value, selection)).toThrow();
        }
        await writeFile(
            join(out, "schema.json"),
            `${canonicalJson(contract)}\n`,
        );
        await expect(readSchemaContract(out, selection)).rejects.toThrow(
            "canonical",
        );
        const forged = structuredClone(contract);
        for (const owner of ["admin", "monitor"] as const) {
            forged.owners[owner].schemaSha256 = "0".repeat(64);
            forged.owners[owner].dataContractId = sha256(
                canonicalJson({
                    owner,
                    version: 1,
                    schemaSha256: forged.owners[owner].schemaSha256,
                }),
            );
        }
        await writeFile(join(out, "schema.json"), canonicalJson(forged));
        await expect(readSchemaContract(out, selection)).rejects.toThrow(
            "fresh-install SQL",
        );
        await writeFile(join(out, "extra.json"), "{}");
        await expect(readSchemaContract(out, selection)).rejects.toThrow(
            "exactly schema.json",
        );
        await rm(out, { recursive: true, force: true });
        await symlink(root, out);
        await expect(
            produceSchemaContract(selection, join(import.meta.dir, ".."), out),
        ).rejects.toThrow("symlink");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("schema producer refuses linked or changing migration sources", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-source-"));
    const out = join(root, "out");
    const adminDir = join(root, "apps/admin/migrations/sqlite-monitor");
    const monitorDir = join(root, "apps/monitor/migrations");
    try {
        await mkdir(adminDir, { recursive: true });
        await mkdir(monitorDir, { recursive: true });
        await writeFile(join(root, "external.sql"), "CREATE TABLE forged(id);");
        await symlink(
            join(root, "external.sql"),
            join(adminDir, "0001_init.sql"),
        );
        await writeFile(
            join(monitorDir, "0001_init.sql"),
            "CREATE TABLE m(id);",
        );
        await expect(
            produceSchemaContract(selection, root, out),
        ).rejects.toThrow("symlink");
        await rm(join(adminDir, "0001_init.sql"));
        await writeFile(join(adminDir, "0001_init.sql"), "CREATE TABLE a(id);");
        setArtifactAfterOpenHookForTest(async (path) => {
            if (path.endsWith("sqlite-monitor/0001_init.sql"))
                await writeFile(path, "CREATE TABLE x(id);");
        });
        await expect(
            produceSchemaContract(selection, root, out),
        ).rejects.toThrow("changed");
    } finally {
        setArtifactAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});

test("analytics schema producer refuses a linked Insights migration source", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-analytics-source-"));
    const out = join(root, "out");
    const adminDir = join(root, "apps/admin/migrations/sqlite-analytics");
    const insightsDir = join(root, "apps/insights/migrations");
    try {
        await mkdir(adminDir, { recursive: true });
        await mkdir(insightsDir, { recursive: true });
        await writeFile(join(adminDir, "0001_init.sql"), "CREATE TABLE a(id);");
        await writeFile(join(root, "external.sql"), "CREATE TABLE forged(id);");
        await symlink(
            join(root, "external.sql"),
            join(insightsDir, "0001_init.sql"),
        );
        await expect(
            produceSchemaContract(analyticsSelection, root, out),
        ).rejects.toThrow("symlink");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("schema bytes use one stable file identity", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-schema-"));
    try {
        const out = join(root, "out");
        await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            out,
        );
        setArtifactReadHookForTest(async (path) => {
            if (path.endsWith("schema.json")) await writeFile(path, "{}");
        });
        await expect(readSchemaContract(out, selection)).rejects.toThrow(
            "changed",
        );
        setArtifactReadHookForTest();
        await produceSchemaContract(
            selection,
            join(import.meta.dir, ".."),
            out,
        );
        setArtifactAfterOpenHookForTest(async (path) => {
            if (path.endsWith("schema.json")) await writeFile(path, "{}");
        });
        await expect(readSchemaContract(out, selection)).rejects.toThrow(
            "changed",
        );
    } finally {
        setArtifactReadHookForTest();
        setArtifactAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});
