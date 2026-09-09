import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Tag, Typography } from "antd";
import { useMemo } from "react";

import { manageAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { localizeBuiltInTaskDescription, localizeBuiltInTaskName } from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { TaskActions, TaskStatusBadge } from "./-task-console-actions";
import { taskListRefreshInterval, taskQueryKeys } from "./-task-refresh";

export const Route = createFileRoute("/manage/task")({ component: TaskPage });

function TaskPage() {
    const locale = useLocale();
    const { data, error, isPending, isFetching, refetch } = useQuery({
        queryKey: taskQueryKeys.list(),
        queryFn: manageAPI.task.list,
        refetchInterval: (query) => taskListRefreshInterval(query.state.data?.data),
    });
    const rows = data?.data ?? [];
    const columns: ProColumns<Task.Item>[] = useMemo(
        () => [
            {
                title: t("名称", "Name"), dataIndex: "name", key: "name", width: 140, ellipsis: true,
                render: (_: unknown, row: Task.Item) => {
                    const name = localizeBuiltInTaskName(row.taskKey, row.name);
                    return <Typography.Text ellipsis={{ tooltip: name }}>{name}</Typography.Text>;
                },
            },
            {
                title: t("描述", "Description"), dataIndex: "description", key: "description", ellipsis: true,
                render: (_: unknown, row: Task.Item) => {
                    const description = localizeBuiltInTaskDescription(row.taskKey, row.description) || "-";
                    return <Typography.Text ellipsis={{ tooltip: description }}>{description}</Typography.Text>;
                },
            },
            { title: "Cron", dataIndex: ["schedule", "expression"], key: "cron", width: 110, render: (_: unknown, row: Task.Item) => <Tag color="blue">{row.schedule.expression}</Tag> },
            { title: t("状态", "Status"), dataIndex: "running", key: "status", width: 100, render: (_: unknown, row: Task.Item) => <TaskStatusBadge testId={`maintenance-task-status-${row.taskKey}`} status={row.running ? "running" : row.lastStatus} /> },
            { title: t("下次运行", "Next run"), dataIndex: "nextRunAt", key: "nextRunAt", width: 160, render: (_: unknown, row: Task.Item) => formatDateTime(row.nextRunAt) },
            { title: t("上次完成", "Last finished"), dataIndex: "lastFinishedAt", key: "lastFinishedAt", width: 150, render: (_: unknown, row: Task.Item) => formatDateTime(row.lastFinishedAt) },
            {
                title: t("上次错误", "Last error"), dataIndex: "lastErrorMessage", key: "lastErrorMessage", width: 120,
                render: (_: unknown, row: Task.Item) => <Typography.Text ellipsis={{ tooltip: row.lastErrorMessage || "-" }}>{row.lastErrorMessage || "-"}</Typography.Text>,
            },
            { title: t("操作", "Actions"), key: "actions", width: 72, fixed: "right", align: "center", render: (_: unknown, row: Task.Item) => <TaskActions record={row} onTaskUpdated={refetch} isFetching={isFetching} /> },
        ],
        [isFetching, locale, refetch],
    );
    return <TaskContent rows={rows} columns={columns} loading={isFetching} pending={isPending} error={error} refetch={refetch} />;
}

function TaskContent({ rows, columns, loading, pending, error, refetch }: { rows: Task.Item[]; columns: ProColumns<Task.Item>[]; loading: boolean; pending: boolean; error: unknown; refetch: () => void }) {
    const title = t("定时任务", "Scheduled tasks");
    const description = t("查看调度状态并手动运行维护任务。", "View scheduling status and run maintenance tasks manually.");
    if (!rows.length && pending) return <TaskCard title={title} description={description}><DataState kind="loading" title={t("正在加载任务", "Loading tasks")} /></TaskCard>;
    if (!rows.length && error) return <TaskCard title={title} description={description}><DataState kind="error" title={t("任务加载失败", "Failed to load tasks")} description={error instanceof Error ? error.message : t("请稍后重试。", "Please try again later.")} action={<Button type="primary" onClick={refetch}>{t("重新加载", "Reload")}</Button>} /></TaskCard>;
    if (!rows.length) return <TaskCard title={title} description={description}><DataState kind="empty" title={t("暂无定时任务", "No scheduled tasks")} /></TaskCard>;
    return <TaskCard title={title} description={description}><DataTableShell fill ariaLabel={t("维护任务", "Maintenance tasks table")} data-testid="maintenance-task-table"><ProTable<Task.Item> rowKey="taskKey" columns={columns} dataSource={rows} loading={loading} search={false} options={false} pagination={false} scroll={{ x: "max-content", y: "100%" }} toolBarRender={false} tableAlertOptionRender={false} rowSelection={false} locale={{ emptyText: <DataState kind="empty" title={t("暂无定时任务", "No scheduled tasks")} /> }} /></DataTableShell></TaskCard>;
}

function TaskCard({ title, description, children }: { title: string; description: string; children: React.ReactNode }) {
    return <PageCard title={title} description={description}>{children}</PageCard>;
}
