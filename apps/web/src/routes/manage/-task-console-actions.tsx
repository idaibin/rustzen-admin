import {
    CheckCircleOutlined,
    ClockCircleOutlined,
    ExclamationCircleOutlined,
    HistoryOutlined,
    LoadingOutlined,
    PauseCircleOutlined,
    PlayCircleOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Button, Form, Modal, Space, Tag, Tooltip, Typography } from "antd";
import { useMemo, useState, type ReactNode } from "react";

import { appMessage, manageAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { localizeBuiltInTaskDescription, localizeBuiltInTaskName } from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { taskQueryKeys, taskRunsRefreshInterval } from "./-task-refresh";

const RUN_PAGE_SIZE = 10;

export function TaskActions({
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
                    onSuccess={onTaskUpdated}
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
        queryKey: taskQueryKeys.runsPage(taskKey, currentPage),
        queryFn: () =>
            manageAPI.task.runs(taskKey, { current: currentPage, pageSize: RUN_PAGE_SIZE }),
        enabled: open,
        staleTime: 0,
        refetchInterval: (query) => taskRunsRefreshInterval(open, query.state.data?.data),
    });
    const rows = data?.data ?? [];
    const columns: ProColumns<Task.RunItem>[] = useMemo(() => runColumns(), [locale]);
    return (
        <>
            <Tooltip title={t("任务日志", "Task logs")}>
                <Button
                    data-testid={`maintenance-task-records-${taskKey}`}
                    icon={<HistoryOutlined />}
                    type="text"
                    size="small"
                    aria-label={t("任务日志", "Task logs")}
                    onClick={() => setOpen(true)}
                />
            </Tooltip>
            <Modal
                data-testid={`maintenance-task-records-dialog-${taskKey}`}
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
                            <Button type="primary" onClick={() => void refetch()}>
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
                        onChange: setCurrentPage,
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

function runColumns(): ProColumns<Task.RunItem>[] {
    return [
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
            render: (_: unknown, row: Task.RunItem) => (
                <TaskStatusBadge
                    testId={`maintenance-task-run-status-${row.id}`}
                    status={row.status}
                />
            ),
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
    ];
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
    const queryClient = useQueryClient();
    const [open, setOpen] = useState(false);
    const [isSubmitting, setIsSubmitting] = useState(false);
    const submit = async () => {
        setIsSubmitting(true);
        try {
            await manageAPI.task.run(record.taskKey);
            await queryClient.invalidateQueries({ queryKey: taskQueryKeys.runs(record.taskKey) });
            appMessage.success(t("任务执行已提交", "Task run submitted"));
            setOpen(false);
            onSuccess();
        } finally {
            setIsSubmitting(false);
        }
    };
    const label = record.running ? t("执行中", "Running") : t("执行任务", "Run task");
    return (
        <>
            <Tooltip title={label}>
                <Button
                    data-testid={`maintenance-task-run-${record.taskKey}`}
                    type="link"
                    size="small"
                    icon={<PlayCircleOutlined />}
                    disabled={disabled}
                    loading={isSubmitting}
                    onClick={() => setOpen(true)}
                    aria-label={label}
                />
            </Tooltip>
            <Modal
                data-testid={`maintenance-task-run-dialog-${record.taskKey}`}
                open={open}
                onCancel={() => setOpen(false)}
                onOk={() => void submit()}
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
                        {localizeBuiltInTaskDescription(record.taskKey, record.description) ||
                            t("立即提交此任务。", "Submit this task immediately.")}
                    </Form.Item>
                </Form>
            </Modal>
        </>
    );
}

export function TaskStatusBadge({
    status,
    testId,
}: {
    status?: Task.RunStatus | "never" | null;
    testId?: string;
}) {
    const meta: Record<
        "running" | "success" | "failed" | "skipped" | "never",
        { label: string; color: string; icon: ReactNode }
    > = {
        running: { label: t("运行中", "Running"), color: "blue", icon: <LoadingOutlined /> },
        success: { label: t("成功", "Success"), color: "green", icon: <CheckCircleOutlined /> },
        failed: { label: t("失败", "Failed"), color: "red", icon: <ExclamationCircleOutlined /> },
        skipped: { label: t("已跳过", "Skipped"), color: "default", icon: <PauseCircleOutlined /> },
        never: {
            label: t("从未运行", "Never run"),
            color: "orange",
            icon: <ClockCircleOutlined />,
        },
    };
    const value = meta[status ?? "never"];
    return (
        <Tag
            data-testid={testId}
            data-status={status ?? "never"}
            color={value.color}
            icon={value.icon}
        >
            {value.label}
        </Tag>
    );
}
