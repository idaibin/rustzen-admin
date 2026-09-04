import { EyeOutlined, StopOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Tag } from "antd";
import { useEffect, useMemo, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { RunDetails } from "./-runs/run-details";
import { RunDialog } from "./-runs/run-dialog";
import { getRunStatusMeta, isActiveRun } from "./-runs/status";

export const Route = createFileRoute("/reports/runs")({ component: RunsPage });

const pageSize = 20;
const title = t("填报执行", "Report runs");
const description = t(
    "通过报表模板写入所选数据，并实时查看执行过程。",
    "Write selected data through a report template and monitor the run in real time.",
);

function RunsPage() {
    const locale = useLocale();
    const [current, setCurrent] = useState(1);
    const [selected, setSelected] = useState<Reports.Run>();
    const client = useQueryClient();
    const { data: flows = [] } = useQuery(reportsQueryOptions.flows());
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["reports", "runs", current],
        queryFn: () => reportsAPI.runs({ current, pageSize }),
        refetchInterval: (query) =>
            query.state.data?.data.some((run) => isActiveRun(run.status)) ? 1000 : false,
    });
    const linkedRunId =
        typeof window === "undefined"
            ? null
            : new URLSearchParams(window.location.search).get("runId");
    const { data: linkedRun } = useQuery({
        queryKey: ["reports", "run", linkedRunId],
        queryFn: () => reportsAPI.run(linkedRunId!),
        enabled: Boolean(linkedRunId),
    });
    const total = data?.total ?? 0;

    useEffect(() => {
        if (linkedRun) setSelected(linkedRun);
    }, [linkedRun]);
    useEffect(() => {
        if (data === undefined || isFetching) return;
        const lastPage = Math.max(1, Math.ceil(total / pageSize));
        if (current > lastPage) setCurrent(lastPage);
    }, [current, data, isFetching, total]);

    const cancel = useMutation({
        mutationFn: reportsAPI.cancelRun,
        onSuccess: async (run) => {
            await client.invalidateQueries({ queryKey: ["reports", "runs"] });
            appMessage.success(
                run.status === "cancelling"
                    ? t("正在停止填报执行", "Stopping report run")
                    : t("填报执行已取消", "Report run cancelled"),
            );
        },
    });
    const runStatusMeta = useMemo(getRunStatusMeta, [locale]);
    const columns: ProColumns<Reports.Run>[] = useMemo(
        () => [
            {
                title: t("执行", "Run"),
                dataIndex: "id",
                key: "id",
                width: 150,
                render: (_: unknown, row) => (
                    <span className="font-mono text-xs">{row.id.slice(0, 8)}</span>
                ),
            },
            {
                title: t("流程", "Template"),
                dataIndex: "flowId",
                key: "flow",
                width: 260,
                render: (_: unknown, row) =>
                    flows.find((flow) => flow.id === row.flowId)?.name ?? row.flowId,
            },
            {
                title: t("状态", "Status"),
                dataIndex: "status",
                key: "status",
                width: 130,
                render: (_: unknown, row) => {
                    const meta = runStatusMeta[row.status];
                    return <Tag color={meta.color}>{meta.label}</Tag>;
                },
            },
            {
                title: t("创建时间", "Created at"),
                dataIndex: "createdAt",
                key: "createdAt",
                width: 180,
                render: (_: unknown, row) => formatDateTime(row.createdAt),
            },
            {
                title: t("错误", "Error"),
                dataIndex: "error",
                key: "error",
                ellipsis: true,
                render: (_: unknown, row) => row.error ?? "-",
            },
            {
                title: t("操作", "Actions"),
                key: "actions",
                fixed: "right",
                width: 150,
                render: (_: unknown, row) => (
                    <div className="flex items-center justify-end gap-1">
                        <Button
                            type="text"
                            icon={<EyeOutlined />}
                            aria-label={t("查看执行", "View run")}
                            onClick={() => setSelected(row)}
                        />
                        <AuthWrap code="reports:run:manage">
                            <Button
                                type="text"
                                icon={<StopOutlined />}
                                danger
                                aria-label={t("取消执行", "Cancel run")}
                                disabled={!(row.status === "queued" || row.status === "running")}
                                onClick={() => cancel.mutate(row.id)}
                            />
                        </AuthWrap>
                    </div>
                ),
            },
        ],
        [cancel, flows, locale, runStatusMeta],
    );
    const actions = (
        <AuthWrap code="reports:run:manage">
            <RunDialog flows={flows} />
        </AuthWrap>
    );

    if (!data?.data.length && isPending)
        return (
            <PageCard title={title} description={description} actions={actions}>
                <DataState kind="loading" title={t("正在加载填报执行", "Loading report runs")} />
            </PageCard>
        );
    if (!data?.data.length && error)
        return (
            <PageCard title={title} description={description} actions={actions}>
                <DataState
                    kind="error"
                    title={t("填报执行加载失败", "Failed to load report runs")}
                    description={t(
                        "无法读取执行记录，请检查 Reports 服务后重试。",
                        "Unable to read run records. Check the Reports service and try again.",
                    )}
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </PageCard>
        );
    if (total === 0)
        return (
            <PageCard title={title} description={description} actions={actions}>
                <DataState
                    kind="empty"
                    title={t("暂无填报执行", "No report runs")}
                    description=""
                />
            </PageCard>
        );

    return (
        <PageCard title={title} description={description} actions={actions}>
            {error ? (
                <DataState
                    kind="error"
                    title={t("填报执行刷新失败", "Failed to refresh report runs")}
                    description={
                        error instanceof Error
                            ? error.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    compact
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            ) : null}
            <DataTableShell ariaLabel={t("报表运行记录", "Report runs table")}>
                <ProTable<Reports.Run>
                    rowKey="id"
                    columns={columns}
                    dataSource={data?.data ?? []}
                    loading={isFetching}
                    search={false}
                    options={false}
                    pagination={{
                        current,
                        pageSize,
                        total,
                        showSizeChanger: false,
                        onChange: setCurrent,
                    }}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText: (
                            <DataState kind="empty" title={t("暂无填报执行", "No report runs")} />
                        ),
                    }}
                />
            </DataTableShell>
            <RunDetails run={selected} onClose={() => setSelected(undefined)} />
        </PageCard>
    );
}
