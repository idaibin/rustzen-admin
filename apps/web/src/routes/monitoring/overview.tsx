import {
    AlertFilled,
    CloudServerOutlined,
    CloudSyncOutlined,
    ExclamationCircleOutlined,
    LineChartOutlined,
    SignalFilled,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card, Tag } from "antd";

import { monitorAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageHeader } from "@/components/page/page-header";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/monitoring/overview")({ component: MonitoringOverviewPage });

function MonitoringOverviewPage() {
    const canViewIncidents = useAuthStore((state) =>
        state.checkPermissions("monitor:incident:view"),
    );
    const { data, dataUpdatedAt, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "overview"],
        queryFn: monitorAPI.overview,
        refetchInterval: 30_000,
    });

    if (isPending && !data) {
        return (
            <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto pr-1">
                <PageHeader
                    title={t("监控概览", "Monitoring overview")}
                    description={t(
                        "查看当前节点可用性和最新基础设施心跳。",
                        "View current node availability and the latest infrastructure heartbeats.",
                    )}
                />
                <DataState kind="loading" title={t("正在加载监控概览", "Loading overview")} />
            </div>
        );
    }

    if (!data) {
        return (
            <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto pr-1">
                <PageHeader
                    title={t("监控概览", "Monitoring overview")}
                    description={t(
                        "查看当前节点可用性和最新基础设施心跳。",
                        "View current node availability and the latest infrastructure heartbeats.",
                    )}
                />
                <DataState
                    kind="error"
                    title={
                        error
                            ? t("监控概览加载失败", "Failed to load monitoring overview")
                            : t("监控概览暂不可用", "Monitoring overview is unavailable")
                    }
                    description={t(
                        "无法读取节点和服务状态，请检查 Monitor 服务后重试。",
                        "Unable to read node and service status. Check the Monitor service and try again.",
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
            icon: <CloudSyncOutlined />,
            tone: "amber" as const,
        },
        {
            label: t("异常检查", "Unhealthy checks"),
            value: data.unhealthyChecks,
            icon: <AlertFilled />,
            tone: "red" as const,
        },
        ...(canViewIncidents
            ? [
                  {
                      label: t("活动事件", "Active incidents"),
                      value: data.activeIncidents,
                      icon: <ExclamationCircleOutlined />,
                      tone: "violet" as const,
                  },
              ]
            : []),
    ];

    return (
        <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto pr-1">
            <PageHeader
                title={t("监控概览", "Monitoring overview")}
                description={t(
                    "查看当前节点可用性和最新基础设施心跳。",
                    "View current node availability and the latest infrastructure heartbeats.",
                )}
            />
            <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-5">
                {cards.map((item) => (
                    <MetricCard
                        key={item.label}
                        label={item.label}
                        value={item.value}
                        icon={item.icon}
                        tone={item.tone}
                    />
                ))}
            </div>
            <div className="mt-4 flex items-center gap-2 text-sm text-muted-foreground">
                <LineChartOutlined />
                <span>
                    {t("监控数据每 30 秒自动刷新。", "Monitoring data refreshes every 30 seconds.")}
                </span>
            </div>
            {error ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            {canViewIncidents ? <IncidentOverviewPanel /> : null}
            {data.registeredNodes === 0 ? (
                <DataState
                    kind="empty"
                    title={t("暂无监控节点", "No monitored nodes")}
                    description={t(
                        "启动节点上的 rz-monitor agent，首次心跳通过后会自动出现在这里。",
                        "Start the rz-monitor agent on a node. It will appear here after its first heartbeat.",
                    )}
                />
            ) : null}
        </div>
    );
}

function IncidentOverviewPanel() {
    const canViewIncidents = useAuthStore((state) =>
        state.checkPermissions("monitor:incident:view"),
    );
    const { data, dataUpdatedAt, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["monitor", "incidents", "overview"],
        queryFn: () => monitorAPI.incidents({ current: 1, pageSize: 5, status: "open" }),
        enabled: canViewIncidents,
        refetchInterval: 30_000,
    });
    if (!canViewIncidents) return null;
    const columns: ProColumns<Monitor.IncidentSummary>[] = [
        {
            title: t("事件", "Incident"),
            key: "title",
            ellipsis: true,
            render: (_value: unknown, row: Monitor.IncidentSummary) => (
                <div>
                    <div className="font-medium">{row.title}</div>
                    <div className="text-xs text-muted-foreground">
                        {row.sourceType} · {row.sourceId}
                    </div>
                </div>
            ),
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 120,
            render: (_value: unknown, _row: Monitor.IncidentSummary) => (
                <Tag color="error">{t("活动", "Open")}</Tag>
            ),
        },
        {
            title: t("最近观察", "Last observed"),
            key: "lastObservedAt",
            width: 180,
            render: (_value: unknown, row: Monitor.IncidentSummary) =>
                formatDateTime(row.lastObservedAt),
        },
    ];

    return (
        <Card
            title={t("活动事件", "Active incidents")}
            extra={
                <AuthWrap code="monitor:incident:view">
                    <Button type="link" href="/monitoring/incidents">
                        {t("查看全部", "View all")}
                    </Button>
                </AuthWrap>
            }
        >
            {error && data ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            {!data && isPending ? (
                <DataState
                    kind="loading"
                    title={t("正在加载活动事件", "Loading active incidents")}
                    compact
                />
            ) : !data && error ? (
                <DataState
                    kind="error"
                    title={t("活动事件暂不可用", "Active incidents unavailable")}
                    description={t(
                        "无法读取事件列表，请检查 Monitor 服务。",
                        "Unable to read incidents. Check the Monitor service.",
                    )}
                    action={<Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>}
                    compact
                />
            ) : data && data.data.length === 0 ? (
                <DataState
                    kind="empty"
                    title={t("暂无活动事件", "No active incidents")}
                    description={t(
                        "Monitor 当前没有活动事件。",
                        "Monitor has no active incidents right now.",
                    )}
                    compact
                />
            ) : (
                <DataTableShell>
                    <ProTable<Monitor.IncidentSummary>
                        rowKey="id"
                        columns={columns}
                        dataSource={data?.data ?? []}
                        loading={isFetching}
                        search={false}
                        options={false}
                        pagination={false}
                        toolBarRender={false}
                        tableAlertOptionRender={false}
                        rowSelection={false}
                    />
                </DataTableShell>
            )}
        </Card>
    );
}
