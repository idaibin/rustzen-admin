export async function verifyReportsFlowScenarios({
    directRequest,
    expectStatus,
    responseData,
    reportsBase,
    adminBase,
}) {
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
    return { flow };
}
