import {
    ClockCircleOutlined,
    LoadingOutlined,
    PauseCircleOutlined,
    PlayCircleOutlined,
    ExclamationCircleOutlined,
    CheckCircleOutlined,
    HistoryOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Modal, Space, Tag, Tooltip, Typography } from "antd";
import { useMemo, useState } from "react";
import type { ReactNode } from "react";

import { appMessage, manageAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { localizeBuiltInTaskDescription, localizeBuiltInTaskName } from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

export const Route = createFileRoute("/manage/task")({
    component: TaskPage,
});

const RUN_PAGE_SIZE = 10;

function TaskPage() {
    const locale = useLocale();
    const { data, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["manage", "task"],
        queryFn: manageAPI.task.list,
    });
    const rows = data?.data ?? [];

    const columns: ProColumns<Task.Item>[] = useMemo(
        () => [
            {
                title: t("名称", "Name"),
                dataIndex: "name",
                key: "name",
                width: 140,
                ellipsis: true,
                render: (_: unknown, row: Task.Item) => {
                    const name = localizeBuiltInTaskName(row.taskKey, row.name);
                    return <Typography.Text ellipsis={{ tooltip: name }}>{name}</Typography.Text>;
                },
            },
            {
                title: t("描述", "Description"),
                dataIndex: "description",
                key: "description",
                width: 200,
                render: (_: unknown, row: Task.Item) => {
                    const description =
                        localizeBuiltInTaskDescription(row.taskKey, row.description) || "-";
                    return (
                        <Typography.Text ellipsis={{ tooltip: description }}>
                            {description}
                        </Typography.Text>
                    );
                },
            },
            {
                title: "Cron",
                dataIndex: ["schedule", "expression"],
                key: "cron",
                width: 110,
                render: (_: unknown, row: Task.Item) => (
                    <Tag color="blue">{row.schedule.expression}</Tag>
                ),
            },
            {
                title: t("状态", "Status"),
                dataIndex: "running",
                key: "status",
                width: 100,
                render: (_: unknown, row: Task.Item) => (
                    <TaskStatusBadge status={row.running ? "running" : row.lastStatus} />
                ),
            },
            {
                title: t("下次运行", "Next run"),
                dataIndex: "nextRunAt",
                key: "nextRunAt",
                width: 160,
                render: (_: unknown, row: Task.Item) => formatDateTime(row.nextRunAt),
            },
            {
                title: t("上次完成", "Last finished"),
                dataIndex: "lastFinishedAt",
                key: "lastFinishedAt",
                width: 150,
                render: (_: unknown, row: Task.Item) => formatDateTime(row.lastFinishedAt),
            },
            {
                title: t("上次错误", "Last error"),
                dataIndex: "lastErrorMessage",
                key: "lastErrorMessage",
                width: 120,
                render: (_: unknown, row: Task.Item) => {
                    const errorMessage = row.lastErrorMessage || "-";
                    return (
                        <Typography.Text ellipsis={{ tooltip: errorMessage }}>
                            {errorMessage}
                        </Typography.Text>
                    );
                },
            },
            {
                title: t("操作", "Actions"),
                key: "actions",
                width: 88,
                align: "left",
                render: (_: unknown, row: Task.Item) => (
                    <TaskActions record={row} onTaskUpdated={refetch} isFetching={isFetching} />
                ),
            },
        ],
        [isFetching, locale, refetch],
    );

    if (!rows.length && isPending) {
        return (
            <PageCard
                title={t("定时任务", "Scheduled tasks")}
                description={t(
                    "查看调度状态并手动运行维护任务。",
                    "View scheduling status and run maintenance tasks manually.",
                )}
            >
                <DataState kind="loading" title={t("正在加载任务", "Loading tasks")} />
            </PageCard>
        );
    }

    if (!rows.length && error) {
        return (
            <PageCard
                title={t("定时任务", "Scheduled tasks")}
                description={t(
                    "查看调度状态并手动运行维护任务。",
                    "View scheduling status and run maintenance tasks manually.",
                )}
            >
                <DataState
                    kind="error"
                    title={t("任务加载失败", "Failed to load tasks")}
                    description={
                        error instanceof Error
                            ? error.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </PageCard>
        );
    }

    if (!rows.length) {
        return (
            <PageCard
                title={t("定时任务", "Scheduled tasks")}
                description={t(
                    "查看调度状态并手动运行维护任务。",
                    "View scheduling status and run maintenance tasks manually.",
                )}
            >
                <DataState kind="empty" title={t("暂无定时任务", "No scheduled tasks")} />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("定时任务", "Scheduled tasks")}
            description={t(
                "查看调度状态并手动运行维护任务。",
                "View scheduling status and run maintenance tasks manually.",
            )}
        >
            <DataTableShell ariaLabel={t("维护任务", "Maintenance tasks table")}>
                <ProTable<Task.Item>
                    rowKey="taskKey"
                    columns={columns}
                    dataSource={rows}
                    loading={isFetching}
                    search={false}
                    options={false}
                    pagination={false}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    locale={{
                        emptyText: (
                            <DataState
                                kind="empty"
                                title={t("暂无定时任务", "No scheduled tasks")}
                            />
                        ),
                    }}
                    rowSelection={false}
                />
            </DataTableShell>
        </PageCard>
    );
}

function TaskActions({
    record,
    onTaskUpdated,
    isFetching,
}: {
    record: Task.Item;
    onTaskUpdated: () => void;
    isFetching: boolean;
}) {
    return (
        <Space size={0}>
            <TaskRunLogDialog
                taskKey={record.taskKey}
                taskName={localizeBuiltInTaskName(record.taskKey, record.name)}
            />
            <AuthWrap code="manage:task:run">
                <RunTaskDialog
                    record={record}
                    onSuccess={() => onTaskUpdated()}
                    disabled={isFetching || record.running}
                />
            </AuthWrap>
        </Space>
    );
}

function TaskRunLogDialog({ taskKey, taskName }: { taskKey: string; taskName: string }) {
    const locale = useLocale();
    const [open, setOpen] = useState(false);
    const [currentPage, setCurrentPage] = useState(1);
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["manage", "task", taskKey, "runs", currentPage],
        queryFn: () =>
            manageAPI.task.runs(taskKey, { current: currentPage, pageSize: RUN_PAGE_SIZE }),
        enabled: open,
    });
    const rows = data?.data ?? [];

    const columns: ProColumns<Task.RunItem>[] = useMemo(
        () => [
            {
                title: t("触发方式", "Trigger"),
                key: "triggerType",
                width: 120,
                render: (_: unknown, row: Task.RunItem) =>
                    row.triggerType === "manual" ? t("手动", "Manual") : t("定时", "Scheduled"),
            },
            {
                title: t("状态", "Status"),
                key: "status",
                width: 120,
                render: (_: unknown, row: Task.RunItem) => <TaskStatusBadge status={row.status} />,
            },
            {
                title: t("计划时间", "Scheduled for"),
                key: "scheduledFor",
                width: 190,
                render: (_: unknown, row: Task.RunItem) => formatDateTime(row.scheduledFor),
            },
            {
                title: t("开始时间", "Started at"),
                key: "startedAt",
                width: 190,
                render: (_: unknown, row: Task.RunItem) => formatDateTime(row.startedAt),
            },
            {
                title: t("完成时间", "Finished at"),
                key: "finishedAt",
                width: 190,
                render: (_: unknown, row: Task.RunItem) => formatDateTime(row.finishedAt),
            },
            {
                title: t("错误", "Error"),
                key: "errorMessage",
                render: (_: unknown, row: Task.RunItem) => <span>{row.errorMessage || "-"}</span>,
            },
        ],
        [locale],
    );

    return (
        <>
            <Tooltip title={t("任务日志", "Task logs")}>
                <Button
                    icon={<HistoryOutlined />}
                    type="text"
                    size="small"
                    aria-label={t("任务日志", "Task logs")}
                    onClick={() => setOpen(true)}
                />
            </Tooltip>
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                footer={
                    <Button type="primary" onClick={() => setOpen(false)}>
                        {t("关闭", "Close")}
                    </Button>
                }
                title={<span>{t(`任务日志 - ${taskName}`, `Task logs - ${taskName}`)}</span>}
                width="90%"
                destroyOnHidden
            >
                <Typography.Paragraph type="secondary">
                    {t("该任务最近的调度执行记录。", "Recent scheduled runs for this task.")}
                </Typography.Paragraph>
                {error ? (
                    <DataState
                        kind="error"
                        title={t("任务日志加载失败", "Failed to load task logs")}
                        description={
                            error instanceof Error
                                ? error.message
                                : t("请稍后重试。", "Please try again later.")
                        }
                        action={
                            <Button
                                type="primary"
                                onClick={() => {
                                    void refetch();
                                }}
                            >
                                {t("重新加载", "Reload")}
                            </Button>
                        }
                    />
                ) : null}
                <ProTable<Task.RunItem>
                    rowKey="id"
                    rowSelection={false}
                    search={false}
                    options={false}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    columns={columns}
                    dataSource={rows}
                    loading={isFetching}
                    pagination={{
                        current: currentPage,
                        pageSize: RUN_PAGE_SIZE,
                        total: data?.total ?? 0,
                        showSizeChanger: false,
                        onChange: (page) => {
                            setCurrentPage(page);
                        },
                    }}
                    locale={{
                        emptyText:
                            !rows.length && !isPending ? (
                                <DataState
                                    kind="empty"
                                    title={t("暂无任务执行记录", "No task runs")}
                                />
                            ) : undefined,
                    }}
                />
            </Modal>
        </>
    );
}

