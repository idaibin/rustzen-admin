import { rm } from "node:fs/promises";
import { expect, test } from "bun:test";
import {
    produceReleaseManifest,
    validateServerAgentPair,
} from "./release-manifest.ts";
import {
    h,
    manifestInputs,
    monitorSelection,
    stagedPayloadFixture,
    serverManifestFixture,
} from "./release-manifest-fixtures.ts";

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
