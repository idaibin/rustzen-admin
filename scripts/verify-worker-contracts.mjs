import { createHmac, randomUUID } from "node:crypto";

import { verifyGatewayLatency } from "./gateway-latency-contract.mjs";

import { insightsAPIContract } from "../apps/web/src/api/insights/contract.ts";
import { monitorAPIContract } from "../apps/web/src/api/monitor/contract.ts";
import { reportsAPIContract } from "../apps/web/src/api/reports/contract.ts";
import { verifyInsightsScenarios } from "./verify-insights-scenarios.mjs";
import { verifyMonitoringScenarios } from "./verify-monitoring-scenarios.mjs";
import { verifyReportsFlowScenarios } from "./verify-reports-flow-scenarios.mjs";
import { verifyReportsScheduleScenarios } from "./verify-reports-schedule-scenarios.mjs";
import { compareManifestRoutes } from "./worker-contract-verifier.mjs";

const ipcToken = required("RUSTZEN_IPC_TOKEN");
const agentToken = required("RUSTZEN_MONITOR_AGENT_TOKEN");
const adminToken = required("RUSTZEN_ADMIN_TOKEN");
const adminUserId = required("RUSTZEN_ADMIN_USER_ID");
const adminBase = `http://127.0.0.1:${required("RUSTZEN_ADMIN_PORT")}`;
const monitorBase = `http://127.0.0.1:${required("RUSTZEN_MONITOR_PORT")}`;
const insightsBase = `http://127.0.0.1:${required("RUSTZEN_INSIGHTS_PORT")}`;
const reportsBase = `http://127.0.0.1:${required("RUSTZEN_REPORTS_PORT")}`;
const latencyOutput = required("RUSTZEN_GATEWAY_LATENCY_OUTPUT");
const latencyProfile = required("RUSTZEN_VERIFY_BUILD_PROFILE");

function required(name) {
    const value = process.env[name]?.trim();
    if (!value) {
        throw new Error(`${name} is required`);
    }
    return value;
}

function delegatedHeaders(module, path, access, method = "GET") {
    const timestamp = Math.floor(Date.now() / 1000).toString();
    const requestId = randomUUID();
    const userId = access === "public" ? "anonymous" : adminUserId;
    const payload = ["1", timestamp, requestId, userId, module, method, path, access].join("\n");
    const signature = createHmac("sha256", ipcToken).update(payload).digest("hex");
    return {
        "x-rustzen-contract-version": "1",
        "x-rustzen-ipc-timestamp": timestamp,
        "x-rustzen-request-id": requestId,
        "x-rustzen-user-id": userId,
        "x-rustzen-module": module,
        "x-rustzen-ipc-capability": access,
        "x-rustzen-ipc-signature": signature,
    };
}

function directRequest(base, module, pathAndQuery, access, init = {}) {
    const method = init.method ?? "GET";
    const path = new URL(`${base}${pathAndQuery}`).pathname;
    return fetch(`${base}${pathAndQuery}`, {
        ...init,
        method,
        headers: {
            ...(init.body ? { "content-type": "application/json" } : {}),
            ...delegatedHeaders(module, path, access, method),
            ...init.headers,
        },
    });
}

async function expectStatus(response, status, label) {
    if (response.status !== status) {
        throw new Error(
            `${label}: expected ${status}, got ${response.status}: ${await response.text()}`,
        );
    }
    return response;
}

async function responseData(response, label) {
    const payload = await response.json();
    if (payload.code !== 0 || payload.message !== "Success") {
        throw new Error(`${label}: invalid response envelope: ${JSON.stringify(payload)}`);
    }
    return payload.data;
}

async function verifyFrontendAPIContract(module, base, clientContract) {
    const response = await expectStatus(
        await fetch(`${base}/internal/v1/manifest`),
        200,
        `${module} runtime Manifest`,
    );
    const manifest = await response.json();
    if (manifest.module !== module || typeof manifest.apiPrefix !== "string") {
        throw new Error(`${module}: invalid runtime Manifest identity`);
    }

    const { missing, extra, invalidAllowlist, clientPublicRoutes } = compareManifestRoutes(
        module,
        manifest,
        clientContract,
    );

    if (
        missing.length > 0 ||
        extra.length > 0 ||
        invalidAllowlist.length > 0 ||
        clientPublicRoutes.length > 0
    ) {
        throw new Error(
            `${module}: frontend API contract drifted from runtime Manifest: ${[
                ...missing.map(({ name, key }) => `missing ${name} (${key})`),
                ...extra.map(({ key, access }) => `unmapped ${access} (${key})`),
                ...invalidAllowlist.map(({ key, access }) => `invalid public allowlist ${key} (${access ?? "missing"})`),
                ...clientPublicRoutes.map(({ name, key }) => `client contract includes public route ${name} (${key})`),
            ].join(", ")}`,
        );
    }

    const expectedMenus = {
        monitor: [
            {
                code: "incidents",
                path: "/monitoring/incidents",
                permission: "monitor:incident:view",
            },
        ],
        reports: [
            {
                code: "schedules",
                path: "/reports/templates",
                permission: "reports:schedule:view",
            },
        ],
        insights: [],
    }[module];
    const menuByCode = new Map((manifest.menus ?? []).map((menu) => [menu.code, menu]));
    for (const expected of expectedMenus) {
        const menu = menuByCode.get(expected.code);
        if (
            !menu ||
            menu.path !== expected.path ||
            menu.permission !== expected.permission
        ) {
            throw new Error(
                `${module}: runtime Manifest menu ${expected.code} is missing or mismatched`,
            );
        }
    }
}

