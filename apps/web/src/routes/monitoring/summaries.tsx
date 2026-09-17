import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Pagination, Typography } from "antd";
import { useMemo, useState } from "react";

import { monitorAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { t } from "@/lib/i18n";

import { hasMonitorBackgroundRefreshFailure, isMonitorPermissionDenied } from "./-save-state";

export const Route = createFileRoute("/monitoring/summaries")({ component: DailySummariesPage });
const PAGE_SIZE = 20;

function DailySummariesPage() {
    const [current, setCurrent] = useState(1);
    const query = useMemo(() => ({ current, pageSize: PAGE_SIZE }), [current]);
    const { data, dataUpdatedAt, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["monitor", "daily-summaries", query],
        queryFn: () => monitorAPI.dailySummaries(query),
        refetchInterval: 30_000,
        retry: false,
    });
    const columns: ProColumns<Monitor.DailySummary>[] = [
        { title: t("日期", "Date"), dataIndex: "date", width: 104 },
        { title: t("节点", "Node"), dataIndex: "nodeId", ellipsis: true },
        {
            title: t("样本", "Samples"),
            dataIndex: "sampleCount",
            width: 100,
            responsive: ["sm"],
            className: "monitoring-summary-detail-column",
        },
        {
            title: t("覆盖率", "Coverage"),
            dataIndex: "coveragePercent",
            width: 88,
            render: (_, row) => `${row.coveragePercent.toFixed(1)}%`,
        },
        {
            title: "CPU min/avg/max",
            key: "cpu",
            responsive: ["sm"],
            className: "monitoring-summary-detail-column",
            render: (_, row) => formatSummaryRange(row.cpu, row.sampleCount),
        },
        {
            title: t("内存 min/avg/max", "Memory min/avg/max"),
            key: "memory",
            responsive: ["sm"],
            className: "monitoring-summary-detail-column",
            render: (_, row) => formatSummaryRange(row.memory, row.sampleCount),
        },
        {
            title: t("磁盘挂载点 min/avg/max", "Disk mounts min/avg/max"),
            key: "diskSummary",
            responsive: ["sm"],
            className: "monitoring-summary-detail-column",
            render: (_, row) => (
                <div className="space-y-1 text-xs">
                    {Object.entries(row.diskSummary).map(([mountPoint, value]) => (
                        <div key={mountPoint}>
                            {mountPoint}: {formatSummaryRange(value, row.sampleCount)}
                        </div>
                    ))}
                </div>
            ),
        },
        {
            title: t("离线", "Offline"),
            dataIndex: "offlineSeconds",
            width: 100,
            responsive: ["sm"],
            className: "monitoring-summary-detail-column",
            render: (_, row) => `${row.offlineSeconds}s`,
        },
        {
            title: t("告警", "Incidents"),
            dataIndex: "incidentCount",
            width: 100,
            responsive: ["sm"],
            className: "monitoring-summary-detail-column",
        },
    ];
    return (
        <PageCard
            title={t("日报", "Daily summaries")}
            description={t(
                "保留最近 30 天的节点资源与告警汇总。",
                "Resource and alert summaries are retained for the latest 30 days.",
            )}
        >
            {isMonitorPermissionDenied(error) ? (
                <DataState
                    kind="permission"
                    title={t(
                        "没有查看监控日报的权限",
                        "You do not have permission to view daily summaries",
                    )}
                    description={t(
                        "无法读取监控日报，请检查权限后重试。",
                        "Unable to read daily summaries. Check your permission and try again.",
                    )}
                    action={
                        <Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>
                    }
                />
            ) : !data ? (
                <DataState
                    kind={isPending ? "loading" : "error"}
                    title={
                        isPending
                            ? t("正在加载日报", "Loading daily summaries")
                            : t("日报加载失败", "Failed to load daily summaries")
                    }
                    action={
                        !isPending ? (
                            <Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>
                        ) : undefined
                    }
                />
            ) : (
                <DataTableShell fill ariaLabel={t("监控日报", "Monitoring daily summaries")}>
                    {hasMonitorBackgroundRefreshFailure(data, error) ? (
                        <BackgroundRefreshNotice
                            updatedAt={dataUpdatedAt}
                            onRetry={() => void refetch()}
                        />
                    ) : null}
                    <ProTable
                        rowKey={(row) => `${row.nodeId}-${row.date}`}
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
                                    title={t("暂无日报", "No daily summaries")}
                                />
                            ),
                        }}
                    />
                    <div className="flex shrink-0 items-center justify-between gap-4 pt-3">
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
            )}
        </PageCard>
    );
}
export function formatSummaryRange(value: Monitor.SummaryRange, sampleCount: number) {
    if (sampleCount === 0 || value.min === null || value.avg === null || value.max === null) {
        return "—";
    }
    return `${value.min.toFixed(1)} / ${value.avg.toFixed(1)} / ${value.max.toFixed(1)}%`;
}
