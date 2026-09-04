import { describe, expect, test } from "bun:test";

const route = await Bun.file(new URL("../src/routes/reports/runs.tsx", import.meta.url)).text();
const dialog = await Bun.file(
    new URL("../src/routes/reports/-runs/run-dialog.tsx", import.meta.url),
).text();
const details = await Bun.file(
    new URL("../src/routes/reports/-runs/run-details.tsx", import.meta.url),
).text();
const frame = await Bun.file(
    new URL("../src/routes/reports/-runs/live-frame.tsx", import.meta.url),
).text();
const status = await Bun.file(
    new URL("../src/routes/reports/-runs/status.ts", import.meta.url),
).text();
const retry = await Bun.file(
    new URL("../src/routes/reports/-runs/retry-run-button.tsx", import.meta.url),
).text();

describe("reports runs route composition", () => {
    test("keeps the route as an orchestrator and delegates each bounded concern", () => {
        expect(route.split("\n").length).toBeLessThan(300);
        expect(route).toContain('import { RunDialog } from "./-runs/run-dialog"');
        expect(route).toContain('import { RunDetails } from "./-runs/run-details"');
        expect(route).toContain('import { RetryRunButton } from "./-runs/retry-run-button"');
        expect(route).toContain("<RunDialog flows={flows} />");
        expect(route).toContain("<RunDetails");
        expect(route).toContain("onRetried={setSelected}");
        expect(route).not.toContain("function LiveFrame(");
        expect(route).not.toContain("function RunDialog(");
        expect(route).not.toContain("function RunDetails(");
    });

    test("preserves the run lifecycle, live frame, and localized status seams", () => {
        expect(dialog).toContain("reportsAPI.createRun");
        expect(dialog).toContain('queryKey: ["reports", "runs"]');
        expect(details).toContain("reportsAPI.runSteps(run!.id)");
        expect(details).toContain("reportsAPI.runArtifacts(run!.id)");
        expect(details).toContain("<RetryRunButton run={currentRun} onRetried={onRetried} />");
        expect(retry).toContain("reportsAPI.retryRun");
        expect(retry).toContain('code="reports:run:manage"');
        expect(retry).toContain('status === "failed" || status === "cancelled"');
        expect(retry).toContain("useIsMutating");
        expect(retry).toContain('const retryMutationKey = (sourceRunId: string) => ["reports", "retry-run", sourceRunId]');
        expect(retry).toContain("mutationKey,");
        expect(retry).toContain("disabled={isRetryPending}");
        expect(frame).toContain("reportsAPI.liveFrame(run!.id, signal)");
        expect(frame).toContain("URL.revokeObjectURL(url)");
        expect(status).toContain('status === "cancelling"');
        expect(status).toContain("getStepStatusMeta");
    });
});
