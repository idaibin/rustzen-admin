import { mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";
import { canonicalJson } from "./release-manifest-core.ts";
import {
    completeSelectedConfigForTest,
    parseSelectedConfig,
    produceSelectedConfig,
    readSelectedConfig,
} from "./selected-config.ts";

const serverSelection = {
    preset: "monitor",
    target: "x86_64-unknown-linux-musl",
};
const agentSelection = {
    preset: "node-agent",
    target: "x86_64-unknown-linux-musl",
};
const notifySelection = {
    preset: "monitor-notify",
    target: "x86_64-unknown-linux-musl",
};
const analyticsSelection = {
    preset: "analytics",
    target: "x86_64-unknown-linux-musl",
};
const runnerFor = (selection: unknown) => {
    const owners = completeSelectedConfigForTest(selection).owners;
    return (
        _binary: "admin" | "insights" | "monitor" | "agent",
        owner: "access" | "insights" | "monitor" | "monitor-agent" | "notifications",
    ) => owners[owner];
};

test("selected config accepts exact server and Agent descriptors", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-config-"));
    try {
        for (const [name, selection] of [
            ["server", serverSelection],
            ["server-notify", notifySelection],
            ["analytics", analyticsSelection],
            ["agent", agentSelection],
        ] as const) {
            const out = join(root, name);
            const a = await produceSelectedConfig(selection, out, runnerFor(selection));
            const b = await produceSelectedConfig(selection, out, runnerFor(selection));
            expect(a.sha256).toBe(b.sha256);
            expect(Object.keys(a.contract.owners)).toEqual(
                name === "server-notify"
                    ? ["access", "monitor", "notifications"]
                    : name === "analytics"
                      ? ["access", "insights"]
                      : name === "server"
                        ? ["access", "monitor"]
                        : ["monitor-agent"],
            );
        }
        await expect(
            produceSelectedConfig(agentSelection, join(root, "bad"), runnerFor(serverSelection)),
        ).rejects.toThrow();
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("analytics config accepts only the reviewed access and Insights owners", () => {
    const value = completeSelectedConfigForTest(analyticsSelection);
    expect(Object.keys(value.owners)).toEqual(["access", "insights"]);
    const access = value.owners.access as any;
    expect(access.fields.some((field: any) => field.key === "RUSTZEN_INSIGHTS_PORT")).toBeTrue();
    expect(access.fields.some((field: any) => field.key === "RUSTZEN_MONITOR_PORT")).toBeFalse();
    expect(access.fields.find((field: any) => field.key === "RUSTZEN_IPC_TOKEN")?.secretRef).toBe(
        "insights.ipc",
    );
    expect(() =>
        parseSelectedConfig(
            { ...value, owners: { ...value.owners, monitor: value.owners.insights } },
            analyticsSelection,
        ),
    ).toThrow("reviewed descriptors");
    expect(() =>
        parseSelectedConfig(
            {
                ...value,
                owners: {
                    ...value.owners,
                    access: completeSelectedConfigForTest(serverSelection).owners.access,
                },
            },
            analyticsSelection,
        ),
    ).toThrow("reviewed descriptors");
});

test("selected config rejects identity and every field metadata mutation", () => {
    const mutations: Array<(v: any) => void> = [
        (v) => (v.compositionId = "0".repeat(64)),
        (v) => (v.preset = "full"),
        (v) => delete v.owners.access,
        (v) => (v.owners.extra = {}),
        (v) => (v.owners.access.owner = "monitor"),
        (v) => (v.owners.access.consumer = "other"),
        (v) => (v.owners.access.fields[0].key = "RUSTZEN_UNKNOWN"),
        (v) => (v.owners.access.fields[0].valueType = "secret"),
        (v) => (v.owners.access.fields[0].required = true),
        (v) => (v.owners.access.fields[0].defaultClass = "none"),
        (v) => (v.owners.access.fields[0].secretRef = "auth.jwt"),
        (v) => (v.owners.access.fields[0].value = "canary-secret"),
        (v) => v.owners.access.fields.push(structuredClone(v.owners.access.fields[0])),
    ];
    for (const mutate of mutations) {
        const value = completeSelectedConfigForTest(serverSelection);
        mutate(value);
        expect(() => parseSelectedConfig(value, serverSelection)).toThrow();
    }
});

test("selected config requires canonical stable single-file bytes", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-config-"));
    try {
        const out = join(root, "out");
        await produceSelectedConfig(serverSelection, out, runnerFor(serverSelection));
        const valid = canonicalJson(completeSelectedConfigForTest(serverSelection));
        await writeFile(join(out, "config.json"), `${valid}\n`);
        await expect(readSelectedConfig(out, serverSelection)).rejects.toThrow("canonical");
        await writeFile(join(out, "extra.json"), "{}");
        await expect(readSelectedConfig(out, serverSelection)).rejects.toThrow(
            "exactly config.json",
        );
        await rm(out, { recursive: true, force: true });
        await symlink(root, out);
        await expect(
            produceSelectedConfig(serverSelection, out, runnerFor(serverSelection)),
        ).rejects.toThrow("symlink");
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("selected config rejects before and after open mutations", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-config-"));
    try {
        const out = join(root, "out");
        await produceSelectedConfig(serverSelection, out, runnerFor(serverSelection));
        setArtifactReadHookForTest(async (path) => {
            if (path.endsWith("config.json")) await writeFile(path, "{}");
        });
        await expect(readSelectedConfig(out, serverSelection)).rejects.toThrow("changed");
        setArtifactReadHookForTest();
        await produceSelectedConfig(serverSelection, out, runnerFor(serverSelection));
        setArtifactAfterOpenHookForTest(async (path) => {
            if (path.endsWith("config.json")) await writeFile(path, "{}");
        });
        await expect(readSelectedConfig(out, serverSelection)).rejects.toThrow("changed");
    } finally {
        setArtifactReadHookForTest();
        setArtifactAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});
