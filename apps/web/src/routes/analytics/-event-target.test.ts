import { expect, test } from "bun:test";

import { projectEventTarget } from "./-event-target";

test("event target projection excludes non-display event data", () => {
    const projection = projectEventTarget({
        id: 1,
        eventName: "page_view",
        visitorId: "visitor-secret",
        userId: "user-secret",
        sessionId: "session-secret",
        platform: "web",
        pagePath: "/analytics/details",
        referrer: "/analytics/overview",
        apiPath: null,
        apiMethod: null,
        statusCode: null,
        durationMs: null,
        isError: false,
        properties: { secret: "properties-secret" },
        occurredAt: "2026-09-05T00:00:00Z",
        receivedAt: "2026-09-05T00:00:00Z",
        extra: "extra-secret",
    } as Insights.Event);

    expect(projection).toEqual({
        kind: "page",
        eventName: "page_view",
        pagePath: "/analytics/details",
        referrer: "/analytics/overview",
        apiPath: null,
        apiMethod: null,
    });
    expect(JSON.stringify(projection)).not.toContain("secret");
});
