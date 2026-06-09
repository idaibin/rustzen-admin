import { HistoryOutlined, PlayCircleOutlined } from "@ant-design/icons";
import { ModalForm, ProTable, type ActionType, type ProColumns } from "@ant-design/pro-components";
import { createFileRoute } from "@tanstack/react-router";
import { Space, Tag } from "antd";
import { useRef, useState } from "react";

import { appMessage, appModal, manageAPI } from "@/api";
import { AuthWrap } from "@/components/base-auth";
import {
    TABLE_ACTION_SPACE_SIZE,
    TableActionButton,
} from "@/components/base-button";

export const Route = createFileRoute("/manage/task")({
    component: TaskPage,
});

const taskStatusMeta: Record<Task.RunStatus, { label: string; color: string }> = {
    running: { label: "Running", color: "processing" },
    success: { label: "Success", color: "success" },
    failed: { label: "Failed", color: "error" },
    skipped: { label: "Skipped", color: "default" },
};

function TaskPage() {
    const actionRef = useRef<ActionType>(null);

    return (
        <ProTable<Task.Item>
            rowKey="taskKey"
            search={false}
            pagination={false}
            scroll={{ y: "calc(100vh - 287px)" }}
            headerTitle="Scheduled Tasks"
            columns={taskColumns}
            request={manageAPI.task.list}
            actionRef={actionRef}
        />
    );
}

const taskColumns: ProColumns<Task.Item>[] = [
    {
        title: "Name",
        dataIndex: "name",
        width: 190,
        ellipsis: true,
    },
    {
        title: "Description",
        dataIndex: "description",
        ellipsis: true,
        render: (_, record) => record.description || "-",
    },
    {
        title: "Cron",
        dataIndex: "schedule",
        width: 150,
        render: (_, record) => <Tag color="blue">{record.schedule.expression}</Tag>,
    },
    {
        title: "Status",
        dataIndex: "lastStatus",
        width: 110,
        render: (_, record) => renderTaskStatus(record.running ? "running" : record.lastStatus),
    },
    {
        title: "Next Run",
        dataIndex: "nextRunAt",
        width: 180,
        render: (_, record) => formatDateTime(record.nextRunAt),
    },
    {
        title: "Last Finished",
        dataIndex: "lastFinishedAt",
        width: 180,
        render: (_, record) => formatDateTime(record.lastFinishedAt),
    },
    {
        title: "Last Error",
        dataIndex: "lastErrorMessage",
        ellipsis: true,
        render: (_, record) =>
            record.lastErrorMessage ? (
                <span className="text-red-500" title={record.lastErrorMessage}>
                    {record.lastErrorMessage}
                </span>
            ) : (
                "-"
            ),
    },
    {
        title: "Actions",
        key: "actions",
        width: 68,
        fixed: "right",
        render: (_dom, record, _index, action) => (
            <Space size={TABLE_ACTION_SPACE_SIZE}>
                <TaskRunLogModal taskKey={record.taskKey} taskName={record.name} />
                <AuthWrap code="manage:task:run">
                    <TableActionButton
                        color="blue"
                        label={record.running ? "Executing" : "Execute"}
                        icon={<PlayCircleOutlined />}
                        disabled={record.running}
                        onClick={() => {
                            appModal.confirm({
                                title: `Execute ${record.name}?`,
                                content: record.description || "Submit this task immediately.",
                                okText: "Execute",
                                cancelText: "Cancel",
                                onOk: async () => {
                                    await manageAPI.task.run(record.taskKey);
                                    appMessage.success("Task execution submitted");
                                    void action?.reload();
                                },
                            });
                        }}
                    />
                </AuthWrap>
            </Space>
        ),
    },
];

const runColumns: ProColumns<Task.RunItem>[] = [
    {
        title: "Trigger",
        dataIndex: "triggerType",
        width: 120,
        render: (_, record) => (record.triggerType === "manual" ? "Manual" : "Scheduled"),
    },
    {
        title: "Status",
        dataIndex: "status",
        width: 120,
        render: (_, record) => renderTaskStatus(record.status),
    },
    {
        title: "Scheduled For",
        dataIndex: "scheduledFor",
        width: 180,
        render: (_, record) => formatDateTime(record.scheduledFor),
    },
    {
        title: "Started At",
        dataIndex: "startedAt",
        width: 180,
        render: (_, record) => formatDateTime(record.startedAt),
    },
    {
        title: "Finished At",
        dataIndex: "finishedAt",
        width: 180,
        render: (_, record) => formatDateTime(record.finishedAt),
    },
    {
        title: "Error",
        dataIndex: "errorMessage",
        ellipsis: true,
        render: (_, record) => record.errorMessage || "-",
    },
];

function TaskRunLogModal({ taskKey, taskName }: { taskKey: string; taskName: string }) {
    const [open, setOpen] = useState(false);

    return (
        <ModalForm<Record<string, never>>
            trigger={<TableActionButton color="default" label="Logs" icon={<HistoryOutlined />} />}
            submitter={false}
            title={`Task Logs - ${taskName}`}
            modalProps={{
                width: 1120,
                destroyOnHidden: true,
                maskClosable: false,
            }}
            onOpenChange={setOpen}
        >
            {open ? (
                <ProTable<Task.RunItem>
                    rowKey="id"
                    search={false}
                    options={false}
                    scroll={{ x: 1000, y: 480 }}
                    columns={runColumns}
                    request={(params) => manageAPI.task.runs(taskKey, params)}
                />
            ) : null}
        </ModalForm>
    );
}

function renderTaskStatus(status?: Task.RunStatus | null) {
    if (!status) {
        return <Tag color="default">Never Run</Tag>;
    }
    const meta = taskStatusMeta[status];
    return <Tag color={meta.color}>{meta.label}</Tag>;
}

function formatDateTime(value?: string | null) {
    if (!value) {
        return "-";
    }
    return new Date(value).toLocaleString();
}
