import { useQuery } from "@tanstack/react-query";
import { Badge, Button, Popover, Space, Typography } from "antd";

import type { NotificationDeliveryStatus } from "@/api/notification-delivery";
import { t } from "@/lib/i18n";

import {
    formatDeliveryBytes,
    notificationDeliveryState,
    retryDelivery,
} from "./notification-delivery-model";

export type { NotificationDeliveryStatus } from "@/api/notification-delivery";

type DeliveryState = ReturnType<typeof notificationDeliveryState>;

export function NotificationDeliveryDetails({
    state,
    onRetry,
}: {
    state: DeliveryState;
    onRetry?: () => void;
}) {
    if (state.kind === "loading")
        return (
            <Typography.Text type="secondary">
                {t("正在加载通知投递状态", "Loading notification delivery status")}
            </Typography.Text>
        );
    if (state.kind === "permission" || state.kind === "error")
        return (
            <Space direction="vertical" size={4}>
                <Typography.Text>
                    {state.kind === "permission"
                        ? t(
                              "无权查看通知投递状态",
                              "You do not have permission to view notification delivery status",
                          )
                        : t("通知投递状态加载失败", "Failed to load notification delivery status")}
                </Typography.Text>
                {onRetry ? (
                    <Button size="small" onClick={onRetry}>
                        {t("重试", "Retry")}
                    </Button>
                ) : null}
            </Space>
        );
    const { status, firstGap, lastGap, lastSuccess } = state;
    return (
        <Space direction="vertical" size={2}>
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
            <Typography.Text>{t(`首个缺口 ${firstGap}`, `First gap ${firstGap}`)}</Typography.Text>
            <Typography.Text>{t(`最后缺口 ${lastGap}`, `Last gap ${lastGap}`)}</Typography.Text>
            <Typography.Text>
                {t(`最后成功 ${lastSuccess}`, `Last success ${lastSuccess}`)}
            </Typography.Text>
        </Space>
    );
}

const triggerCopy = (state: DeliveryState) => {
    if (state.kind === "loading")
        return { badge: "default" as const, label: t("通知投递…", "Delivery…") };
    if (state.kind === "permission")
        return {
            badge: "warning" as const,
            label: t("无权查看通知投递状态", "Delivery status unavailable"),
        };
    if (state.kind === "error")
        return {
            badge: "error" as const,
            label: t("通知投递状态加载失败", "Delivery status failed"),
        };
    if (state.kind === "healthy")
        return { badge: "success" as const, label: t("通知投递健康", "Delivery healthy") };
    return {
        badge: "warning" as const,
        label: t(
            `通知投递存在 ${state.gaps} 个不可恢复缺口`,
            `${state.gaps} irreversible delivery gaps`,
        ),
    };
};

export function NotificationDeliveryCardView({
    state,
    onRetry,
}: {
    state: DeliveryState;
    onRetry?: () => void;
}) {
    const { badge, label } = triggerCopy(state);
    return (
        <Popover
            trigger="click"
            placement="bottomRight"
            title={t("通知投递状态", "Notification delivery")}
            content={<NotificationDeliveryDetails state={state} onRetry={onRetry} />}
        >
            <Button data-testid={`notification-delivery-${state.kind}`} size="small" type="text">
                <Badge status={badge} text={label} />
            </Button>
        </Popover>
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
