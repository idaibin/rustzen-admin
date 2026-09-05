import { ReloadOutlined } from "@ant-design/icons";
import { useIsMutating, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "antd";

import { appMessage, reportsAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { t } from "@/lib/i18n";

import {
    createRetryRunHandlers,
    isRetryableRunStatus,
    retryRunMutationKey,
} from "./retry-run-state";

export function RetryRunButton({
    run,
    onRetried,
    surface = "list",
}: {
    run: Reports.Run;
    onRetried: (run: Reports.Run) => void;
    surface?: "list" | "audit";
}) {
    const client = useQueryClient();
    const mutationKey = retryRunMutationKey(run.id);
    const isRetryPending = useIsMutating({ mutationKey }) > 0;
    const handlers = createRetryRunHandlers({
        invalidateRuns: () => client.invalidateQueries({ queryKey: ["reports", "runs"] }),
        selectRun: onRetried,
        showSuccess: () => appMessage.success(t("已打开重试执行", "Retry run opened")),
        showError: (message) => appMessage.error(message),
        fallbackError: t("重试失败，请稍后重试。", "Retry failed."),
    });
    const retry = useMutation({
        mutationKey,
        mutationFn: reportsAPI.retryRun,
        ...handlers,
    });

    if (!isRetryableRunStatus(run.status)) return null;
    return (
        <AuthWrap code="reports:run:manage">
            <Button
                type="text"
                icon={<ReloadOutlined />}
                data-testid={`run-retry-${surface}-${run.id}`}
                aria-label={t("重试执行", "Retry run")}
                disabled={isRetryPending}
                loading={isRetryPending}
                onClick={() => retry.mutate(run.id)}
            >
                {t("重试", "Retry")}
            </Button>
        </AuthWrap>
    );
}
