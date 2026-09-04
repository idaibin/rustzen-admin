import { expect, test } from "bun:test";
import { spawnSync } from "node:child_process";

import { nodeOnboardingCommand, normalizeControllerUrl } from "./-node-onboarding-command";

test("setup runs the agent with the exact node, controller and locally entered token", () => {
    const command = nodeOnboardingCommand("node-01", "https://admin.example.com/console/");
    const result = spawnSync(
        "bash",
        [
            "-c",
            [
                'rz-monitor-agent() { printf "%s\\n" "$RUSTZEN_MONITOR_NODE_ID" "$RUSTZEN_MONITOR_CONTROLLER_URL" "$RUSTZEN_MONITOR_AGENT_TOKEN"; }',
                command,
            ].join("\n"),
        ],
        { input: "local-test-token\n", encoding: "utf8" },
    );
    expect(result.status).toBe(0);
    expect(result.stdout.trim().split("\n")).toEqual([
        "node-01",
        "https://admin.example.com/console",
        "local-test-token",
    ]);
    expect(command).not.toContain("local-test-token");
});

test("missing token prevents agent execution", () => {
    const result = spawnSync(
        "bash",
        [
            "-c",
            "rz-monitor-agent() { echo AGENT_STARTED; };\n" +
                nodeOnboardingCommand("node-01", "https://admin.example.com"),
        ],
        { input: "\n", encoding: "utf8" },
    );
    expect(result.status).not.toBe(0);
    expect(result.stdout).not.toContain("AGENT_STARTED");
});

test("reject invalid identities and addresses instead of generating an unsafe setup", () => {
    for (const id of ["", "a".repeat(129), "node;echo bad", "节点"]) {
        expect(() => nodeOnboardingCommand(id, "https://example.com")).toThrow();
    }
    for (const url of [
        "file:///tmp/monitor",
        "https://user:secret@example.com",
        "https://example.com?q=1",
        "https://example.com/#a",
    ]) {
        expect(() => normalizeControllerUrl(url)).toThrow();
    }
});
