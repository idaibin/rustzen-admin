import { createHmac, randomUUID } from "node:crypto";

import { gatewayLatencyResult } from "./gateway-latency-contract.mjs";

import { insightsAPIContract } from "../apps/web/src/api/insights/contract.ts";
import { monitorAPIContract } from "../apps/web/src/api/monitor/contract.ts";
import { reportsAPIContract } from "../apps/web/src/api/reports/contract.ts";
import { verifyMonitoringScenarios } from "./verify-monitoring-scenarios.mjs";
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

const verificationProjectKey = "verify-project-key";
const verificationOrigin = "https://app.example";
const collectionPolicyUpdate = (collectionEnabled) =>
    directRequest(insightsBase, "insights", "/api/insights/collection-policy", "insights:manage", {
        method: "PUT",
        body: JSON.stringify({
            collectionEnabled,
            projectKey: verificationProjectKey,
            allowedOrigins: [verificationOrigin],
        }),
    });

const disabledPolicy = await responseData(
    await expectStatus(
        await collectionPolicyUpdate(false),
        200,
        "Insights authenticated collection policy setup (disabled)",
    ),
    "Insights authenticated collection policy setup (disabled)",
);
if (disabledPolicy.collectionEnabled || !disabledPolicy.projectConfigured) {
    throw new Error(`unexpected disabled Insights policy: ${JSON.stringify(disabledPolicy)}`);
}

await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/track", "public", {
        method: "POST",
        headers: {
            "x-rustzen-project-key": verificationProjectKey,
            origin: verificationOrigin,
        },
        body: JSON.stringify({
            eventName: "page_view",
            visitorId: "visitor-before-enable",
            pagePath: "/verify",
        }),
    }),
    403,
    "Insights collection rejects events while disabled",
);

const enabledPolicy = await responseData(
    await expectStatus(
        await collectionPolicyUpdate(true),
        200,
        "Insights authenticated collection policy setup (enabled)",
    ),
    "Insights authenticated collection policy setup (enabled)",
);
if (!enabledPolicy.collectionEnabled || !enabledPolicy.projectConfigured) {
    throw new Error(`unexpected enabled Insights policy: ${JSON.stringify(enabledPolicy)}`);
}

const preflight = await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/track", "public", {
        method: "OPTIONS",
        headers: {
            origin: "HTTPS://APP.EXAMPLE:443/",
            "access-control-request-method": "POST",
            "access-control-request-headers": "content-type, x-rustzen-project-key",
        },
    }),
    204,
    "Insights allowed CORS preflight",
);
if (
    preflight.headers.get("access-control-allow-origin") !== verificationOrigin ||
    preflight.headers.get("access-control-allow-methods") !== "POST" ||
    preflight.headers.get("access-control-allow-headers") !==
        "content-type, x-rustzen-project-key" ||
    preflight.headers.get("vary") !== "Origin"
) {
    throw new Error("Insights allowed CORS preflight did not return the bounded policy headers");
}
const deniedPreflight = await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/track", "public", {
        method: "OPTIONS",
        headers: {
            origin: "https://not-allowed.example",
            "access-control-request-method": "POST",
            "access-control-request-headers": "content-type, x-rustzen-project-key",
        },
    }),
    403,
    "Insights denied CORS preflight",
);
if (
    deniedPreflight.headers.get("access-control-allow-origin") ||
    deniedPreflight.headers.get("access-control-allow-headers") ||
    deniedPreflight.headers.get("vary") !== "Origin"
) {
    throw new Error("Insights denied CORS preflight leaked an allow header");
}

const acceptedResponse = await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/track", "public", {
        method: "POST",
        headers: {
            "x-rustzen-project-key": verificationProjectKey,
            origin: verificationOrigin,
        },
        body: JSON.stringify([
            {
                eventName: "page_view",
                visitorId: "visitor-a",
                platform: "web",
                pagePath: "/verify",
                durationMs: 12,
            },
            {
                eventName: "api_request",
                visitorId: "visitor-a",
                platform: "web",
                apiPath: "/api/verify",
                apiMethod: "GET",
                statusCode: 500,
                durationMs: 42,
                isError: true,
            },
            {
                eventName: "custom_export",
                visitorId: "visitor-b",
                platform: "web",
                pagePath: "/verify",
                properties: { feature: "contract", result: "ok" },
            },
        ]),
    }),
    200,
    "Insights batch event write",
);
if (acceptedResponse.headers.get("access-control-allow-origin") !== verificationOrigin) {
    throw new Error("Insights allowed POST did not echo the verified origin");
}
const accepted = await responseData(acceptedResponse, "Insights batch event write");
if (accepted.accepted !== 3) {
    throw new Error(`unexpected Insights accepted count: ${JSON.stringify(accepted)}`);
}

