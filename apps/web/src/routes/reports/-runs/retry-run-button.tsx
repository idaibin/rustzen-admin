import { ReloadOutlined } from "@ant-design/icons";
import { useIsMutating, useMutation, useQueryClient } from "@tanstack/react-query";
import { Button } from "antd";

import { appMessage, reportsAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { t } from "@/lib/i18n";

const isRetryable = (status: Reports.Run["status"]) =>
    status === "failed" || status === "cancelled";
const retryMutationKey = (sourceRunId: string) => ["reports", "retry-run", sourceRunId] as const;

export function RetryRunButton({
    run,
    onRetried,
}: {
    run: Reports.Run;
    onRetried: (run: Reports.Run) => void;
}) {
    const client = useQueryClient();
    const mutationKey = retryMutationKey(run.id);
    const isRetryPending = useIsMutating({ mutationKey }) > 0;
    const retry = useMutation({
        mutationKey,
        mutationFn: reportsAPI.retryRun,
        onSuccess: async (retriedRun) => {
            await client.invalidateQueries({ queryKey: ["reports", "runs"] });
            onRetried(retriedRun);
            appMessage.success(t("已打开重试执行", "Retry run opened"));
        },
        onError: (error) => {
            appMessage.error(
                error instanceof Error ? error.message : t("重试失败，请稍后重试。", "Retry failed."),
            );
        },
    });

    if (!isRetryable(run.status)) return null;
    return (
        <AuthWrap code="reports:run:manage">
            <Button
                type="text"
                icon={<ReloadOutlined />}
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
