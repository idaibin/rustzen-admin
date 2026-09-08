import { describe, expect, test } from "bun:test";

import { NotificationRealtimeClient, retryDelay, type StreamState } from "./realtime-client";

const response = (
    body: string,
    status = 200,
    headers: Record<string, string> = { "content-type": "text/event-stream" },
) =>
    new Response(
        new ReadableStream({
            start(controller) {
                controller.enqueue(new TextEncoder().encode(body));
                controller.close();
            },
        }),
        { status, headers },
    );

describe("NotificationRealtimeClient", () => {
    test("default fetch keeps the browser global as its receiver", async () => {
        const original = globalThis.fetch;
        let receiver: unknown;
        globalThis.fetch = async function (this: typeof globalThis) {
            receiver = this;
            return response("", 204);
        } as unknown as typeof fetch;
        try {
            const client = new NotificationRealtimeClient("secret.jwt", 1, {
                currentGeneration: () => 1,
                onInvalidate: () => {},
                onState: () => {},
                onUnauthorized: () => {},
            });
            client.start();
            await new Promise((resolve) => setTimeout(resolve, 10));
            client.stop();
            expect(receiver).toBe(globalThis);
        } finally {
            globalThis.fetch = original;
        }
    });

    test("uses bearer and Last-Event-ID headers without URL credentials", async () => {
        const calls: Array<[string, RequestInit]> = [];
        let count = 0;
        const states: StreamState[] = [];
        const client = new NotificationRealtimeClient(
            "secret.jwt",
            1,
            {
                currentGeneration: () => (count < 2 ? 1 : 2),
                onInvalidate: () => {},
                onState: (state) => states.push(state),
                onUnauthorized: () => {},
            },
            (async (url: RequestInfo | URL, init?: RequestInit) => {
                calls.push([
                    typeof url === "string" ? url : url instanceof URL ? url.href : url.url,
                    init!,
                ]);
                count += 1;
                return response(
                    `event: inbox.changed\nid: ${count}\ndata: {"schemaVersion":1,"revision":${count}}\n\n`,
                );
            }) as unknown as typeof fetch,
            () => 0,
        );
        client.start();
        await new Promise((resolve) => setTimeout(resolve, 1_100));
        client.stop();
        expect(calls[0][0]).toBe("/api/notifications/stream");
        expect(calls[0][0]).not.toContain("secret.jwt");
        expect(new Headers(calls[0][1].headers).get("authorization")).toBe("Bearer secret.jwt");
        expect(new Headers(calls[1][1].headers).get("last-event-id")).toBe("1");
        expect(states).toContain("connected");
    });

    test("401 is terminal and clears identity callback", async () => {
        let unauthorized = 0,
            calls = 0;
        const client = new NotificationRealtimeClient(
            "bad",
            1,
            {
                currentGeneration: () => 1,
                onInvalidate: () => {},
                onState: () => {},
                onUnauthorized: () => {
                    unauthorized += 1;
                },
            },
            (async () => {
                calls += 1;
                return response("", 401, { "content-type": "application/json" });
            }) as unknown as typeof fetch,
        );
        client.start();
        await new Promise((resolve) => setTimeout(resolve, 10));
        client.stop();
        expect([calls, unauthorized]).toEqual([1, 1]);
    });

    test("a late 401 from an old generation cannot clear the current session", async () => {
        let resolveResponse: ((value: Response) => void) | undefined;
        let generation = 1;
        let unauthorized = 0;
        const client = new NotificationRealtimeClient(
            "old.jwt",
            1,
            {
                currentGeneration: () => generation,
                onInvalidate: () => {},
                onState: () => {},
                onUnauthorized: () => {
                    unauthorized += 1;
                },
            },
            (() =>
                new Promise<Response>((resolve) => {
                    resolveResponse = resolve;
                })) as unknown as typeof fetch,
        );
        client.start();
        await new Promise((resolve) => setTimeout(resolve, 0));
        generation = 2;
        resolveResponse?.(response("", 401, { "content-type": "application/json" }));
        await new Promise((resolve) => setTimeout(resolve, 10));
        client.stop();
        expect(unauthorized).toBe(0);
    });

    test("retry jitter remains inside 1..30 seconds", () => {
        expect(retryDelay(0, () => 0)).toBe(1_000);
        expect(retryDelay(10, () => 1)).toBe(30_000);
    });

    test("handles deliberate stop, forbidden and capacity responses without fast retry", async () => {
        for (const [status, expectedState, expectedInvalidations] of [
            [204, "connecting", 1],
            [403, "forbidden", 0],
            [503, "reconnecting", 1],
        ] as const) {
            const states: StreamState[] = [];
            let calls = 0,
                invalidations = 0;
            const client = new NotificationRealtimeClient(
                "secret.jwt",
                1,
                {
                    currentGeneration: () => 1,
                    onInvalidate: () => {
                        invalidations += 1;
                    },
                    onState: (state) => states.push(state),
                    onUnauthorized: () => {},
                },
                (async () => {
                    calls += 1;
                    return response("", status, { "retry-after": "60" });
                }) as unknown as typeof fetch,
                () => 0,
            );
            client.start();
            await new Promise((resolve) => setTimeout(resolve, 10));
            client.stop();
            expect([calls, states.at(-1), invalidations]).toEqual([
                1,
                expectedState,
                expectedInvalidations,
            ]);
        }
    });

    test("rejects malformed advisory payloads instead of retrying them", async () => {
        const states: StreamState[] = [];
        let calls = 0;
        const client = new NotificationRealtimeClient(
            "secret.jwt",
            1,
            {
                currentGeneration: () => 1,
                onInvalidate: () => {},
                onState: (state) => states.push(state),
                onUnauthorized: () => {},
            },
            (async () => {
                calls += 1;
                return response("event: reconcile.required\ndata: not-json\n\n");
            }) as unknown as typeof fetch,
        );
        client.start();
        await new Promise((resolve) => setTimeout(resolve, 10));
        client.stop();
        expect(calls).toBe(1);
        expect(states.at(-1)).toBe("error");
    });
});
