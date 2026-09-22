import { describe, expect, test } from "bun:test";

import {
    createRetryRunHandlers,
    isRetryableRunStatus,
    retryRunMutationKey,
} from "./retry-run-state";

const run = (id: string, status: Reports.Run["status"]): Reports.Run =>
    ({ id, status }) as Reports.Run;

describe("report run retry behavior", () => {
    test("audit identity follows the selected run without waiting for its refresh", async () => {
        const details = await Bun.file(new URL("./run-details.tsx", import.meta.url)).text();
        expect(details).toContain('data-testid="run-audit" data-run-id={run?.id}');
        expect(details).not.toContain('data-testid="run-audit" data-run-id={currentRun?.id}');
    });

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

    test("selects the direct child before refreshing the list and announcing success", async () => {
        const events: string[] = [];
        const child = run("child-1", "queued");
        let finishRefresh!: () => void;
        const handlers = createRetryRunHandlers({
            invalidateRuns: () => {
                events.push("refresh-started");
                return new Promise<void>((resolve) => {
                    finishRefresh = () => {
                        events.push("invalidated");
                        resolve();
                    };
                });
            },
            selectRun: (selected) => events.push(`selected:${selected.id}`),
            showSuccess: () => events.push("success"),
            showError: (message) => events.push(`error:${message}`),
            fallbackError: "fallback",
        });

        const completion = handlers.onSuccess(child);
        expect(events).toEqual(["selected:child-1", "success", "refresh-started"]);
        finishRefresh();
        await completion;

        expect(events).toEqual(["selected:child-1", "success", "refresh-started", "invalidated"]);
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
