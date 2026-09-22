import { EyeOutlined, StopOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Space, Tag } from "antd";
import { useEffect, useMemo, useRef, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { NotificationDeliveryCard } from "@/components/feedback/notification-delivery-card";
import { PageCard } from "@/components/page/page-card";
import { actionColumnWidth } from "@/components/table/action-column";
import { DataTableShell } from "@/components/table/data-table-shell";
import {
    emptyTableLocale,
    pagedTableProps,
    tablePagination,
} from "@/components/table/table-presets";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { RetryRunButton } from "./-runs/retry-run-button";
import { RunDetails } from "./-runs/run-details";
import { RunDialog } from "./-runs/run-dialog";
import { getRunStatusMeta, isActiveRun } from "./-runs/status";
import { REPORTS_FLOW_VIEW } from "./-schedule-permissions";

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
    const generation = useAuthStore((state) => state.authGeneration);
    const selectedGeneration = useRef(generation);
    const visibleSelected = selectedGeneration.current === generation ? selected : undefined;
    const canViewFlows = useAuthStore((state) => state.checkPermissions(REPORTS_FLOW_VIEW));
    const { data: flows = [] } = useQuery({
        ...reportsQueryOptions.flows(),
        enabled: canViewFlows,
    });
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
    const {
        data: linkedRun,
        error: linkedRunError,
        isFetching: linkedRunFetching,
    } = useQuery({
        queryKey: ["reports", "run", generation, linkedRunId],
        queryFn: () => reportsAPI.run(linkedRunId!),
        enabled: Boolean(linkedRunId),
        retry: false,
        staleTime: 0,
        refetchOnMount: "always",
    });
    const total = data?.total ?? 0;
    const deliveryCard = (
        <NotificationDeliveryCard
            queryKey={["reports", "notification-delivery"]}
            queryFn={reportsAPI.notificationDelivery}
        />
    );

    useEffect(() => {
        if (linkedRunError || linkedRunFetching) {
            setSelected(undefined);
            return;
        }
        if (linkedRun) setSelected(linkedRun);
    }, [linkedRun, linkedRunError, linkedRunFetching]);
    useEffect(() => {
        selectedGeneration.current = generation;
        setSelected(undefined);
    }, [generation]);
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
                width: 200,
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
                width: actionColumnWidth(3),
                render: (_: unknown, row) => (
                    <div className="flex items-center gap-1">
                        <Button
                            type="text"
                            icon={<EyeOutlined />}
                            data-testid={`run-view-${row.id}`}
                            aria-label={t("查看执行", "View run")}
                            onClick={() => setSelected(row)}
                        />
                        <RetryRunButton run={row} onRetried={setSelected} surface="list" />
                        <AuthWrap code="reports:run:manage">
                            <Button
                                type="text"
                                icon={<StopOutlined />}
                                danger
                                data-testid={`run-cancel-${row.id}`}
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
        <Space>
            {deliveryCard}
            <AuthWrap code="reports:run:manage">
                <RunDialog flows={flows} />
            </AuthWrap>
        </Space>
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
                    {...pagedTableProps}
                    pagination={tablePagination({
                        current,
                        pageSize,
                        total,
                        onChange: setCurrent,
                    })}
                    locale={emptyTableLocale(t("暂无填报执行", "No report runs"))}
                />
            </DataTableShell>
            <RunDetails
                run={visibleSelected}
                onClose={() => setSelected(undefined)}
                onRetried={setSelected}
            />
        </PageCard>
    );
}
