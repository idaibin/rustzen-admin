import { describe, expect, test } from "bun:test";

import {
    beginScheduleDialogCycle,
    createScheduleSaveHandlers,
    endScheduleDialogCycle,
    initialScheduleDialogCycle,
    isCurrentScheduleDialogCycle,
    markScheduleDraftInitialized,
    shouldInitializeScheduleDraft,
} from "./schedule-save-state";

describe("schedule save behavior", () => {
    test("retains the open draft and exposes a request failure for retry", () => {
        const events: string[] = [];
        const handlers = createScheduleSaveHandlers({
            refresh: async () => events.push("refreshed"),
            isCurrent: () => true,
            showSuccess: () => events.push("success"),
            close: () => events.push("closed"),
            showError: (message) => events.push(`error:${message}`),
            fallbackError: "Save failed.",
        });

        handlers.onError(new Error("service unavailable"), { cycle: 1 });
        handlers.onError(null, { cycle: 1 });

        expect(events).toEqual(["error:service unavailable", "error:Save failed."]);
    });

    test("refreshes and closes only after a successful save", async () => {
        const events: string[] = [];
        const handlers = createScheduleSaveHandlers({
            refresh: async () => events.push("refreshed"),
            isCurrent: () => true,
            showSuccess: () => events.push("success"),
            close: () => events.push("closed"),
            showError: (message) => events.push(`error:${message}`),
            fallbackError: "Save failed.",
        });

        await handlers.onSuccess(undefined, { cycle: 1 });

        expect(events).toEqual(["refreshed", "success", "closed"]);
    });

    test("ignores late callbacks from a closed dialog cycle", async () => {
        const events: string[] = [];
        const handlers = createScheduleSaveHandlers({
            refresh: async () => events.push("refreshed"),
            isCurrent: (cycle) => cycle === 2,
            showSuccess: () => events.push("success"),
            close: () => events.push("closed"),
            showError: (message) => events.push(`error:${message}`),
            fallbackError: "Save failed.",
        });

        handlers.onError(new Error("late error"), { cycle: 1 });
        await handlers.onSuccess(undefined, { cycle: 1 });

        expect(events).toEqual(["refreshed"]);
    });

    test("initializes a draft once per open cycle despite prop refreshes", () => {
        const opened = beginScheduleDialogCycle(initialScheduleDialogCycle);
        expect(shouldInitializeScheduleDraft(opened)).toBe(true);

        const initialized = markScheduleDraftInitialized(opened);
        expect(shouldInitializeScheduleDraft(initialized)).toBe(false);
        expect(isCurrentScheduleDialogCycle(initialized, opened.id)).toBe(true);

        const closed = endScheduleDialogCycle(initialized);
        const reopened = beginScheduleDialogCycle(closed);
        expect(shouldInitializeScheduleDraft(reopened)).toBe(true);
        expect(isCurrentScheduleDialogCycle(reopened, opened.id)).toBe(false);
    });
});
