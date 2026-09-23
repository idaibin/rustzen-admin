import { readFile } from "node:fs/promises";
import { URL } from "node:url";
import { runInNewContext } from "node:vm";

import { describe, expect, test } from "bun:test";

const TRACKER_SOURCE = await readFile(new URL("./tracker.js", import.meta.url), "utf8");
const TRACKING_ENDPOINT = "https://collect.example/api/insights/track";

class MemoryStorage {
    #values = new Map();

    getItem(key) {
        return this.#values.get(String(key)) ?? null;
    }

    setItem(key, value) {
        this.#values.set(String(key), String(value));
    }

    removeItem(key) {
        this.#values.delete(String(key));
    }

    dump() {
        return Object.fromEntries(this.#values.entries());
    }
}

class FakeXMLHttpRequest {
    constructor() {
        this.status = 0;
        this.listeners = new Map();
    }

    open(method, url, ...rest) {
        this.opened = { method, url, rest };
        return undefined;
    }

    send(...args) {
        this.sent = args;
        return undefined;
    }

    addEventListener(type, listener) {
        const listeners = this.listeners.get(type) ?? [];
        listeners.push(listener);
        this.listeners.set(type, listeners);
    }

    emit(type) {
        for (const listener of this.listeners.get(type) ?? []) listener.call(this, { type });
    }
}

class FakeRequest {
    constructor(url, init = {}) {
        this.url = String(url);
        this.method = String(init.method ?? "GET").toUpperCase();
    }
}

function response(status) {
    return { status, ok: status >= 200 && status < 300 };
}

function createHarness() {
    const localStorage = new MemoryStorage();
    const sessionStorage = new MemoryStorage();
    const fetchCalls = [];
    const intervals = new Map();
    const timeouts = [];
    const pagehideListeners = [];
    const eventListeners = new Map();
    const clock = { now: 1_000, performance: 10 };
    let nextTimerId = 1;
    let uuid = 0;
    let fetchResponder = async () => response(200);

    const nativeFetch = async (input, init) => {
        const call = { input, init };
        fetchCalls.push(call);
        return fetchResponder(call);
    };
    const window = { fetch: nativeFetch };
    const document = {
        currentScript: {
            src: "https://cdn.example/assets/tracker.js",
            dataset: { endpoint: TRACKING_ENDPOINT, projectKey: "script-project-key" },
        },
        referrer: "https://ref.example/from?private=1#fragment",
    };
    const location = new URL("https://app.example/dashboard?private=1#fragment");
    const addEventListener = (type, listener) => {
        if (type === "pagehide") pagehideListeners.push(listener);
        const listeners = eventListeners.get(type) ?? [];
        listeners.push(listener);
        eventListeners.set(type, listeners);
    };
    const removeEventListener = (type, listener) => {
        const listeners = eventListeners.get(type) ?? [];
        eventListeners.set(
            type,
            listeners.filter((current) => current !== listener),
        );
    };
    const setInterval = (callback, delay) => {
        const id = nextTimerId++;
        intervals.set(id, { callback, delay });
        return id;
    };
    const clearInterval = (id) => intervals.delete(id);
    const setTimeout = (callback, delay) => {
        const id = nextTimerId++;
        timeouts.push({ id, callback, delay });
        return id;
    };
    const clearTimeout = (id) => {
        const index = timeouts.findIndex((timer) => timer.id === id);
        if (index >= 0) timeouts.splice(index, 1);
    };
    const timers = {
        intervals,
        timeouts,
        async tick() {
            for (const { callback } of [...intervals.values()]) callback();
            await flush();
        },
        async runTimeouts() {
            while (timeouts.length > 0) {
                const timer = timeouts.shift();
                timer.callback();
                await flush();
            }
        },
    };
    const flush = async () => {
        for (let index = 0; index < 8; index += 1) await Promise.resolve();
    };
    const triggerPagehide = async () => {
        for (const listener of [...pagehideListeners]) listener({ type: "pagehide" });
        await flush();
    };
    const crypto = { randomUUID: () => `visitor-session-${++uuid}` };
    const NativeDate = Date;
    class TestDate extends NativeDate {
        constructor(...args) {
            super(...(args.length > 0 ? args : [clock.now]));
        }

        static now() {
            return clock.now;
        }
    }
    const context = {
        URL,
        Request: FakeRequest,
        TextEncoder,
        XMLHttpRequest: FakeXMLHttpRequest,
        addEventListener,
        clearInterval,
        clearTimeout,
        console,
        crypto,
        Date: TestDate,
        document,
        location,
        localStorage,
        navigator: { platform: "test-platform", userAgentData: { platform: "test-platform" } },
        performance: { now: () => clock.performance },
        removeEventListener,
        sessionStorage,
        setInterval,
        setTimeout,
        window,
    };
    window.addEventListener = addEventListener;
    window.removeEventListener = removeEventListener;
    window.crypto = crypto;
    window.XMLHttpRequest = FakeXMLHttpRequest;
    runInNewContext(TRACKER_SOURCE, context, { filename: "tracker.js" });

    return {
        clock,
        document,
        fetchCalls,
        localStorage,
        nativeFetch,
        originalFetch: nativeFetch,
        originalOpen: FakeXMLHttpRequest.prototype.open,
        originalSend: FakeXMLHttpRequest.prototype.send,
        runTimeouts: timers.runTimeouts,
        sessionStorage,
        setFetchResponder(responder) {
            fetchResponder = responder;
        },
        tick: timers.tick,
        timers,
        triggerPagehide,
        transportCalls() {
            return fetchCalls.filter((call) => call.input === TRACKING_ENDPOINT);
        },
        window,
        XMLHttpRequest: FakeXMLHttpRequest,
    };
}

function decodeBatch(harness, index = 0) {
    const call = harness.transportCalls()[index];
    expect(call).toBeDefined();
    return JSON.parse(call.init.body);
}

describe("Insights tracker browser contract", () => {
    test("stays inert until consent, isolates host hooks, and clears state on opt-out", async () => {
        const harness = createHarness();
        const tracker = harness.window.rustzenAnalytics;

        expect(tracker).toBeDefined();
        expect(harness.localStorage.dump()).toEqual({});
        expect(harness.sessionStorage.dump()).toEqual({});
        expect(harness.fetchCalls).toHaveLength(0);
        expect(harness.timers.intervals.size).toBe(0);
        expect(harness.window.fetch).toBe(harness.originalFetch);
        expect(harness.XMLHttpRequest.prototype.open).toBe(harness.originalOpen);
        expect(harness.XMLHttpRequest.prototype.send).toBe(harness.originalSend);

        expect(tracker.enable({ consent: false, projectKey: "ignored" })).toBe(false);
        expect(harness.localStorage.dump()).toEqual({});
        expect(harness.sessionStorage.dump()).toEqual({});
        expect(harness.window.fetch).toBe(harness.originalFetch);

        const transportEvents = [];
        expect(
            tracker.enable({
                consent: true,
                endpoint: TRACKING_ENDPOINT,
                onTransportEvent: (event) => transportEvents.push(event),
                projectKey: "public-project-key",
            }),
        ).toBe(true);
        expect(harness.localStorage.dump()).toHaveProperty("rz_vid", "visitor-session-1");
        expect(harness.sessionStorage.dump()).toHaveProperty("rz_sid", "visitor-session-2");
        expect(harness.timers.intervals.size).toBe(1);
        expect(harness.window.fetch).not.toBe(harness.originalFetch);
        expect(harness.XMLHttpRequest.prototype.open).not.toBe(harness.originalOpen);
        expect(harness.XMLHttpRequest.prototype.send).not.toBe(harness.originalSend);
        expect(harness.fetchCalls).toHaveLength(0);

        const xhr = new harness.XMLHttpRequest();
        xhr.open("GET", "/api/items?secret=1#fragment");
        xhr.status = 200;
        xhr.send();
        xhr.emit("loadend");
        await harness.triggerPagehide();
        expect(harness.transportCalls()).toHaveLength(1);

        tracker.optOut();
        expect(harness.localStorage.dump()).toEqual({});
        expect(harness.sessionStorage.dump()).toEqual({});
        expect(harness.timers.intervals.size).toBe(0);
        expect(harness.window.fetch).toBe(harness.originalFetch);
        expect(harness.XMLHttpRequest.prototype.open).toBe(harness.originalOpen);
        expect(harness.XMLHttpRequest.prototype.send).toBe(harness.originalSend);
        await harness.triggerPagehide();
        await harness.tick();
        expect(harness.transportCalls()).toHaveLength(1);
        expect(transportEvents.some((event) => event.type === "accepted")).toBe(true);
    });

    test("rotates visitor/session IDs and strips query/hash from collected paths", async () => {
        const harness = createHarness();
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({ consent: true, endpoint: TRACKING_ENDPOINT, projectKey: "project-key" });
        await harness.triggerPagehide();

        const initialVisitor = harness.localStorage.getItem("rz_vid");
        const initialSession = harness.sessionStorage.getItem("rz_sid");
        const firstBatch = decodeBatch(harness);
        expect(firstBatch[0]).toMatchObject({ pagePath: "/dashboard", referrer: "/from" });
        expect(firstBatch[0].pagePath).not.toMatch(/[?#]/);
        expect(firstBatch[0].referrer).not.toMatch(/[?#]/);

        tracker.track("custom_export", {
            apiPath: "https://api.example/items?token=private#fragment",
            pagePath: "/reports?secret=1#fragment",
            properties: { feature: "export", secret: "drop" },
            referrer: "https://ref.example/source?private=1#fragment",
        });
        expect(harness.localStorage.getItem("rz_vid")).toBe(initialVisitor);
        expect(harness.sessionStorage.getItem("rz_sid")).toBe(initialSession);
        await harness.triggerPagehide();
        const secondBatch = decodeBatch(harness, 1);
        expect(secondBatch[0]).toMatchObject({
            apiPath: "/items",
            pagePath: "/reports",
            referrer: "/source",
            properties: { feature: "export" },
        });
        expect(secondBatch[0].apiPath).not.toMatch(/[?#]/);
        expect(secondBatch[0].pagePath).not.toMatch(/[?#]/);
        expect(secondBatch[0].referrer).not.toMatch(/[?#]/);
        expect(secondBatch[0].properties.secret).toBeUndefined();

        harness.clock.now += 30 * 60 * 1000 + 1;
        tracker.track("custom_export");
        expect(harness.localStorage.getItem("rz_vid")).toBe(initialVisitor);
        expect(harness.sessionStorage.getItem("rz_sid")).not.toBe(initialSession);

        harness.clock.now += 24 * 60 * 60 * 1000;
        tracker.track("custom_export");
        expect(harness.localStorage.getItem("rz_vid")).not.toBe(initialVisitor);
        expect(harness.sessionStorage.getItem("rz_sid")).not.toBe(initialSession);
    });

    test("drops the newest event when the local queue reaches its fixed limit", async () => {
        const harness = createHarness();
        const events = [];
        let resolveFirst;
        harness.setFetchResponder(
            () =>
                new Promise((resolve) => {
                    resolveFirst = resolve;
                }),
        );
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            onTransportEvent: (event) => events.push(event),
            projectKey: "queue-project-key",
        });

        for (let index = 0; index < 49; index += 1) tracker.track("custom_export");
        for (let index = 0; index < 1000; index += 1) tracker.track("custom_export");
        expect(tracker.track("custom_export")).toBe(false);
        expect(events).toContainEqual(
            expect.objectContaining({
                type: "queue_dropped",
                reason: "queue_full",
                dropped: 1,
                queueLength: 1000,
            }),
        );

        resolveFirst(response(200));
        await harness.tick();
        await harness.tick();
        expect(harness.transportCalls()).toHaveLength(2);
    });

    test("rejects overlong single events before queueing", () => {
        const harness = createHarness();
        const events = [];
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            onTransportEvent: (event) => events.push(event),
            projectKey: "field-project-key",
        });

        expect(
            tracker.track("api_request", { apiPath: `/${"x".repeat(2000)}` }),
        ).toBe(false);
        expect(
            tracker.track("custom_export", { properties: { feature: "x".repeat(257) } }),
        ).toBe(false);
        expect(events).toEqual([
            { type: "event_dropped", reason: "field_too_long", field: "apiPath" },
            { type: "event_dropped", reason: "field_too_long", field: "properties" },
        ]);
        expect(harness.transportCalls()).toHaveLength(0);
    });

    test("uses UTF-8 byte bounds for property strings", () => {
        const harness = createHarness();
        const events = [];
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            onTransportEvent: (event) => events.push(event),
            projectKey: "utf8-project-key",
        });

        expect(
            tracker.track("custom_export", { properties: { feature: "😀".repeat(64) } }),
        ).toBe(true);
        expect(
            tracker.track("custom_export", { properties: { feature: "😀".repeat(65) } }),
        ).toBe(false);
        expect(
            tracker.track("custom_export", { properties: { feature: "中".repeat(85) } }),
        ).toBe(true);
        expect(
            tracker.track("custom_export", { properties: { feature: "中".repeat(86) } }),
        ).toBe(false);
        expect(events).toEqual([
            { type: "event_dropped", reason: "field_too_long", field: "properties" },
            { type: "event_dropped", reason: "field_too_long", field: "properties" },
        ]);
    });

    test("uses UTF-8 byte bounds for the public project key", () => {
        const acceptedHarness = createHarness();
        expect(
            acceptedHarness.window.rustzenAnalytics.enable({
                consent: true,
                endpoint: TRACKING_ENDPOINT,
                projectKey: "😀".repeat(64),
            }),
        ).toBe(true);

        const rejectedHarness = createHarness();
        expect(
            rejectedHarness.window.rustzenAnalytics.enable({
                consent: true,
                endpoint: TRACKING_ENDPOINT,
                projectKey: "😀".repeat(65),
            }),
        ).toBe(false);
        expect(rejectedHarness.localStorage.dump()).toEqual({});
        expect(rejectedHarness.sessionStorage.dump()).toEqual({});
    });

    test("caps each JSON request at the server body limit while delivering long events", async () => {
        const harness = createHarness();
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            projectKey: "body-size-project-key",
        });
        await harness.triggerPagehide();

        const paths = [];
        for (let index = 0; index < 50; index += 1) {
            const suffix = String(index).padStart(8, "0");
            const pagePath = `/${"x".repeat(1991)}${suffix}`;
            paths.push(pagePath);
            expect(tracker.track("custom_export", { pagePath })).toBe(true);
        }
        for (let index = 0; index < 5; index += 1) await harness.tick();

        const calls = harness.transportCalls();
        expect(calls.length).toBeGreaterThan(1);
        for (const call of calls) {
            expect(new TextEncoder().encode(call.init.body).length).toBeLessThanOrEqual(64 * 1024);
            expect(JSON.parse(call.init.body).length).toBeLessThanOrEqual(50);
        }
        const delivered = calls
            .flatMap((call) => JSON.parse(call.init.body))
            .filter((event) => event.eventName === "custom_export")
            .map((event) => event.pagePath);
        expect(delivered).toHaveLength(paths.length);
        expect(new Set(delivered)).toEqual(new Set(paths));
    });

    test("reports invalid status, duration, and type fields accurately", () => {
        const harness = createHarness();
        const events = [];
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            onTransportEvent: (event) => events.push(event),
            projectKey: "invalid-field-project-key",
        });

        expect(tracker.track("api_request", { statusCode: 99 })).toBe(false);
        expect(tracker.track("api_request", { durationMs: -1 })).toBe(false);
        expect(tracker.track("api_request", { isError: "yes" })).toBe(false);
        expect(tracker.track("api_request", { apiMethod: {} })).toBe(false);
        expect(tracker.track("custom_export", { properties: { feature: {} } })).toBe(false);
        expect(tracker.track("custom_export", { properties: { feature: [] } })).toBe(false);
        expect(events).toEqual([
            { type: "event_dropped", reason: "invalid_field", field: "statusCode" },
            { type: "event_dropped", reason: "invalid_field", field: "durationMs" },
            { type: "event_dropped", reason: "invalid_field", field: "isError" },
            { type: "event_dropped", reason: "invalid_field", field: "apiMethod" },
            { type: "event_dropped", reason: "invalid_field", field: "properties" },
            { type: "event_dropped", reason: "invalid_field", field: "properties" },
        ]);
    });

    test("does not let an invalid event poison a valid mixed batch", async () => {
        const harness = createHarness();
        const events = [];
        harness.setFetchResponder(async (call) => {
            const batch = JSON.parse(call.init.body);
            return batch.some((event) => typeof event.apiMethod !== "undefined" &&
                typeof event.apiMethod !== "string")
                ? response(422)
                : response(200);
        });
        const tracker = harness.window.rustzenAnalytics;
        tracker.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            onTransportEvent: (event) => events.push(event),
            projectKey: "mixed-batch-project-key",
        });

        expect(tracker.track("api_request", { apiPath: "/api/items", apiMethod: {} })).toBe(false);
        expect(
            tracker.track("api_request", {
                apiPath: "/api/items",
                apiMethod: "GET",
                statusCode: 200,
            }),
        ).toBe(true);
        await harness.triggerPagehide();

        expect(harness.transportCalls()).toHaveLength(1);
        expect(JSON.parse(harness.transportCalls()[0].init.body)).toEqual(
            expect.arrayContaining([
                expect.objectContaining({ eventName: "page_view" }),
                expect.objectContaining({ eventName: "api_request", apiMethod: "GET" }),
            ]),
        );
        expect(events).toContainEqual({ type: "event_dropped", reason: "invalid_field", field: "apiMethod" });
        expect(events).toContainEqual(expect.objectContaining({ type: "accepted", status: 200 }));
    });