function RunTaskDialog({
    record,
    onSuccess,
    disabled,
}: {
    record: Task.Item;
    onSuccess: () => void;
    disabled: boolean;
}) {
    const [open, setOpen] = useState(false);
    const [isSubmitting, setIsSubmitting] = useState(false);

    const submit = async () => {
        setIsSubmitting(true);
        try {
            await manageAPI.task.run(record.taskKey);
            appMessage.success(t("任务执行已提交", "Task run submitted"));
            setOpen(false);
            onSuccess();
        } finally {
            setIsSubmitting(false);
        }
    };

    return (
        <>
            <Tooltip title={record.running ? t("执行中", "Running") : t("执行任务", "Run task")}>
                <Button
                    type="link"
                    size="small"
                    icon={<PlayCircleOutlined />}
                    disabled={disabled}
                    loading={isSubmitting}
                    onClick={() => setOpen(true)}
                    aria-label={record.running ? t("执行中", "Running") : t("执行任务", "Run task")}
                />
            </Tooltip>
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                onOk={submit}
                okText={t("执行", "Run")}
                cancelText={t("取消", "Cancel")}
                okButtonProps={{ loading: isSubmitting }}
                title={
                    <span>
                        {t(
                            `执行 ${localizeBuiltInTaskName(record.taskKey, record.name)}？`,
                            `Run ${localizeBuiltInTaskName(record.taskKey, record.name)}?`,
                        )}
                    </span>
                }
                centered
            >
                <Form layout="vertical">
                    <Form.Item>
                        <span>
                            {localizeBuiltInTaskDescription(record.taskKey, record.description) ||
                                t("立即提交此任务。", "Submit this task immediately.")}
                        </span>
                    </Form.Item>
                </Form>
            </Modal>
        </>
    );
}

function TaskStatusBadge({ status }: { status?: Task.RunStatus | "never" | null }) {
    const taskStatusMeta: Record<
        "running" | "success" | "failed" | "skipped" | "never",
        { label: string; color: string; icon: ReactNode }
    > = {
        running: {
            label: t("运行中", "Running"),
            color: "blue",
            icon: <LoadingOutlined />,
        },
        success: {
            label: t("成功", "Success"),
            color: "green",
            icon: <CheckCircleOutlined />,
        },
        failed: {
            label: t("失败", "Failed"),
            color: "red",
            icon: <ExclamationCircleOutlined />,
        },
        skipped: {
            label: t("已跳过", "Skipped"),
            color: "default",
            icon: <PauseCircleOutlined />,
        },
        never: {
            label: t("从未运行", "Never run"),
            color: "orange",
            icon: <ClockCircleOutlined />,
        },
    };

    const meta = taskStatusMeta[status ?? "never"];

    return (
        <Tag color={meta.color} icon={meta.icon}>
            {meta.label}
        </Tag>
    );
}
