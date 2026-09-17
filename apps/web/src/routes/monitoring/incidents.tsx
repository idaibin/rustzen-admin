import { CheckCircleOutlined, ExclamationCircleOutlined, EyeOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Pagination, Select, Space, Tag, Typography } from "antd";
import { useMemo, useState } from "react";

import { monitorAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { NotificationDeliveryCard } from "@/components/feedback/notification-delivery-card";
import { PageCard } from "@/components/page/page-card";
import { actionColumnWidth } from "@/components/table/action-column";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { IncidentDrawer } from "./-incident-drawer";
import { hasMonitorBackgroundRefreshFailure, isMonitorPermissionDenied } from "./-save-state";

export const Route = createFileRoute("/monitoring/incidents")({
    component: MonitoringIncidentsPage,
});
const PAGE_SIZE = 20;

function MonitoringIncidentsPage() {
    const [status, setStatus] = useState<"all" | Monitor.IncidentStatus>("all");
    const [kind, setKind] = useState<"all" | Monitor.IncidentKind>("all");
    const [current, setCurrent] = useState(1);
    const [selected, setSelected] = useState<Monitor.IncidentSummary>();
    const [linkedId, setLinkedId] = useState(() =>
        typeof window === "undefined"
            ? undefined
            : (new URLSearchParams(window.location.search).get("incidentId") ?? undefined),
    );
    const query = useMemo<Monitor.IncidentQuery>(
        () => ({
            current,
            pageSize: PAGE_SIZE,
            status: status === "all" ? undefined : status,
            kind: kind === "all" ? undefined : kind,
        }),
        [current, kind, status],
    );
    const { data, dataUpdatedAt, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["monitor", "incidents", query],
        queryFn: () => monitorAPI.incidents(query),
        refetchInterval: 30_000,
        retry: false,
    });
    const searchControls = (
        <Space wrap>
            <Select
                aria-label={t("告警状态", "Incident status")}
                value={status}
                onChange={(value) => {
                    setStatus(value);
                    setCurrent(1);
                }}
                options={[
                    { value: "all", label: t("全部状态", "All statuses") },
                    { value: "active", label: t("活动", "Active") },
                    { value: "resolved", label: t("已解决", "Resolved") },
                ]}
            />
            <Select
                aria-label={t("告警类型", "Incident kind")}
                value={kind}
                onChange={(value) => {
                    setKind(value);
                    setCurrent(1);
                }}
                options={[
                    { value: "all", label: t("全部类型", "All kinds") },
                    { value: "cpuHigh", label: "CPU" },
                    { value: "memoryHigh", label: t("内存", "Memory") },
                    { value: "diskHigh", label: t("磁盘", "Disk") },
                    { value: "nodeOffline", label: t("节点离线", "Node offline") },
                ]}
            />
        </Space>
    );

    const deliveryCard = (
        <NotificationDeliveryCard
            queryKey={["monitor", "notification-delivery"]}
            queryFn={monitorAPI.notificationDelivery}
        />
    );
    const permissionDenied = isMonitorPermissionDenied(error);
    if (permissionDenied)
        return (
            <PageCard
                toolbar={searchControls}
                actions={deliveryCard}
                title={t("告警事件", "Alert incidents")}
                description={t(
                    "查看活动和最近解决的资源与离线告警。",
                    "View active and recently resolved resource and offline alerts.",
                )}
            >
                <DataState
                    kind="permission"
                    title={t(
                        "没有查看告警事件的权限",
                        "You do not have permission to view alert incidents",
                    )}
                    description={t(
                        "无法读取告警事件，请检查权限后重试。",
                        "Unable to read alert incidents. Check your permission and try again.",
                    )}
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </PageCard>
        );
    if (!data)
        return (
            <PageCard
                toolbar={searchControls}
                actions={deliveryCard}
                title={t("告警事件", "Alert incidents")}
                description={t(
                    "查看活动和最近解决的资源与离线告警。",
                    "View active and recently resolved resource and offline alerts.",
                )}
            >
                <DataState
                    kind={isPending ? "loading" : "error"}
                    title={
                        isPending
                            ? t("正在加载告警事件", "Loading alert incidents")
                            : t("告警事件加载失败", "Failed to load alert incidents")
                    }
                    action={
                        !isPending ? (
                            <Button type="primary" onClick={() => void refetch()}>
                                {t("重新加载", "Reload")}
                            </Button>
                        ) : undefined
                    }
                />
            </PageCard>
        );
    const columns: ProColumns<Monitor.IncidentSummary>[] = [
        {
            title: t("事件", "Incident"),
            key: "title",
            width: 160,
            className: "monitoring-incident-primary-column",
            render: (_, row) => (
                <div className="min-w-0">
                    <div className="truncate font-medium" title={row.title}>
                        {row.title}
                    </div>
                    <div
                        className="truncate text-xs text-muted-foreground"
                        title={`${row.nodeId} · ${row.target}`}
                    >
                        {row.nodeId} · {row.target}
                    </div>
                </div>
            ),
        },
        {
            title: t("类型", "Kind"),
            dataIndex: "kind",
            width: 130,
            responsive: ["sm"],
            className: "monitoring-incident-detail-column",
        },
        {
            title: t("状态", "Status"),
            dataIndex: "status",
            width: 96,
            render: (_, row) => (
                <Tag
                    color={row.status === "active" ? "error" : "success"}
                    icon={
                        row.status === "active" ? (
                            <ExclamationCircleOutlined />
                        ) : (
                            <CheckCircleOutlined />
                        )
                    }
                >
                    {row.status === "active" ? t("活动", "Active") : t("已解决", "Resolved")}
                </Tag>
            ),
        },
        {
            title: t("最近观察", "Last observed"),
            dataIndex: "lastObservedAt",
            width: 190,
            responsive: ["sm"],
            className: "monitoring-incident-detail-column",
            render: (_, row) => formatDateTime(row.lastObservedAt),
        },
        {
            title: t("详情", "Details"),
            key: "actions",
            width: actionColumnWidth(1),
            fixed: "right",
            render: (_, row) => (
                <Button
                    type="text"
                    icon={<EyeOutlined />}
                    aria-label={t("查看事件", "View incident")}
                    onClick={() => setSelected(row)}
                />
            ),
        },
    ];
    return (
        <PageCard
            toolbar={searchControls}
            actions={deliveryCard}
            title={t("告警事件", "Alert incidents")}
            description={t(
                "查看活动和最近解决的资源与离线告警。",
                "View active and recently resolved resource and offline alerts.",
            )}
        >
            {hasMonitorBackgroundRefreshFailure(data, error) ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            <DataTableShell fill ariaLabel={t("告警事件", "Alert incidents table")}>
                <ProTable
                    rowKey="id"
                    columns={columns}
                    dataSource={data.data}
                    loading={isFetching}
                    search={false}
                    options={false}
                    pagination={false}
                    scroll={{ y: "100%" }}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText: (
                            <DataState
                                kind="empty"
                                title={t("暂无告警事件", "No alert incidents")}
                            />
                        ),
                    }}
                />
                <div className="data-table-pagination">
                    <Typography.Text type="secondary">
                        {t(`共 ${data.total} 条`, `${data.total} total`)}
                    </Typography.Text>
                    <Pagination
                        current={current}
                        pageSize={PAGE_SIZE}
                        total={data.total}
                        showSizeChanger={false}
                        showLessItems
                        onChange={setCurrent}
                    />
                </div>
            </DataTableShell>
            <IncidentDrawer
                incident={selected}
                incidentId={selected?.id ?? linkedId}
                onClose={() => {
                    setSelected(undefined);
                    setLinkedId(undefined);
                }}
            />
        </PageCard>
    );
}