    test("retries only explicit 429/507 responses within a bounded attempt count", async () => {
        for (const [status, statuses, expectedCalls, expectedAccepted, expectedDelays] of [
            [429, [429, 429, 429, 429, 200], 4, false, [500, 1000, 2000]],
            [507, [507, 507, 200], 3, true, [500, 1000]],
        ]) {
            const harness = createHarness();
            const events = [];
            const remaining = [...statuses];
            harness.setFetchResponder(async () => response(remaining.shift() ?? 200));
            harness.window.rustzenAnalytics.enable({
                consent: true,
                endpoint: TRACKING_ENDPOINT,
                onTransportEvent: (event) => events.push(event),
                projectKey: `project-${status}`,
            });

            await harness.tick();
            await harness.runTimeouts();
            expect(harness.transportCalls()).toHaveLength(expectedCalls);
            const retryEvents = events.filter((event) => event.type === "temporary_retry");
            expect(retryEvents).toHaveLength(expectedCalls - 1);
            expect(retryEvents.map((event) => event.delayMs)).toEqual(expectedDelays);
            expect(events.some((event) => event.type === "temporary_dropped")).toBe(
                !expectedAccepted,
            );
            expect(events.some((event) => event.type === "accepted")).toBe(expectedAccepted);
            await harness.tick();
            expect(harness.transportCalls()).toHaveLength(expectedCalls);
        }
    });

