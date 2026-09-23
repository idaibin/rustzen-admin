import { describe, expect, test } from "bun:test";

import { QueryClient } from "@tanstack/react-query";

import { bindAuthenticatedQueryCache } from "./auth-query-cache";

describe("authenticated query cache", () => {
    test("clears every cached user query exactly when auth generation changes", () => {
        const client = new QueryClient();
        let state = { authGeneration: 7 };
        let listener: ((value: typeof state) => void) | undefined;
        const unsubscribe = bindAuthenticatedQueryCache(client, {
            getState: () => state,
            subscribe: (next) => {
                listener = next;
                return () => {
                    listener = undefined;
                };
            },
        });
        client.setQueryData(["monitor", "incident", "secret"], { title: "old user" });
        client.setQueryData(["reports", "run", "secret"], { id: "secret" });
        client.setQueryData(["notifications", 7, "inbox"], { count: 9 });

        listener?.(state);
        expect(client.getQueryCache().getAll()).toHaveLength(3);
        state = { authGeneration: 8 };
        listener?.(state);
        expect(client.getQueryCache().getAll()).toHaveLength(0);
        unsubscribe();
    });

    test("aborts an authenticated request still running during an identity change", async () => {
        const client = new QueryClient();
        let listener: ((value: { authGeneration: number }) => void) | undefined;
        let aborted = false;
        let started: (() => void) | undefined;
        const ready = new Promise<void>((resolve) => {
            started = resolve;
        });
        bindAuthenticatedQueryCache(client, {
            getState: () => ({ authGeneration: 1 }),
            subscribe: (next) => {
                listener = next;
                return () => {};
            },
        });
        const pending = client
            .fetchQuery({
                queryKey: ["reports", "run", "old-user"],
                queryFn: ({ signal }) =>
                    new Promise<void>((_resolve, reject) => {
                        signal.addEventListener("abort", () => {
                            aborted = true;
                            reject(new Error("aborted"));
                        });
                        started?.();
                    }),
            })
            .catch(() => undefined);
        await ready;
        listener?.({ authGeneration: 2 });
        await pending;
        expect(aborted).toBe(true);
        expect(client.getQueryCache().getAll()).toHaveLength(0);
    });
});
