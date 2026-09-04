import { describe, expect, test } from "bun:test";

import {
    createRetryRunHandlers,
    isRetryableRunStatus,
    retryRunMutationKey,
} from "./retry-run-state";

const run = (id: string, status: Reports.Run["status"]): Reports.Run =>
    ({ id, status }) as Reports.Run;

describe("report run retry behavior", () => {
    test("offers retry only for failed and cancelled terminal runs", () => {
        expect(isRetryableRunStatus("failed")).toBe(true);
        expect(isRetryableRunStatus("cancelled")).toBe(true);
        for (const status of ["queued", "running", "cancelling", "succeeded"] as const) {
            expect(isRetryableRunStatus(status)).toBe(false);
        }
    });

    test("shares pending state between list and detail for the same source only", () => {
        expect(retryRunMutationKey("source-1")).toEqual(retryRunMutationKey("source-1"));
        expect(retryRunMutationKey("source-1")).not.toEqual(retryRunMutationKey("source-2"));
    });

    test("refreshes the list before selecting and announcing the direct child", async () => {
        const events: string[] = [];
        const child = run("child-1", "queued");
        const handlers = createRetryRunHandlers({
            invalidateRuns: async () => {
                events.push("invalidated");
            },
            selectRun: (selected) => events.push(`selected:${selected.id}`),
            showSuccess: () => events.push("success"),
            showError: (message) => events.push(`error:${message}`),
            fallbackError: "fallback",
        });

        await handlers.onSuccess(child);

        expect(events).toEqual(["invalidated", "selected:child-1", "success"]);
    });

    test("reports request failures without replacing the selected source", () => {
        const selected = run("source-1", "failed");
        const selections: Reports.Run[] = [selected];
        const errors: string[] = [];
        const handlers = createRetryRunHandlers({
            invalidateRuns: async () => {},
            selectRun: (next) => selections.push(next),
            showSuccess: () => {},
            showError: (message) => errors.push(message),
            fallbackError: "Retry failed.",
        });

        handlers.onError(new Error("service unavailable"));
        handlers.onError(null);

        expect(selections).toEqual([selected]);
        expect(errors).toEqual(["service unavailable", "Retry failed."]);
    });
});
