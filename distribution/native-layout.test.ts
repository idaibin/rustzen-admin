import { mkdtemp, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { expect, test } from "bun:test";
import {
    setArtifactAfterOpenHookForTest,
    setArtifactReadHookForTest,
} from "./release-manifest-artifacts.ts";
import { canonicalJson, sha256 } from "./release-manifest-core.ts";
import {
    generatedNativeLayout,
    nativeUnitBytes,
    parseNativeLayout,
    produceNativeLayout,
    readNativeLayout,
} from "./native-layout.ts";
import { distributionCatalog } from "./resolver.ts";

const server = { preset: "monitor", target: "x86_64-unknown-linux-musl" };
const agent = { preset: "node-agent", target: "x86_64-unknown-linux-musl" };

const directiveValues = (unit: string, name: string): string[] =>
    unit
        .split("\n")
        .filter((line) => line.startsWith(`${name}=`))
        .map((line) => line.slice(name.length + 1));

function requireExactDirective(unit: string, name: string, value: string) {
    const actual = directiveValues(unit, name);
    if (actual.length !== 1 || actual[0] !== value)
        throw new Error(`${name} must occur once with value ${value}`);
}

function validateSelectedUnitSemantics(
    serverBytes: Record<string, string>,
    agentBytes: string,
) {
    for (const [path, identity] of [
        ["systemd/rz-admin.service", "rz-admin"],
        ["systemd/rz-monitor.service", "rz-monitor"],
    ] as const) {
        requireExactDirective(serverBytes[path], "User", identity);
        requireExactDirective(serverBytes[path], "Group", identity);
        requireExactDirective(serverBytes[path], "UMask", "0027");
        requireExactDirective(
            serverBytes[path],
            "Wants",
            "network-online.target",
        );
    }
    requireExactDirective(agentBytes, "User", "rz-monitor-agent");
    requireExactDirective(agentBytes, "Group", "rz-monitor-agent");
    requireExactDirective(agentBytes, "UMask", "0027");
    requireExactDirective(agentBytes, "Wants", "network-online.target");
    requireExactDirective(agentBytes, "StateDirectory", "rustzen-monitor-agent");
    requireExactDirective(agentBytes, "LogsDirectory", "rustzen-monitor-agent");
    requireExactDirective(
        agentBytes,
        "Environment",
        "RUSTZEN_RUNTIME_ROOT=/var/lib/rustzen-monitor-agent",
    );
    requireExactDirective(
        serverBytes["systemd/rz.target"],
        "Wants",
        "rz-admin.service rz-monitor.service",
    );
    for (const unit of [...Object.values(serverBytes), agentBytes]) {
        if (directiveValues(unit, "Requires").length !== 0)
            throw new Error("selected units must not contain Requires");
        if (directiveValues(unit, "ExecCondition").length !== 0)
            throw new Error("selected units must not contain ExecCondition");
    }
}

test("native layout exactly scopes Monitor server and Agent members", () => {
    const serverLayout = generatedNativeLayout(server);
    expect(serverLayout.units.map((x) => x.path)).toEqual([
        "systemd/rz-admin.service",
        "systemd/rz-monitor.service",
        "systemd/rz.target",
    ]);
    expect(serverLayout.configs.map((x) => x.path)).toEqual([
        "config/rz-admin.env",
        "config/rz-monitor.env",
    ]);
    const agentLayout = generatedNativeLayout(agent);
    expect(agentLayout.units.map((x) => x.path)).toEqual([
        "systemd/rz-monitor-agent.service",
    ]);
    expect(agentLayout.configs.map((x) => x.path)).toEqual([
        "config/rz-monitor-agent.env",
    ]);
    const serverBytes = nativeUnitBytes(server);
    for (const entry of serverLayout.units)
        expect(entry.sha256).toBe(sha256(serverBytes[entry.path]));
    const agentBytes =
        nativeUnitBytes(agent)["systemd/rz-monitor-agent.service"];
    expect(agentLayout.units[0].sha256).toBe(sha256(agentBytes));
    validateSelectedUnitSemantics(serverBytes, agentBytes);
    for (const [path, identity] of [
        ["systemd/rz-admin.service", "rz-admin"],
        ["systemd/rz-monitor.service", "rz-monitor"],
    ] as const) {
        expect(serverBytes[path]).toContain(
            `User=${identity}\nGroup=${identity}\nUMask=0027`,
        );
        expect(serverBytes[path]).toContain("StartLimitIntervalSec=60");
        expect(serverBytes[path]).toContain("StartLimitBurst=5");
    }
    expect(agentBytes).toContain("Wants=network-online.target");
    expect(agentBytes).toContain(
        "User=rz-monitor-agent\nGroup=rz-monitor-agent\nUMask=0027",
    );
    expect(agentBytes).not.toContain("PartOf=rz.target");
    expect(serverBytes["systemd/rz.target"]).toContain(
        "Wants=rz-admin.service rz-monitor.service\nAfter=network.target",
    );
    const selectedUnitBytes = [...Object.values(serverBytes), agentBytes].join(
        "\n",
    );
    for (const forbidden of [
        "rz-recovery",
        "rz-insights",
        "rz-reports",
        "ExecCondition",
    ])
        expect(selectedUnitBytes).not.toContain(forbidden);
});

test("native unit semantics reject duplicate override directives", () => {
    const originalServer = nativeUnitBytes(server);
    const agentBytes =
        nativeUnitBytes(agent)["systemd/rz-monitor-agent.service"];
    for (const [path, appended] of [
        ["systemd/rz-admin.service", "User=root\n"],
        ["systemd/rz-monitor.service", "UMask=0000\n"],
        ["systemd/rz.target", "Wants=rogue.service\n"],
    ] as const) {
        const mutated = { ...originalServer };
        mutated[path] += appended;
        expect(() =>
            validateSelectedUnitSemantics(mutated, agentBytes),
        ).toThrow();
    }
});

test("native layout rejects extra, missing, duplicate, stale and crossed members", () => {
    const mutations: Array<(value: any) => void> = [
        (v) => (v.compositionId = "0".repeat(64)),
        (v) => (v.artifactClass = "node-agent"),
        (v) => v.units.pop(),
        (v) => v.units.push(structuredClone(v.units[0])),
        (v) =>
            v.units.push({
                path: "systemd/rz-insights.service",
                sha256: "0".repeat(64),
            }),
        (v) => (v.configs[0].consumer = "rz-monitor-agent"),
        (v) => v.configs[0].keys.push("RUSTZEN_OTHER"),
    ];
    for (const mutate of mutations) {
        const value = structuredClone(generatedNativeLayout(server));
        mutate(value);
        expect(() => parseNativeLayout(value, server)).toThrow();
    }
});

test("native layout service basenames stay exactly bound to resolver units", () => {
    const catalog = structuredClone(distributionCatalog);
    const monitor = catalog.capabilities.find(
        (capability) => capability.id === "monitor",
    )!;
    monitor.units = ["rz-monitor.service", "rz-insights.service"];
    expect(() => generatedNativeLayout(server, catalog)).toThrow(
        "units differ",
    );
});

test("native layout is canonical, one stable non-link file", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-native-layout-"));
    const out = join(root, "out");
    try {
        await produceNativeLayout(server, out);
        const valid = canonicalJson(generatedNativeLayout(server));
        await writeFile(join(out, "native-layout.json"), `${valid}\n`);
        await expect(readNativeLayout(out, server)).rejects.toThrow(
            "canonical",
        );
        await writeFile(join(out, "extra.json"), "{}");
        await expect(readNativeLayout(out, server)).rejects.toThrow(
            "exactly native-layout.json",
        );
        await rm(out, { recursive: true, force: true });
        await symlink(root, out);
        await expect(produceNativeLayout(server, out)).rejects.toThrow(
            "symlink",
        );
    } finally {
        await rm(root, { recursive: true, force: true });
    }
});

test("native layout rejects TOCTOU replacement", async () => {
    const root = await mkdtemp(join(tmpdir(), "rz-native-layout-"));
    const out = join(root, "out");
    try {
        await produceNativeLayout(agent, out);
        setArtifactReadHookForTest(async (path) => {
            if (path.endsWith("native-layout.json"))
                await writeFile(path, "{}");
        });
        await expect(readNativeLayout(out, agent)).rejects.toThrow("changed");
        setArtifactReadHookForTest();
        await produceNativeLayout(agent, out);
        setArtifactAfterOpenHookForTest(async (path) => {
            if (path.endsWith("native-layout.json"))
                await writeFile(path, "{}");
        });
        await expect(readNativeLayout(out, agent)).rejects.toThrow("changed");
    } finally {
        setArtifactReadHookForTest();
        setArtifactAfterOpenHookForTest();
        await rm(root, { recursive: true, force: true });
    }
});
