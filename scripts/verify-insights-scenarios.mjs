export async function verifyInsightsScenarios({
    directRequest,
    expectStatus,
    responseData,
    insightsBase,
}) {
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
}