const invalidEvent = await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/track", "public", {
        method: "POST",
        headers: {
            "x-rustzen-project-key": verificationProjectKey,
            origin: verificationOrigin,
        },
        body: JSON.stringify({
            eventName: "unknown",
            visitorId: "visitor-invalid-event",
            pagePath: "/verify",
        }),
    }),
    422,
    "Insights allowed CORS business error",
);
if (invalidEvent.headers.get("access-control-allow-origin") !== verificationOrigin) {
    throw new Error("Insights allowed business error did not echo the verified origin");
}

const deniedOrigin = await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/track", "public", {
        method: "POST",
        headers: {
            "x-rustzen-project-key": verificationProjectKey,
            origin: "https://not-allowed.example",
        },
        body: JSON.stringify({
            eventName: "page_view",
            visitorId: "visitor-bad-origin",
            pagePath: "/verify",
        }),
    }),
    403,
    "Insights collection rejects a non-allowed origin",
);
if (
    deniedOrigin.headers.get("access-control-allow-origin") ||
    deniedOrigin.headers.get("access-control-allow-headers") ||
    deniedOrigin.headers.get("vary") !== "Origin"
) {
    throw new Error("Insights denied POST leaked an allow header");
}

const overview = await responseData(
    await expectStatus(
        await directRequest(
            insightsBase,
            "insights",
            "/api/insights/overview",
            "insights:overview:view",
        ),
        200,
        "Insights overview",
    ),
    "Insights overview",
);
if (
    overview.pv !== 1 ||
    overview.uv !== 2 ||
    overview.eventCount !== 3 ||
    overview.requestCount !== 1 ||
    overview.errorCount !== 1 ||
    overview.p95DurationMs !== 42
) {
    throw new Error(`unexpected Insights overview: ${JSON.stringify(overview)}`);
}

const details = await responseData(
    await expectStatus(
        await directRequest(
            insightsBase,
            "insights",
            "/api/insights/events",
            "insights:event:view",
        ),
        200,
        "Insights details query",
    ),
    "Insights details query",
);
if (!details.success || details.total !== 3) {
    throw new Error(`unexpected Insights details: ${JSON.stringify(details)}`);
}
for (const event of details.data ?? []) {
    for (const field of ["pagePath", "apiPath", "referrer"]) {
        if (typeof event[field] === "string" && /[?#]/.test(event[field])) {
            throw new Error(`Insights details exposed an unsafe ${field}: ${event[field]}`);
        }
    }
}

const trackerScript = await expectStatus(
    await directRequest(insightsBase, "insights", "/api/insights/tracker.js", "public"),
    200,
    "Insights tracker script",
);
if (!trackerScript.headers.get("content-type")?.startsWith("application/javascript")) {
    throw new Error("Insights tracker did not return JavaScript content type");
}

const reportTarget = await responseData(
    await expectStatus(
        await directRequest(
            reportsBase,
            "reports",
            "/api/reports/systems",
            "reports:system:manage",
            {
                method: "POST",
                body: JSON.stringify({
                    name: "Verification fixture",
                    baseUrl: adminBase,
                    notes: "Contract fixture",
                }),
            },
        ),
        200,
        "Report target creation",
    ),
    "Report target creation",
);
const flow = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", "/api/reports/flows", "reports:flow:manage", {
            method: "POST",
            body: JSON.stringify({
                systemId: reportTarget.id,
                name: "Fixture template",
                steps: [
                    { action: "goto", url: "/health" },
                    { action: "assertText", selector: "body", text: "ok" },
                ],
            }),
        }),
        200,
        "Report template creation",
    ),
    "Report template creation",
);
await expectStatus(
    await directRequest(reportsBase, "reports", "/api/reports/flows", "reports:flow:manage", {
        method: "POST",
        body: JSON.stringify({
            systemId: reportTarget.id,
            name: "Cross origin",
            steps: [{ action: "goto", url: "https://example.com" }],
        }),
    }),
    400,
    "Report cross-origin rejection",
);
const reportRun = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", "/api/reports/runs", "reports:run:manage", {
            method: "POST",
            body: JSON.stringify({ flowId: flow.id, input: {} }),
        }),
        200,
        "Report filling run creation",
    ),
    "Report filling run creation",
);
if (reportRun.status !== "queued") throw new Error("Report filling run was not queued");

