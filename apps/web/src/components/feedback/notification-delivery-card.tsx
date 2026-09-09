import { useQuery } from "@tanstack/react-query";
import { Alert, Button, Space, Typography } from "antd";

import type { NotificationDeliveryStatus } from "@/api/notification-delivery";
import { t } from "@/lib/i18n";

import {
    formatDeliveryBytes,
    notificationDeliveryState,
    retryDelivery,
} from "./notification-delivery-model";

export type { NotificationDeliveryStatus } from "@/api/notification-delivery";

export function NotificationDeliveryCardView({
    state,
    onRetry,
}: {
    state: ReturnType<typeof notificationDeliveryState>;
    onRetry?: () => void;
}) {
    if (state.kind === "loading")
        return (
            <Alert
                data-testid="notification-delivery-loading"
                type="info"
                title={t("正在加载通知投递状态", "Loading notification delivery status")}
            />
        );
    if (state.kind === "permission" || state.kind === "error")
        return (
            <Alert
                data-testid={`notification-delivery-${state.kind}`}
                type={state.kind === "permission" ? "warning" : "error"}
                title={
                    state.kind === "permission"
                        ? t(
                              "无权查看通知投递状态",
                              "You do not have permission to view notification delivery status",
                          )
                        : t("通知投递状态加载失败", "Failed to load notification delivery status")
                }
                action={<Button onClick={onRetry}>{t("重试", "Retry")}</Button>}
            />
        );
    const { status, gaps, firstGap, lastGap, lastSuccess } = state;
    return (
        <Alert
            data-testid={`notification-delivery-${state.kind}`}
            type={state.kind === "healthy" ? "success" : "warning"}
            title={
                state.kind === "healthy"
                    ? t("通知投递健康", "Notification delivery healthy")
                    : t(
                          `通知投递存在 ${gaps} 个不可恢复缺口`,
                          `${gaps} irreversible notification delivery gaps`,
                      )
            }
            description={
                <Space wrap>
                    <Typography.Text>
                        {t(
                            `待投递 ${status.pendingCount}（${formatDeliveryBytes(status.pendingBytes)}）`,
                            `Pending ${status.pendingCount} (${formatDeliveryBytes(status.pendingBytes)})`,
                        )}
                    </Typography.Text>
                    <Typography.Text>
                        {t(
                            `隔离 ${status.quarantineCount}（${formatDeliveryBytes(status.quarantineBytes)}）`,
                            `Quarantine ${status.quarantineCount} (${formatDeliveryBytes(status.quarantineBytes)})`,
                        )}
                    </Typography.Text>
                    <Typography.Text>
                        {t(`首个缺口 ${firstGap}`, `First gap ${firstGap}`)}
                    </Typography.Text>
                    <Typography.Text>
                        {t(`最后缺口 ${lastGap}`, `Last gap ${lastGap}`)}
                    </Typography.Text>
                    <Typography.Text>
                        {t(`最后成功 ${lastSuccess}`, `Last success ${lastSuccess}`)}
                    </Typography.Text>
                </Space>
            }
        />
    );
}

export function NotificationDeliveryCard({
    queryKey,
    queryFn,
}: {
    queryKey: readonly unknown[];
    queryFn: () => Promise<NotificationDeliveryStatus>;
}) {
    const q = useQuery({ queryKey, queryFn, retry: false });
    const state = notificationDeliveryState({
        data: q.data,
        error: q.error,
        isPending: q.isPending,
    });
    return (
        <div data-testid="notification-delivery-card">
            <NotificationDeliveryCardView
                state={state}
                onRetry={() => void retryDelivery(q.refetch)}
            />
        </div>
    );
}
