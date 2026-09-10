import { expect, test } from "bun:test";

import { hasExactBearerAuthorization, isReadyNotificationStream, observeNotificationStreams, streamDiagnostics } from "./monitor-notify-business-cdp.ts";

const stream = (url = "http://example.test/api/notifications/stream") => [{ requestId: "sse-1", method: "GET", url }];

test("P8f-B correlates Bearer evidence from CDP extra-info without retaining its value", () => {
    const headers = { Authorization: "Bearer test-secret-value" };
    expect(hasExactBearerAuthorization(headers, "test-secret-value")).toBe(true);
    expect(hasExactBearerAuthorization(headers, "other-token")).toBe(false);
    const observations = observeNotificationStreams(
        stream("http://example.test/api/notifications/stream?forbidden=query"),
        new Map([["sse-1", { status: 200, contentType: "text/event-stream; charset=utf-8" }]]),
        new Map([["sse-1", 12]]),
        new Map([["sse-1", hasExactBearerAuthorization(headers, "test-secret-value")]]),
    );
    expect(observations[0]).toMatchObject({ bearer: true, query: true, bytes: 12 });
    expect(isReadyNotificationStream(observations[0])).toBe(false);
    const rendered = JSON.stringify(streamDiagnostics(observations));
    expect(rendered).not.toContain("test-secret-value");
    expect(rendered).not.toContain("forbidden=query");
});

test("P8f-B SSE diagnostics distinguish missing request metadata, response readiness, bytes, and Bearer availability", () => {
    expect(observeNotificationStreams([], new Map(), new Map(), new Map())).toEqual([]);
    const response = new Map([["sse-1", { status: 200, contentType: "text/event-stream" }]]);
    const withoutExtraInfo = observeNotificationStreams(stream(), response, new Map([["sse-1", 0]]), new Map());
    expect(withoutExtraInfo[0]).toEqual({ requestId: "sse-1", method: "GET", query: false, status: 200, eventStream: true, bytes: 0, bearer: null });
    expect(isReadyNotificationStream(withoutExtraInfo[0])).toBe(false);
    const wrongResponse = observeNotificationStreams(stream(), new Map([["sse-1", { status: 204, contentType: "application/json" }]]), new Map([["sse-1", 1]]), new Map([["sse-1", true]]));
    expect(wrongResponse[0]).toMatchObject({ status: 204, eventStream: false, bytes: 1, bearer: true });
    expect(isReadyNotificationStream(wrongResponse[0])).toBe(false);
    const missingContentType = observeNotificationStreams(stream(), new Map([["sse-1", { status: 200 }]]), new Map([["sse-1", 1]]), new Map([["sse-1", true]]));
    expect(missingContentType[0]).toMatchObject({ status: 200, eventStream: false, bytes: 1, bearer: true });
    expect(isReadyNotificationStream(missingContentType[0])).toBe(false);
    const ready = observeNotificationStreams(stream(), response, new Map([["sse-1", 1]]), new Map([["sse-1", true]]));
    expect(isReadyNotificationStream(ready[0])).toBe(true);
});
