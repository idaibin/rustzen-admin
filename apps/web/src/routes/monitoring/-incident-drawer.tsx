import { useQuery } from "@tanstack/react-query";
import { Button, Drawer, Space, Typography } from "antd";

import { monitorAPI } from "@/api";
import { ApiRequestError } from "@/api/request";
import { DataState } from "@/components/feedback/data-state";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const IncidentDrawer = ({
    incidentId,
    onClose,
}: {
    incident?: Monitor.IncidentSummary;
    incidentId?: string;
    onClose: () => void;
}) => {
    const generation = useAuthStore((state) => state.authGeneration);
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["monitor", "incident", generation, incidentId],
        queryFn: () => monitorAPI.incident(incidentId!),
        enabled: Boolean(incidentId),
        retry: false,
        staleTime: 0,
        refetchOnMount: "always",
    });
    const inaccessible = error instanceof ApiRequestError && [403, 404].includes(error.status ?? 0);
    const visibleData = !error && !isFetching ? data : undefined;
    return (
        <Drawer
            open={Boolean(incidentId)}
            onClose={onClose}
            title={
                inaccessible
                    ? t("事件详情", "Incident details")
                    : (visibleData?.title ?? t("事件详情", "Incident details"))
            }
            size="large"
            destroyOnHidden
        >
            {isPending ? (
                <DataState
                    kind="loading"
                    title={t("正在加载事件详情", "Loading incident details")}
                    compact
                />
            ) : null}
            {error ? (
                <DataState
                    kind="error"
                    title={t("事件不可访问", "Incident is inaccessible")}
                    action={
                        !inaccessible ? (
                            <Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>
                        ) : undefined
                    }
                    compact
                />
            ) : null}
            {visibleData ? (
                <Space orientation="vertical" className="w-full">
                    <Typography.Text>
                        {visibleData.node.hostname} · {visibleData.node.nodeId}
                    </Typography.Text>
                    <Typography.Text>
                        {t("阈值", "Threshold")}: {visibleData.thresholdPercent ?? "-"}% ·{" "}
                        {t("观测值", "Observed")}: {visibleData.observedPercent ?? "-"}%
                    </Typography.Text>
                    <Typography.Text>
                        {t("打开时间", "Opened")}: {formatDateTime(visibleData.openedAt)}
                    </Typography.Text>
                    <Typography.Text>
                        {t("解决原因", "Resolution reason")}: {visibleData.resolutionReason ?? "-"}
                    </Typography.Text>
                    <pre className="max-h-72 overflow-auto rounded bg-muted p-3 text-xs">
                        {JSON.stringify(visibleData.details, null, 2)}
                    </pre>
                </Space>
            ) : null}
        </Drawer>
    );
};
