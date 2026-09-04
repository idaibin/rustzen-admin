export type RetryableRunStatus = "failed" | "cancelled";

export const isRetryableRunStatus = (status: Reports.Run["status"]): status is RetryableRunStatus =>
    status === "failed" || status === "cancelled";

export const retryRunMutationKey = (sourceRunId: string) =>
    ["reports", "retry-run", sourceRunId] as const;

interface RetryRunEffects {
    invalidateRuns: () => Promise<void>;
    selectRun: (run: Reports.Run) => void;
    showSuccess: () => void;
    showError: (message: string) => void;
    fallbackError: string;
}

export const createRetryRunHandlers = (effects: RetryRunEffects) => ({
    onSuccess: async (retriedRun: Reports.Run) => {
        await effects.invalidateRuns();
        effects.selectRun(retriedRun);
        effects.showSuccess();
    },
    onError: (error: unknown) => {
        effects.showError(error instanceof Error ? error.message : effects.fallbackError);
    },
});
