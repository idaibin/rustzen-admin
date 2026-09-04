import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button } from "antd";
import { useState } from "react";

import { monitorAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/monitoring/summaries")({ component: DailySummariesPage });
const PAGE_SIZE = 20;

function DailySummariesPage() {
    const [current, setCurrent] = useState(1);
    const query = { current, pageSize: PAGE_SIZE };
    const { data, isPending, isFetching, refetch } = useQuery({
        queryKey: ["monitor", "daily-summaries", query],
        queryFn: () => monitorAPI.dailySummaries(query),
    });
    const columns: ProColumns<Monitor.DailySummary>[] = [
        { title: t("日期", "Date"), dataIndex: "date", width: 120 },
        { title: t("节点", "Node"), dataIndex: "nodeId" },
        { title: t("样本", "Samples"), dataIndex: "sampleCount", width: 100 },
        {
            title: t("覆盖率", "Coverage"),
            dataIndex: "coveragePercent",
            width: 100,
            render: (_, row) => `${row.coveragePercent.toFixed(1)}%`,
        },
        {
            title: "CPU min/avg/max",
            key: "cpu",
            render: (_, row) => formatSummaryRange(row.cpu, row.sampleCount),
        },
        {
            title: t("内存 min/avg/max", "Memory min/avg/max"),
            key: "memory",
            render: (_, row) => formatSummaryRange(row.memory, row.sampleCount),
        },
        {
            title: t("磁盘挂载点 min/avg/max", "Disk mounts min/avg/max"),
            key: "diskSummary",
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
            render: (_, row) => `${row.offlineSeconds}s`,
        },
        { title: t("告警", "Incidents"), dataIndex: "incidentCount", width: 100 },
    ];
    return (
        <PageCard
            title={t("日报", "Daily summaries")}
            description={t(
                "保留最近 30 天的节点资源与告警汇总。",
                "Resource and alert summaries are retained for the latest 30 days.",
            )}
        >
            {!data ? (
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
                <DataTableShell ariaLabel={t("监控日报", "Monitoring daily summaries")}>
                    <ProTable
                        rowKey={(row) => `${row.nodeId}-${row.date}`}
                        columns={columns}
                        dataSource={data.data}
                        loading={isFetching}
                        search={false}
                        options={false}
                        pagination={{
                            current,
                            pageSize: PAGE_SIZE,
                            total: data.total,
                            showSizeChanger: false,
                            onChange: setCurrent,
                        }}
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
