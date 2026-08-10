import { useQuery } from "@tanstack/react-query";
import { Button, Card, Tag } from "antd";

import { insightsQueryOptions } from "@/api/insights/query-options";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export function CollectionPolicyStatus() {
    const canManage = useAuthStore((state) => state.checkPermissions("insights:manage"));
    const { data, dataUpdatedAt, error, isPending, refetch } = useQuery({
        ...insightsQueryOptions.collectionPolicy(),
        enabled: canManage,
    });

    if (!canManage) {
        return (
            <DataState
                kind="permission"
                compact
                title={t("无采集策略查看权限", "Collection policy permission required")}
                description={t(
                    "当前角色不能查看 Insights 采集状态。",
                    "The current role cannot view Insights collection status.",
                )}
            />
        );
    }

    if (!data && isPending) {
        return (
            <DataState
                kind="loading"
                compact
                title={t("正在加载采集策略", "Loading collection policy")}
            />
        );
    }

    if (!data) {
        return (
            <DataState
                kind="error"
                compact
                title={
                    error
                        ? t("采集策略加载失败", "Failed to load collection policy")
                        : t("采集策略暂不可用", "Collection policy is unavailable")
                }
                description={t(
                    "无法读取 Insights 采集状态，请检查服务后重试。",
                    "Unable to read Insights collection status. Check the service and try again.",
                )}
                action={<Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>}
            />
        );
    }

    return (
        <Card
            size="small"
            title={t("采集策略状态", "Collection policy status")}
            className="border border-border bg-card"
        >
            {error ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            <div className="flex flex-wrap items-center gap-3" role="status" aria-live="polite">
                <span className="text-sm font-medium text-foreground">
                    {t("公开采集", "Public collection")}
                </span>
                <Tag color={data.collectionEnabled ? "success" : "warning"}>
                    {data.collectionEnabled ? t("已启用", "Enabled") : t("已停用", "Disabled")}
                </Tag>
                <span className="text-sm text-muted-foreground">
                    {t("项目标识", "Project identifier")}
                </span>
                <Tag color={data.projectConfigured ? "success" : "default"}>
                    {data.projectConfigured
                        ? t("已配置", "Configured")
                        : t("未配置", "Not configured")}
                </Tag>
                <span className="text-sm text-muted-foreground">
                    {t("允许来源", "Allowed origins")}
                </span>
                <Tag>{data.allowedOrigins.length}</Tag>
            </div>
        </Card>
    );
}