await expectStatus(
    await directRequest(reportsBase, "reports", `/api/reports/runs/${reportRun.id}/retry`, "reports:run:view", {
        method: "POST",
    }),
    403,
    "report run view capability cannot retry",
);

const reportsRuntimeRoot = required("RUSTZEN_RUNTIME_ROOT");
const reportsDatabase = `${reportsRuntimeRoot}/data/reports/db/reports.db`;
const scheduleSettings = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", "/api/reports/settings", "reports:schedule:view"),
        200,
        "schedule installation settings",
    ),
    "schedule installation settings",
);
if (scheduleSettings.timezone !== "UTC") {
    throw new Error(`worker verifier requires its UTC fixture timezone, got ${scheduleSettings.timezone}`);
}

const utcParts = (date) => ({
    dueTime: `${String(date.getUTCHours()).padStart(2, "0")}:${String(date.getUTCMinutes()).padStart(2, "0")}`,
    weekday: (date.getUTCDay() + 6) % 7,
});
const waitForScheduleDecision = async (id, decision) => {
    const path = `/api/reports/schedules/${id}`;
    for (let attempt = 0; attempt < 360; attempt += 1) {
        const current = await responseData(
            await expectStatus(
                await directRequest(reportsBase, "reports", path, "reports:schedule:view"),
                200,
                `read ${decision} schedule`,
            ),
            `read ${decision} schedule`,
        );
        if (current.lastOccurrence?.decision === decision) return current;
        await Bun.sleep(250);
    }
    throw new Error(`schedule ${id} did not record ${decision} within 90 seconds`);
};
const backdateScheduleFixture = (id, dueTime) => {
    // This disposable service-verification database makes a post-downtime slot
    // deterministic. HTTP owns every public schedule transition and readback.
    const result = Bun.spawnSync([
        "sqlite3",
        reportsDatabase,
        `UPDATE automation_schedules SET due_time='${dueTime}', effective_at='2000-01-01T00:00:00+00:00' WHERE id='${id}';`,
    ]);
    if (result.exitCode !== 0) {
        throw new Error(`could not prepare missed schedule fixture: ${result.stderr.toString()}`);
    }
};
const createSchedule = async (cadence, input) => responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", "/api/reports/schedules", "reports:schedule:manage", {
            method: "POST",
            body: JSON.stringify(input),
        }),
        200,
        `${cadence} schedule creation`,
    ),
    `${cadence} schedule creation`,
);
const scheduleList = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", "/api/reports/schedules", "reports:schedule:view"),
        200,
        "schedule list",
    ),
    "schedule list",
);
if (!Array.isArray(scheduleList)) throw new Error("schedule list was not an array");

