import { expect, test } from "bun:test";

import { formatDateTime } from "@/lib/format-date-time";

import {
    formatDeliveryBytes,
    notificationDeliveryState,
    retryDelivery,
} from "./notification-delivery-model";
const base = {
    pendingCount: 1,
    pendingBytes: 2,
    quarantineCount: 3,
    quarantineBytes: 4,
    omittedCount: 0,
    expiredCount: 0,
    unconfirmedCount: 0,
    quarantinedCount: 0,
    quarantineEvictedCount: 0,
    firstGapAt: null,
    lastGapAt: null,
    lastSuccessAt: null,
};
test("delivery states and retry", async () => {
    expect(notificationDeliveryState({ isPending: true }).kind).toBe("loading");
    expect(notificationDeliveryState({ data: base, isPending: false }).kind).toBe("healthy");
    const gap = notificationDeliveryState({
        data: {
            ...base,
            omittedCount: 1,
            expiredCount: 1,
            unconfirmedCount: 1,
            quarantinedCount: 1,
            quarantineEvictedCount: 1,
        },
        isPending: false,
    });
    expect(gap.kind).toBe("gap");
    if (gap.kind !== "gap") throw new Error("gap");
    expect(gap.gaps).toBe(5);
    expect(notificationDeliveryState({ error: { status: 403 }, isPending: false }).kind).toBe(
        "permission",
    );
    expect(notificationDeliveryState({ error: new Error(), isPending: false }).kind).toBe("error");
    expect(formatDeliveryBytes(2)).toBe("2 B");
    expect(gap.firstGap).toBe("-");
    expect(gap.lastGap).toBe("-");
    expect(gap.lastSuccess).toBe("-");
    const withDates = notificationDeliveryState({
        data: {
            ...base,
            firstGapAt: "2026-09-10T01:02:03.000Z",
            lastGapAt: "2026-09-10T02:03:04.000Z",
            lastSuccessAt: "2026-09-10T03:04:05.000Z",
        },
        isPending: false,
    });
    if (withDates.kind !== "healthy") throw new Error("healthy");
    expect(withDates.firstGap).toBe(formatDateTime(withDates.status.firstGapAt));
    expect(withDates.lastGap).toBe(formatDateTime(withDates.status.lastGapAt));
    expect(withDates.lastSuccess).toBe(formatDateTime(withDates.status.lastSuccessAt));
    let calls = 0;
    let resolveRefetch!: () => void;
    let completed = false;
    const retry = retryDelivery(
        () =>
            new Promise<void>((resolve) => {
                calls++;
                resolveRefetch = resolve;
            }),
    ).then(() => {
        completed = true;
    });
    await Promise.resolve();
    expect(calls).toBe(1);
    expect(completed).toBeFalse();
    resolveRefetch();
    await retry;
    expect(completed).toBeTrue();
    expect(calls).toBe(1);
});
