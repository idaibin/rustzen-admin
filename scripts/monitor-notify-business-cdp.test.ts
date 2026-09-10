import { expect, test } from "bun:test";

import { exactPersistedSseAuthorization, isReadyNotificationStream, observeNotificationStreams, sseFetchProbeSource, streamDiagnostics } from "./monitor-notify-business-cdp.ts";

const stream = (url = "http://example.test/api/notifications/stream") => [{ requestId: "sse-1", method: "GET", url }];

test("P8f-B proves the effective fetch Authorization from the persisted browser session", () => {
    const persisted = JSON.stringify({ state: { token: "test-secret-value" } }), headers = { Authorization: "Bearer test-secret-value" };
    expect(exactPersistedSseAuthorization("http://example.test/api/notifications/stream", { headers }, persisted)).toBe(true);
    expect(exactPersistedSseAuthorization(new Request("http://example.test/api/notifications/stream", { headers }), undefined, persisted)).toBe(true);
    expect(exactPersistedSseAuthorization(new Request("http://example.test/api/notifications/stream", { headers: { Authorization: "Bearer stale" } }), { headers: new Headers(headers) }, persisted)).toBe(true);
    expect(exactPersistedSseAuthorization("http://example.test/api/notifications/stream", { headers }, JSON.stringify({ state: { token: "other-token" } }))).toBe(false);
    expect(exactPersistedSseAuthorization("http://example.test/api/notifications", { headers }, persisted)).toBe(false);
    expect(exactPersistedSseAuthorization("http://example.test/api/notifications/stream", { headers }, "not-json")).toBe(false);
    expect(sseFetchProbeSource()).toContain("new Request(input, init)");
    expect(sseFetchProbeSource()).toContain('localStorage.getItem("auth-store")');
    expect(sseFetchProbeSource()).not.toContain("test-secret-value");
    const observations = observeNotificationStreams(
        stream("http://example.test/api/notifications/stream?forbidden=query"),
        new Map([["sse-1", { status: 200, contentType: "text/event-stream; charset=utf-8" }]]),
        new Map([["sse-1", 12]]),
        true,
    );
    expect(observations[0]).toMatchObject({ bearer: true, query: true, bytes: 12 });
    expect(isReadyNotificationStream(observations[0])).toBe(false);
    const rendered = JSON.stringify(streamDiagnostics(observations));
    expect(rendered).not.toContain("test-secret-value");
    expect(rendered).not.toContain("forbidden=query");
});

test("P8f-B SSE diagnostics distinguish missing request metadata, response readiness, bytes, and Bearer availability", () => {
    expect(observeNotificationStreams([], new Map(), new Map(), false)).toEqual([]);
    const response = new Map([["sse-1", { status: 200, contentType: "text/event-stream" }]]);
    const noBearer = observeNotificationStreams(stream(), response, new Map([["sse-1", 0]]), false);
    expect(noBearer[0]).toEqual({ requestId: "sse-1", method: "GET", query: false, status: 200, eventStream: true, bytes: 0, bearer: false });
    expect(isReadyNotificationStream(noBearer[0])).toBe(false);
    const wrongResponse = observeNotificationStreams(stream(), new Map([["sse-1", { status: 204, contentType: "application/json" }]]), new Map([["sse-1", 1]]), true);
    expect(wrongResponse[0]).toMatchObject({ status: 204, eventStream: false, bytes: 1, bearer: true });
    expect(isReadyNotificationStream(wrongResponse[0])).toBe(false);
    const missingContentType = observeNotificationStreams(stream(), new Map([["sse-1", { status: 200 }]]), new Map([["sse-1", 1]]), true);
    expect(missingContentType[0]).toMatchObject({ status: 200, eventStream: false, bytes: 1, bearer: true });
    expect(isReadyNotificationStream(missingContentType[0])).toBe(false);
    const ready = observeNotificationStreams(stream(), response, new Map([["sse-1", 1]]), true);
    expect(isReadyNotificationStream(ready[0])).toBe(true);
});