const nextMinute = new Date(Date.now() + 60_000);
nextMinute.setUTCSeconds(0, 0);
const dailyInput = { flowId: flow.id, cadence: "daily", weekday: null, dueTime: utcParts(nextMinute).dueTime, input: {}, enabled: false };
const daily = await createSchedule("daily", dailyInput);
if (daily.enabled || daily.nextDue !== null || daily.cadence !== "daily") {
    throw new Error("daily: disabled schedule state mismatch");
}
const dailyPath = `/api/reports/schedules/${daily.id}`;
const readDaily = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", dailyPath, "reports:schedule:view"),
        200,
        "daily schedule read",
    ),
    "daily schedule read",
);
if (readDaily.id !== daily.id) throw new Error("daily schedule readback mismatched its created id");
await expectStatus(await directRequest(reportsBase, "reports", dailyPath, "reports:schedule:view", {
    method: "PUT", body: JSON.stringify(dailyInput),
}), 403, "schedule read capability cannot mutate");
const enabledDaily = await responseData(await expectStatus(await directRequest(
    reportsBase, "reports", dailyPath, "reports:schedule:manage",
    { method: "PUT", body: JSON.stringify({ ...dailyInput, enabled: true }) },
), 200, "enable daily schedule"), "enabled daily schedule");
if (!enabledDaily.enabled || !enabledDaily.nextDue || !enabledDaily.timezone) {
    throw new Error("daily: enabled schedule lacks next occurrence/timezone");
}
const enqueuedDaily = await waitForScheduleDecision(daily.id, "enqueued");
if (!enqueuedDaily.lastOccurrence?.runId || enqueuedDaily.lastRun?.id !== enqueuedDaily.lastOccurrence.runId) {
    throw new Error(`daily enqueued occurrence lost its run linkage: ${JSON.stringify(enqueuedDaily)}`);
}
const sourceRunId = enqueuedDaily.lastOccurrence.runId;
const sourceSnapshot = Bun.spawnSync([
    "sqlite3",
    reportsDatabase,
    `UPDATE automation_runs SET status='failed',error='fixture failure',finished_at='2000-01-01T00:00:00+00:00' WHERE id='${sourceRunId}'; SELECT flow_id || '|' || input_json FROM automation_runs WHERE id='${sourceRunId}';`,
]);
if (sourceSnapshot.exitCode !== 0) throw new Error(`could not prepare retry fixture: ${sourceSnapshot.stderr.toString()}`);
const retrySourceSnapshot = sourceSnapshot.stdout.toString().trim();
const retriedRun = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", `/api/reports/runs/${sourceRunId}/retry`, "reports:run:manage", {
            method: "POST",
        }),
        200,
        "retry failed report run",
    ),
    "retry failed report run",
);
if (retriedRun.id === sourceRunId || retriedRun.status !== "queued") {
    throw new Error(`retry did not return an independent queued run: ${JSON.stringify(retriedRun)}`);
}
const retryCheck = Bun.spawnSync([
    "sqlite3",
    reportsDatabase,
    `SELECT flow_id || '|' || input_json FROM automation_runs WHERE id='${retriedRun.id}'; SELECT run_id FROM automation_schedule_occurrences WHERE schedule_id='${daily.id}'; SELECT COUNT(*) FROM automation_schedule_occurrences WHERE run_id='${retriedRun.id}';`,
]);
if (retryCheck.exitCode !== 0) throw new Error(`could not inspect retry fixture: ${retryCheck.stderr.toString()}`);
const [retriedSnapshot, linkedSourceRunId, retryOccurrences] = retryCheck.stdout.toString().trim().split("\n");
if (retriedSnapshot !== retrySourceSnapshot || linkedSourceRunId !== sourceRunId || retryOccurrences !== "0") {
    throw new Error("retry changed source snapshot or schedule occurrence linkage");
}
const repeatedRetry = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", `/api/reports/runs/${sourceRunId}/retry`, "reports:run:manage", {
            method: "POST",
        }),
        200,
        "repeat retry returns the existing child",
    ),
    "repeat retry returns the existing child",
);
if (repeatedRetry.id !== retriedRun.id) {
    throw new Error(`repeat retry created a different child: ${JSON.stringify(repeatedRetry)}`);
}
const terminalRetryFixture = Bun.spawnSync([
    "sqlite3",
    reportsDatabase,
    `UPDATE automation_runs SET status='failed',error='retry fixture failure',finished_at='2000-01-01T00:00:00+00:00' WHERE id='${retriedRun.id}';`,
]);
if (terminalRetryFixture.exitCode !== 0) throw new Error(`could not finish retry fixture: ${terminalRetryFixture.stderr.toString()}`);
const terminalRepeatedRetry = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", `/api/reports/runs/${sourceRunId}/retry`, "reports:run:manage", {
            method: "POST",
        }),
        200,
        "terminal child remains the source retry result",
    ),
    "terminal child remains the source retry result",
);
if (terminalRepeatedRetry.id !== retriedRun.id || terminalRepeatedRetry.status !== "failed") {
    throw new Error(`terminal retry child was replaced: ${JSON.stringify(terminalRepeatedRetry)}`);
}
const chainedRetry = await responseData(
    await expectStatus(
        await directRequest(reportsBase, "reports", `/api/reports/runs/${retriedRun.id}/retry`, "reports:run:manage", {
            method: "POST",
        }),
        200,
        "retry terminal child",
    ),
    "retry terminal child",
);
if (chainedRetry.id === retriedRun.id) throw new Error("terminal retry child did not create a new chain link");
await expectStatus(
    await directRequest(reportsBase, "reports", `/api/reports/runs/${chainedRetry.id}/retry`, "reports:run:manage", {
        method: "POST",
    }),
    409,
    "retry non-terminal report run",
);

