import { expect, test } from "bun:test";
import { resolveSelection } from "./resolver.ts";
import {
    syntheticServerBuildCommands,
    syntheticServerInventory,
} from "./selected-server-synthetic-export-plan.ts";

const target = "x86_64-unknown-linux-musl";
test("Analytics synthetic export has exact Admin and Insights closure", () => {
    const plan = resolveSelection({ preset: "analytics", target });
    expect(syntheticServerInventory(plan)).toEqual({
        binaries: ["bin/rz-admin", "bin/rz-insights"],
        hasAgentWitness: false,
    });
    expect(syntheticServerBuildCommands(plan)).toEqual([
        [
            "env",
            "RUSTFLAGS=-C target-feature=+crt-static",
            "cargo",
            "build",
            "--release",
            "--target",
            target,
            "-p",
            "rustzen-admin",
            "--no-default-features",
            "--features",
            "analytics-distribution",
            "--bin",
            "rz-admin",
        ],
        [
            "env",
            "RUSTFLAGS=-C target-feature=+crt-static",
            "cargo",
            "build",
            "--release",
            "--target",
            target,
            "-p",
            "rustzen-insights",
            "--no-default-features",
            "--features",
            "selected-distribution",
            "--bin",
            "rz-insights",
        ],
    ]);
    expect(() =>
        syntheticServerBuildCommands(
            resolveSelection({ preset: "reports", target }),
        ),
    ).toThrow();
});

test("selected server plans fail closed outside exact ordered identities", () => {
    const monitor = resolveSelection({ preset: "monitor", target });
    expect(syntheticServerInventory(monitor)).toEqual({
        binaries: ["bin/rz-admin", "bin/rz-monitor"],
        hasAgentWitness: true,
    });
    const notify = resolveSelection({ preset: "monitor-notify", target });
    expect(syntheticServerBuildCommands(notify)).toEqual([
        [
            "env",
            "RUSTFLAGS=-C target-feature=+crt-static",
            "cargo",
            "build",
            "--release",
            "--target",
            target,
            "-p",
            "rustzen-admin",
            "--no-default-features",
            "--features",
            "monitor-distribution,notifications",
        ],
        [
            "env",
            "RUSTFLAGS=-C target-feature=+crt-static",
            "cargo",
            "build",
            "--release",
            "--target",
            target,
            "-p",
            "rustzen-monitor",
            "--no-default-features",
            "--features",
            "notifications",
            "--bin",
            "rz-monitor",
        ],
        [
            "env",
            "RUSTFLAGS=-C target-feature=+crt-static",
            "cargo",
            "build",
            "--release",
            "--target",
            target,
            "-p",
            "rustzen-monitor",
            "--no-default-features",
            "--features",
            "agent",
            "--bin",
            "rz-monitor-agent",
        ],
    ]);
    for (const selection of [
        { preset: "full", target },
        { preset: "reports", target },
        { preset: "node-agent", target },
        { preset: "analytics", target: "aarch64-unknown-linux-gnu" },
        { preset: "custom", capabilities: ["access", "insights"], target },
    ])
        expect(() =>
            syntheticServerBuildCommands(resolveSelection(selection)),
        ).toThrow("exact selected server");
    const reordered = { ...monitor, capabilities: ["monitor", "access"] };
    expect(() => syntheticServerBuildCommands(reordered)).toThrow(
        "exact selected server",
    );
    const changed = { ...monitor, compositionId: "0".repeat(64) };
    expect(() => syntheticServerBuildCommands(changed)).toThrow(
        "exact selected server",
    );
    expect(() =>
        syntheticServerBuildCommands({
            ...monitor,
            artifactClass: "agent" as unknown as "server",
        }),
    ).toThrow("exact selected server");
    expect(() =>
        syntheticServerBuildCommands({ ...monitor, capabilities: ["access"] }),
    ).toThrow("exact selected server");
    expect(() =>
        syntheticServerBuildCommands({
            ...monitor,
            target: "aarch64-unknown-linux-gnu",
        } as any),
    ).toThrow("exact selected server");
});
