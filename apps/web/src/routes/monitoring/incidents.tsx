import { CheckCircleOutlined, ExclamationCircleOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Drawer, Pagination, Select, Space, Tag, Typography } from "antd";
import { useMemo, useState } from "react";

import { monitorAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/monitoring/incidents")({
    component: MonitoringIncidentsPage,
});
const PAGE_SIZE = 20;

function MonitoringIncidentsPage() {
    const [status, setStatus] = useState<"all" | Monitor.IncidentStatus>("all");
    const [kind, setKind] = useState<"all" | Monitor.IncidentKind>("all");
    const [current, setCurrent] = useState(1);
    const [selected, setSelected] = useState<Monitor.IncidentSummary>();
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
    });
    const searchControls = (
        <Space wrap>
            <Select
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

    if (!data)
        return (
            <PageCard
                toolbar={searchControls}
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
            render: (_, row) => (
                <div>
                    <div className="font-medium">{row.title}</div>
                    <div className="text-xs text-muted-foreground">
                        {row.nodeId} · {row.target}
                    </div>
                </div>
            ),
        },
        { title: t("类型", "Kind"), dataIndex: "kind", width: 130 },
        {
            title: t("状态", "Status"),
            dataIndex: "status",
            width: 120,
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
            render: (_, row) => formatDateTime(row.lastObservedAt),
        },
        {
            title: t("详情", "Details"),
            key: "actions",
            width: 88,
            fixed: "right",
            render: (_, row) => (
                <Button type="link" onClick={() => setSelected(row)}>
                    {t("查看", "View")}
                </Button>
            ),
        },
    ];
    return (
        <PageCard
            toolbar={searchControls}
            title={t("告警事件", "Alert incidents")}
            description={t(
                "查看活动和最近解决的资源与离线告警。",
                "View active and recently resolved resource and offline alerts.",
            )}
        >
            {error ? (
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
                            <DataState kind="empty" title={t("暂无告警事件", "No alert incidents")} />
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
            <IncidentDrawer incident={selected} onClose={() => setSelected(undefined)} />
        </PageCard>
    );
}

function IncidentDrawer({
    incident,
    onClose,
}: {
    incident?: Monitor.IncidentSummary;
    onClose: () => void;
}) {
    const { data, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "incident", incident?.id],
        queryFn: () => monitorAPI.incident(incident!.id),
        enabled: Boolean(incident),
    });
    return (
        <Drawer
            open={Boolean(incident)}
            onClose={onClose}
            title={data?.title ?? incident?.title ?? t("事件详情", "Incident details")}
            size="large"
            destroyOnHidden
        >
            {isPending ? (
                <DataState
                    kind="loading"
                    title={t("正在加载事件详情", "Loading incident details")}
                    compact
                />
            ) : error ? (
                <DataState
                    kind="error"
                    title={t("事件详情加载失败", "Failed to load incident details")}
                    action={<Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>}
                    compact
                />
            ) : data ? (
                <Space orientation="vertical" className="w-full">
                    <Typography.Text>
                        {data.node.hostname} · {data.node.nodeId}
                    </Typography.Text>
                    <Typography.Text>
                        {t("阈值", "Threshold")}: {data.thresholdPercent ?? "-"}% ·{" "}
                        {t("观测值", "Observed")}: {data.observedPercent ?? "-"}%
                    </Typography.Text>
                    <Typography.Text>
                        {t("打开时间", "Opened")}: {formatDateTime(data.openedAt)}
                    </Typography.Text>
                    <Typography.Text>
                        {t("解决原因", "Resolution reason")}: {data.resolutionReason ?? "-"}
                    </Typography.Text>
                    <pre className="max-h-72 overflow-auto rounded bg-muted p-3 text-xs">
                        {JSON.stringify(data.details, null, 2)}
                    </pre>
                </Space>
            ) : null}
        </Drawer>
    );
}