    test("reports a 413 validation rejection once without retry", async () => {
        const harness = createHarness();
        const events = [];
        harness.setFetchResponder(async () => response(413));
        harness.window.rustzenAnalytics.enable({
            consent: true,
            endpoint: TRACKING_ENDPOINT,
            onTransportEvent: (event) => events.push(event),
            projectKey: "payload-project-key",
        });

        await harness.tick();
        expect(harness.transportCalls()).toHaveLength(1);
        expect(events).toContainEqual(
            expect.objectContaining({ type: "validation_rejected", status: 413, attempt: 0 }),
        );
        await harness.runTimeouts();
        await harness.tick();
        expect(harness.transportCalls()).toHaveLength(1);
    });

    test("drops ambiguous 5xx and network failures without retry or requeue", async () => {
        for (const [label, responder, reason] of [
            ["5xx", async () => response(503), "ambiguous_response"],
            ["network", async () => Promise.reject(new Error("offline")), "network_error"],
        ]) {
            const harness = createHarness();
            const events = [];
            harness.setFetchResponder(responder);
            harness.window.rustzenAnalytics.enable({
                consent: true,
                endpoint: TRACKING_ENDPOINT,
                onTransportEvent: (event) => events.push(event),
                projectKey: `project-${label}`,
            });

            await harness.tick();
            await harness.runTimeouts();
            expect(harness.transportCalls()).toHaveLength(1);
            expect(events).toContainEqual(
                expect.objectContaining({ type: "temporary_dropped", reason }),
            );
            await harness.tick();
            expect(harness.transportCalls()).toHaveLength(1);
        }
    });
});