await Promise.all([
    verifyFrontendAPIContract("monitor", monitorBase, monitorAPIContract),
    verifyFrontendAPIContract("insights", insightsBase, insightsAPIContract),
    verifyFrontendAPIContract("reports", reportsBase, reportsAPIContract),
]);

await verifyMonitoringScenarios({ adminBase, adminToken, agentToken });

const report = {
    nodeId: "verify-agent",
    bootId: randomUUID(),
    sequence: 1,
    hostname: "verify-host",
    agentVersion: "0.5.0",
    cpuPercent: 12.5,
    memory: { usedBytes: 10, totalBytes: 20 },
    disks: [{ mountPoint: "/", usedBytes: 30, totalBytes: 40 }],
    collectedAt: new Date().toISOString(),
};

await expectStatus(
    await fetch(`${adminBase}/api/monitor/agent-reports`, {
        method: "POST",
        headers: {
            "content-type": "application/json",
            "x-rustzen-monitor-agent-token": agentToken,
        },
        body: JSON.stringify(report),
    }),
    200,
    "public Monitor agent report through Admin",
);

await expectStatus(
    await directRequest(monitorBase, "monitor", "/api/monitor/agent-reports", "public", {
        method: "POST",
        headers: { "x-rustzen-monitor-agent-token": agentToken },
        body: JSON.stringify({ ...report, nodeId: "verify-direct-agent", bootId: randomUUID() }),
    }),
    200,
    "direct delegated Monitor agent report",
);

const nodes = await responseData(
    await expectStatus(
        await directRequest(monitorBase, "monitor", "/api/monitor/nodes", "monitor:node:view"),
        200,
        "Monitor node list",
    ),
    "Monitor node list",
);
if (!nodes.some((node) => node.nodeId === "verify-agent")) {
    throw new Error("Monitor public gateway agent report was not persisted");
}
const verifyNode = nodes.find((node) => node.nodeId === "verify-agent");

const metrics = await responseData(
    await expectStatus(
        await directRequest(
            monitorBase,
            "monitor",
            `/api/monitor/nodes/${verifyNode.nodeId}/metrics?bucket=raw`,
            "monitor:node:view",
        ),
        200,
        "Monitor metric history",
    ),
    "Monitor metric history",
);
if (metrics.points?.length !== 1 || metrics.points[0].cpuPercent !== report.cpuPercent) {
    throw new Error(`unexpected Monitor metric history: ${JSON.stringify(metrics)}`);
}

await expectStatus(
    await directRequest(monitorBase, "monitor", "/api/monitor/nodes", "monitor:manage"),
    403,
    "Monitor local capability mismatch",
);

await verifyInsightsScenarios({ directRequest, expectStatus, responseData, insightsBase });

const { flow } = await verifyReportsFlowScenarios({
    directRequest,
    expectStatus,
    responseData,
    reportsBase,
    adminBase,
});
await verifyReportsScheduleScenarios({
    directRequest,
    expectStatus,
    responseData,
    reportsBase,
    reportsRuntimeRoot: required("RUSTZEN_RUNTIME_ROOT"),
    flow,
    now: () => new Date(),
    sleep: Bun.sleep,
    spawnSync: Bun.spawnSync,
});

await verifyGatewayLatency({
    directPrepare: () => {
        const headers = delegatedHeaders("monitor", "/api/monitor/nodes", "monitor:node:view");
        return () => fetch(`${monitorBase}/api/monitor/nodes`, { headers });
    },
    gatewayFetch: () => fetch(`${adminBase}/api/monitor/nodes`, {
        headers: { authorization: `Bearer ${adminToken}` },
    }),
    expectStatus,
    latencyOutput,
    latencyProfile,
});

console.log("Frontend API, Monitor, Insights, Reports, delegation, and gateway contracts verified");
