import {
    CloudServerOutlined,
    DisconnectOutlined,
    ExclamationCircleOutlined,
    SignalFilled,
} from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card, Progress, Tag, Typography } from "antd";

import { monitorAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageHeader } from "@/components/page/page-header";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { hasMonitorBackgroundRefreshFailure, isMonitorPermissionDenied } from "./-save-state";

export const Route = createFileRoute("/monitoring/overview")({ component: MonitoringOverviewPage });

function MonitoringOverviewPage() {
    const { data, dataUpdatedAt, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "overview"],
        queryFn: monitorAPI.overview,
        refetchInterval: 30_000,
        retry: false,
    });
    const header = (
        <PageHeader
            title={t("监控概览", "Monitoring overview")}
            description={t(
                "查看节点在线状态、活动告警与最近一次资源上报。",
                "View node availability, active alerts, and the most recent resource report.",
            )}
            actions={
                <span className="text-xs text-muted-foreground">
                    {t("每 30 秒自动刷新", "Refreshes every 30 seconds")}
                </span>
            }
        />
    );
    const permissionDenied = isMonitorPermissionDenied(error);
    if (permissionDenied) {
        return (
            <div className="flex h-full min-h-0 flex-col gap-5 overflow-y-auto">
                {header}
                <DataState
                    kind="permission"
                    title={t(
                        "没有查看监控概览的权限",
                        "You do not have permission to view monitoring",
                    )}
                    description={t(
                        "无法读取监控概览，请检查权限后重试。",
                        "Unable to read monitoring data. Check your permission and try again.",
                    )}
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </div>
        );
    }
    if (!data) {
        return (
            <div className="flex h-full min-h-0 flex-col gap-5 overflow-y-auto">
                {header}
                <DataState
                    kind={isPending ? "loading" : "error"}
                    title={
                        isPending
                            ? t("正在加载监控概览", "Loading overview")
                            : t("监控概览加载失败", "Failed to load monitoring overview")
                    }
                    action={
                        !isPending ? (
                            <Button type="primary" onClick={() => void refetch()}>
                                {t("重新加载", "Reload")}
                            </Button>
                        ) : undefined
                    }
                />
            </div>
        );
    }
    const cards = [
        {
            label: t("已注册节点", "Registered nodes"),
            value: data.registeredNodes,
            icon: <CloudServerOutlined />,
            tone: "blue" as const,
        },
        {
            label: t("在线节点", "Online nodes"),
            value: data.onlineNodes,
            icon: <SignalFilled />,
            tone: "green" as const,
        },
        {
            label: t("离线节点", "Offline nodes"),
            value: data.offlineNodes,
            icon: <DisconnectOutlined />,
            tone: "amber" as const,
        },
        {
            label: t("活动告警", "Active incidents"),
            value: data.activeIncidents,
            icon: <ExclamationCircleOutlined />,
            tone: "red" as const,
        },
    ];
    return (
        <div className="flex h-full min-h-0 flex-col gap-5 overflow-y-auto">
            {header}
            <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                {cards.map((item) => (
                    <MetricCard key={item.label} {...item} />
                ))}
            </div>
            {hasMonitorBackgroundRefreshFailure(data, error) ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            <LatestResource value={data.latestResource} />
        </div>
    );
}

function LatestResource({ value }: { value: Monitor.Overview["latestResource"] }) {
    if (!value)
        return (
            <DataState
                kind="empty"
                title={t("暂无资源上报", "No resource reports")}
                description={t(
                    "节点 Agent 首次上报后，最新资源数据会显示在这里。",
                    "The latest resource data appears after an agent's first report.",
                )}
            />
        );
    return (
        <Card
            className="page-panel"
            title={t("最近资源上报", "Latest resource report")}
            extra={
                <Typography.Text type="secondary">
                    {formatDateTime(value.collectedAt)}
                </Typography.Text>
            }
        >
            <div className="grid gap-4 md:grid-cols-3">
                <Resource label="CPU" percent={value.cpuPercent} />
                <Resource label={t("内存", "Memory")} percent={value.memoryPercent} />
                <div>
                    <Typography.Text type="secondary">
                        {t("磁盘挂载点", "Disk mounts")}
                    </Typography.Text>
                    <div className="mt-2 space-y-2">
                        {value.disks.map((disk) => (
                            <div key={disk.mountPoint}>
                                <div className="flex justify-between text-sm">
                                    <span>{disk.mountPoint}</span>
                                    <span>{disk.usagePercent.toFixed(1)}%</span>
                                </div>
                                <Progress percent={disk.usagePercent} size="small" />
                            </div>
                        ))}
                    </div>
                </div>
            </div>
            <div className="mt-3">
                <Tag>{value.nodeId}</Tag>
            </div>
        </Card>
    );
}

function Resource({ label, percent }: { label: string; percent: number }) {
    return (
        <div>
            <Typography.Text type="secondary">{label}</Typography.Text>
            <div className="mt-2 text-2xl font-semibold">{percent.toFixed(1)}%</div>
            <Progress percent={percent} size="small" />
        </div>
    );
}
