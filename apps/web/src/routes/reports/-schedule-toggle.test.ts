import { expect, test } from "bun:test";

import {
    isCurrentScheduleToggle,
    settleScheduleToggle,
    startScheduleToggle,
    type ScheduleToggleTracker,
} from "./-schedule-toggle";

const tracker = (id: string): ScheduleToggleTracker => ({
    id,
    nextRequestId: 0,
    pendingRequestId: null,
});

test("blocks duplicate toggles while the same schedule id is pending", () => {
    const first = startScheduleToggle(tracker("schedule-1"), true);
    expect(first.request).toMatchObject({ id: "schedule-1", requestId: 1, enabled: true });

    const duplicate = startScheduleToggle(first.tracker, false);
    expect(duplicate.request).toBeUndefined();
    expect(duplicate.tracker.pendingRequestId).toBe(1);
});

test("ignores an out-of-order response and keeps a failure visible for the current request", () => {
    const first = startScheduleToggle(tracker("schedule-1"), true);
    const newer: ScheduleToggleTracker = {
        ...first.tracker,
        nextRequestId: 2,
        pendingRequestId: 2,
    };
    expect(isCurrentScheduleToggle(newer, { id: "schedule-1", requestId: 1 })).toBe(false);
    expect(settleScheduleToggle(newer, { id: "schedule-1", requestId: 1 })).toEqual(newer);

    const failed = settleScheduleToggle(
        newer,
        { id: "schedule-1", requestId: 2 },
        "request failed",
    );
    expect(failed).toMatchObject({ pendingRequestId: null, errorMessage: "request failed" });
});
