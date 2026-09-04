import { mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import { canonicalJson } from "./release-manifest-core.ts";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";
import {
    completeSelectedApiContractForTest,
    parseSelectedApiContract,
    produceSelectedContract,
    readSelectedApiContract,
} from "./selected-contract.ts";

const selection = { preset: "monitor", target: "x86_64-unknown-linux-musl" };
const complete = () => completeSelectedApiContractForTest(selection);
const runner = (kind: "admin" | "monitor") => complete().owners[kind];

test("selected contract accepts only the complete current registration corpus", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-contract-"));
    try {
        const out = join(root, "out");
        const a = await produceSelectedContract(selection, out, runner);
        const b = await produceSelectedContract(selection, out, runner);
        expect(a.sha256).toBe(b.sha256);
        const { contract } = await readSelectedApiContract(out, selection);
        expect(contract.owners.admin.routes).toHaveLength(20);
        expect(contract.owners.monitor.routes).toHaveLength(13);
        expect(contract.owners.monitor.menus).toHaveLength(4);
        await expect(
            produceSelectedContract({ preset: "node-agent" }, out, runner),
        ).rejects.toThrow();
        await rm(out, { recursive: true, force: true });
        await symlink(root, out);
        await expect(
            produceSelectedContract(selection, out, runner),
        ).rejects.toThrow("symlink");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("selected contract rejects every identity, owner, route, access, and menu mutation", () => {
    const mutations: Array<(value: any) => void> = [
        (v) => (v.compositionId = "0".repeat(64)),
        (v) => (v.preset = "full"),
        (v) => delete v.owners.admin,
        (v) => (v.owners.extra = {}),
        (v) => (v.owners.admin.routes = []),
        (v) => (v.owners.admin.routes[0].method = "PATCH"),
        (v) => (v.owners.admin.routes[0].path = "/api/reports/runs"),
        (v) => (v.owners.admin.routes[0].operation = "changed"),
        (v) =>
            (v.owners.admin.routes[0].access = {
                kind: "require",
                capabilities: ["system:user:delete", "extra"],
            }),
        (v) => (v.owners.monitor.module = "reports"),
        (v) => (v.owners.monitor.routes = v.owners.monitor.routes.slice(1)),
        (v) => (v.owners.monitor.routes[0].permission = "monitor:node:view"),
        (v) => (v.owners.monitor.routes[0].access = "public"),
        (v) => (v.owners.monitor.menus[0].path = "/monitoring/settings"),
        (v) => (v.owners.monitor.menus[0].extra = true),
    ];
    for (const mutate of mutations) {
        const value = structuredClone(complete());
        mutate(value);
        expect(() => parseSelectedApiContract(value, selection)).toThrow();
    }
});

test("selected API artifact must be canonical and structurally complete", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-contract-"));
    try {
        const out = join(root, "out");
        await produceSelectedContract(selection, out, runner);
        const valid = canonicalJson(complete());
        await writeFile(join(out, "api.json"), `${valid}\n`);
        await expect(readSelectedApiContract(out, selection)).rejects.toThrow(
            "canonical",
        );
        await writeFile(join(out, "api.json"), "{}");
        await expect(readSelectedApiContract(out, selection)).rejects.toThrow();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("selected API bytes are read from one stable file identity", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-contract-"));
    try {
        const out = join(root, "out");
        await produceSelectedContract(selection, out, runner);
        setArtifactReadHookForTest(async (path) => {
            if (path.endsWith("api.json")) await writeFile(path, "{}");
        });
        await expect(readSelectedApiContract(out, selection)).rejects.toThrow(
            "changed",
        );
        setArtifactReadHookForTest();

        await produceSelectedContract(selection, out, runner);
        setArtifactAfterOpenHookForTest(async (path) => {
            if (path.endsWith("api.json")) await writeFile(path, "{}");
        });
        await expect(readSelectedApiContract(out, selection)).rejects.toThrow(
            "changed",
        );
    } finally {
        setArtifactReadHookForTest();
        setArtifactAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});
