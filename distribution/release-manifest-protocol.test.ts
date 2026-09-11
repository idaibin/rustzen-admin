import { rm } from "node:fs/promises";
import { expect, test } from "bun:test";
import {
    parseReleaseManifest,
    produceReleaseManifest,
    validateServerAgentPair,
} from "./release-manifest.ts";
import {
    analyticsSelection,
    h,
    manifestInputs,
    monitorSelection,
    stagedPayloadFixture,
    serverManifestFixture,
} from "./release-manifest-fixtures.ts";

test("Analytics manifest binds the delegation protocol and rejects Agent pairing", async () => {
    const analytics = await serverManifestFixture(analyticsSelection);
    const agentFixture = await stagedPayloadFixture("agent");
    try {
        if (analytics.manifest.artifactClass !== "server")
            throw new Error("Analytics fixture is not a server manifest");
        const server = analytics.manifest;
        expect(Object.hasOwn(server, "agentProtocolContractId")).toBeTrue();
        expect(server.protocolArtifactDigest).toMatch(/^[a-f0-9]{64}$/);
        expect(server.binaryDigests.map(({ path }) => path)).toEqual([
            "bin/rz-admin",
            "bin/rz-insights",
        ]);
        const agent = await produceReleaseManifest({
            ...manifestInputs,
            selection: { preset: "node-agent", target: monitorSelection.target },
            staging: agentFixture.staging,
        });
        if (agent.artifactClass !== "node-agent")
            throw new Error("Agent fixture is not a node-agent manifest");
        expect(agent.agentProtocolContractId).not.toBe(server.agentProtocolContractId);
        expect(() => validateServerAgentPair(server, agent)).toThrow(
            "protocol IDs do not match",
        );
        expect(() =>
            parseReleaseManifest(
                { ...server, agentProtocolContractId: "not-a-hash" },
                analyticsSelection,
            ),
        ).toThrow();
    } finally {
        await rm(analytics.root, { recursive: true, force: true });
        await rm(agentFixture.root, { recursive: true, force: true });
    }
});

test("server and Agent manifests bind the derived protocol pairing ID", async () => {
    const server = await serverManifestFixture();
    const agentFixture = await stagedPayloadFixture("agent");
    try {
        const agent = await produceReleaseManifest({
            ...manifestInputs,
            selection: {
                preset: "node-agent",
                target: monitorSelection.target,
            },
            staging: agentFixture.staging,
        });
        if (
            server.manifest.artifactClass !== "server" ||
            agent.artifactClass !== "node-agent"
        )
            throw new Error("fixture release classes differ from expectation");
        const serverManifest = server.manifest;
        const agentManifest = agent;
        expect(serverManifest.agentProtocolContractId).toBe(
            agentManifest.agentProtocolContractId,
        );
        expect(serverManifest.protocolArtifactDigest).not.toBe(
            agentManifest.protocolArtifactDigest,
        );
        validateServerAgentPair(serverManifest, agentManifest);
        agentManifest.agentProtocolContractId = h("9");
        expect(() =>
            validateServerAgentPair(serverManifest, agentManifest),
        ).toThrow("protocol");
    } finally {
        await rm(server.root, { recursive: true, force: true });
        await rm(agentFixture.root, { recursive: true, force: true });
    }
});
