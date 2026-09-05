import { expect, test } from "bun:test";

import {
    failedNetworkAction,
    hasNodesBackgroundRefreshFailure,
    retryFailedNetworkAction,
    shouldHydrateNodePolicy,
} from "./-save-state";

test("network failures retain the initial save snapshot while HTTP errors stay toast-only", () => {
    const values = { cpu: 82 };
    const action = { type: "save" as const, values };

    expect(failedNetworkAction(new TypeError("Failed to fetch"), action)).toBe(action);
    expect(failedNetworkAction(new Error("Unprocessable Entity"), action)).toBeUndefined();
    expect(failedNetworkAction(new Response(null, { status: 422 }), action)).toBeUndefined();
});

test("only managers can retry the original save or reset action", () => {
    const values = { cpu: 82 };
    const retried: unknown[] = [];
    let resets = 0;
    const effects = {
        save: (value: unknown) => retried.push(value),
        reset: () => {
            resets += 1;
        },
    };

    retryFailedNetworkAction(false, { type: "save", values }, effects);
    retryFailedNetworkAction(false, { type: "reset" }, effects);
    expect(retried).toEqual([]);
    expect(resets).toBe(0);

    retryFailedNetworkAction(true, { type: "save", values }, effects);
    retryFailedNetworkAction(true, { type: "reset" }, effects);
    expect(retried).toEqual([values]);
    expect(resets).toBe(1);
});

test("policy hydration respects a failed or edited draft and background Nodes errors keep cache", () => {
    expect(shouldHydrateNodePolicy(true, true)).toBe(true);
    expect(shouldHydrateNodePolicy(false, false)).toBe(true);
    expect(shouldHydrateNodePolicy(false, true)).toBe(false);

    expect(hasNodesBackgroundRefreshFailure([{ nodeId: "cached" }], new Error("offline"))).toBe(
        true,
    );
    expect(hasNodesBackgroundRefreshFailure([{ nodeId: "cached" }], undefined)).toBe(false);
    expect(hasNodesBackgroundRefreshFailure(undefined, new Error("offline"))).toBe(false);
});