const missedMoment = new Date(Date.now() - 120_000);
const weeklyParts = utcParts(missedMoment);
const weeklyInput = { flowId: flow.id, cadence: "weekly", weekday: weeklyParts.weekday, dueTime: weeklyParts.dueTime, input: {}, enabled: true };
const weekly = await createSchedule("weekly", weeklyInput);
backdateScheduleFixture(weekly.id, weeklyParts.dueTime);
const skippedWeekly = await waitForScheduleDecision(weekly.id, "skipped");
if (skippedWeekly.lastOccurrence?.reason !== "missed" || skippedWeekly.lastOccurrence.runId || skippedWeekly.lastRun) {
    throw new Error(`weekly missed occurrence had an invalid run relationship: ${JSON.stringify(skippedWeekly)}`);
}
const disabledWeekly = await responseData(await expectStatus(await directRequest(
    reportsBase, "reports", `/api/reports/schedules/${weekly.id}`, "reports:schedule:manage",
    { method: "PUT", body: JSON.stringify({ ...weeklyInput, enabled: false }) },
), 200, "disable weekly schedule"), "disabled weekly schedule");
if (disabledWeekly.enabled || disabledWeekly.nextDue !== null) throw new Error("weekly schedule did not disable");

for (const id of [daily.id, weekly.id]) {
    const path = `/api/reports/schedules/${id}`;
    await expectStatus(await directRequest(reportsBase, "reports", path, "reports:schedule:manage", {
        method: "DELETE",
    }), 200, "remove verification schedule");
    await expectStatus(await directRequest(reportsBase, "reports", path, "reports:schedule:view"), 404, "removed schedule");
}
console.log("Reports daily/weekly schedule CRUD, idempotent retry chains, permissions, occurrence decisions and run linkage verified");

function percentile(values, quantile) {
    const sorted = [...values].sort((left, right) => left - right);
    return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * quantile) - 1)];
}

async function timedRequest(kind) {
    const directHeaders =
        kind === "direct"
            ? delegatedHeaders("monitor", "/api/monitor/nodes", "monitor:node:view")
            : undefined;
    const start = performance.now();
    const response =
        kind === "direct"
            ? await fetch(`${monitorBase}/api/monitor/nodes`, { headers: directHeaders })
            : await fetch(`${adminBase}/api/monitor/nodes`, {
                  headers: { authorization: `Bearer ${adminToken}` },
              });
    await expectStatus(response, 200, `${kind} latency request`);
    await response.arrayBuffer();
    return performance.now() - start;
}

async function runBatch(kind, concurrency) {
    return Promise.all(Array.from({ length: concurrency }, () => timedRequest(kind)));
}

const concurrency = 32;
for (let batch = 0; batch < 4; batch += 1) {
    await runBatch("direct", concurrency);
    await runBatch("gateway", concurrency);
}

const directSamples = [];
const gatewaySamples = [];
for (let batch = 0; batch < 10; batch += 1) {
    directSamples.push(...(await runBatch("direct", concurrency)));
    gatewaySamples.push(...(await runBatch("gateway", concurrency)));
}

const direct = {
    p50Ms: percentile(directSamples, 0.5),
    p95Ms: percentile(directSamples, 0.95),
    p99Ms: percentile(directSamples, 0.99),
};
const gateway = {
    p50Ms: percentile(gatewaySamples, 0.5),
    p95Ms: percentile(gatewaySamples, 0.95),
    p99Ms: percentile(gatewaySamples, 0.99),
};
const overhead = {
    p50Ms: gateway.p50Ms - direct.p50Ms,
    p95Ms: gateway.p95Ms - direct.p95Ms,
    p99Ms: gateway.p99Ms - direct.p99Ms,
};
const latency = {
    measuredAt: new Date().toISOString(),
    endpoint: "GET /api/monitor/nodes",
    ...gatewayLatencyResult(latencyProfile, overhead.p95Ms),
    host: "127.0.0.1",
    concurrency,
    warmupRequestsPerPath: concurrency * 4,
    sampleRequestsPerPath: directSamples.length,
    direct,
    gateway,
    overhead,
};
await Bun.write(latencyOutput, `${JSON.stringify(latency, null, 2)}\n`);
console.log(`Gateway latency: ${JSON.stringify(latency)}`);
if (latency.budgetEnforced && !latency.budgetPassed) {
    throw new Error(
        `gateway p95 overhead ${overhead.p95Ms.toFixed(3)} ms exceeds ${latency.p95BudgetMs} ms`,
    );
}

console.log("Frontend API, Monitor, Insights, Reports, delegation, and gateway contracts verified");
