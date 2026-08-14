import {
    CloudUploadOutlined,
    DeleteOutlined,
    FileDoneOutlined,
    FileSearchOutlined,
    UploadOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Input, Upload, Button, Modal, Form, Tag } from "antd";
import type { UploadFile } from "antd/es/upload/interface";
import { useEffect, useState } from "react";

import { appMessage, manageAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/manage/deploy")({
    component: DeployPage,
});

const PAGE_SIZE = 20;

function DeployPage() {
    const [currentPage, setCurrentPage] = useState(1);
    const params: Deploy.ListParams = { current: currentPage, pageSize: PAGE_SIZE };
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["manage", "deploy", params],
        queryFn: () => manageAPI.deploy.list(params),
    });
    const rows = data?.data ?? [];
    const total = data?.total ?? 0;

    useEffect(() => {
        if (data === undefined || isFetching) {
            return;
        }
        const lastPage = Math.max(1, Math.ceil(total / PAGE_SIZE));
        if (currentPage > lastPage) {
            setCurrentPage(lastPage);
        }
    }, [currentPage, data, isFetching, total]);

    const refresh = () => {
        void refetch();
    };

    const columns: ProColumns<Deploy.Item>[] = [
        {
            title: t("组件", "Component"),
            dataIndex: "component",
            key: "component",
            width: 120,
            render: (_: unknown, row: Deploy.Item) => componentLabel(row.component),
        },
        {
            title: t("版本", "Version"),
            dataIndex: "version",
            key: "version",
            width: 120,
            render: (_: unknown, row: Deploy.Item) => (
                <span className="font-medium">{row.version}</span>
            ),
        },
        {
            title: t("架构", "Architecture"),
            key: "arch",
            width: 96,
            render: (_: unknown, row: Deploy.Item) => row.arch || "-",
        },
        {
            title: t("大小", "Size"),
            dataIndex: "fileSize",
            key: "fileSize",
            width: 110,
            render: (_: unknown, row: Deploy.Item) => formatFileSize(row.fileSize),
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 120,
            render: (_: unknown, row: Deploy.Item) => <DeployStatusBadge record={row} />,
        },
        {
            title: t("部署人", "Deployed by"),
            dataIndex: "deployedBy",
            key: "deployedBy",
            width: 140,
            render: (_: unknown, row: Deploy.Item) => row.deployedBy || "-",
        },
        {
            title: t("部署时间", "Deployed at"),
            dataIndex: "deployedAt",
            key: "deployedAt",
            width: 190,
            render: (_: unknown, row: Deploy.Item) => formatDateTime(row.deployedAt),
        },
        {
            title: t("过期时间", "Expired at"),
            dataIndex: "expiredAt",
            key: "expiredAt",
            width: 190,
            render: (_: unknown, row: Deploy.Item) => formatDateTime(row.expiredAt),
        },
        {
            title: t("备注", "Notes"),
            dataIndex: "notes",
            key: "notes",
            width: 220,
            render: (_: unknown, row: Deploy.Item) => row.notes || "-",
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 150,
            render: (_: unknown, row: Deploy.Item) => (
                <DeployActions record={row} onSuccess={refresh} />
            ),
        },
    ];

    if (!rows.length && isPending) {
        return (
            <PageCard
                title={t("部署版本", "Deployment versions")}
                description={t(
                    "上传签名的 rz 完整发行包并应用到四个服务。",
                    "Upload a signed complete rz release bundle and apply it to all four services.",
                )}
                actions={
                    <div className="flex flex-wrap gap-2">
                        <AuthWrap code="manage:deploy:create">
                            <UploadVersionDialog onSuccess={refresh} />
                        </AuthWrap>
                        <AuthWrap code="manage:deploy:delete">
                            <CleanupDialog onSuccess={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="loading"
                    title={t("正在加载部署版本", "Loading deployment versions")}
                />
            </PageCard>
        );
    }

    if (!rows.length && error) {
        return (
            <PageCard
                title={t("部署版本", "Deployment versions")}
                description={t(
                    "上传签名的 rz 完整发行包并应用到四个服务。",
                    "Upload a signed complete rz release bundle and apply it to all four services.",
                )}
                actions={
                    <div className="flex flex-wrap gap-2">
                        <AuthWrap code="manage:deploy:create">
                            <UploadVersionDialog onSuccess={refresh} />
                        </AuthWrap>
                        <AuthWrap code="manage:deploy:delete">
                            <CleanupDialog onSuccess={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="error"
                    title={t("部署版本加载失败", "Failed to load deployment versions")}
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

    if (total === 0) {
        return (
            <PageCard
                title={t("部署版本", "Deployment versions")}
                description={t(
                    "上传签名的 rz 完整发行包并应用到四个服务。",
                    "Upload a signed complete rz release bundle and apply it to all four services.",
                )}
                actions={
                    <div className="flex flex-wrap gap-2">
                        <AuthWrap code="manage:deploy:create">
                            <UploadVersionDialog onSuccess={refresh} />
                        </AuthWrap>
                        <AuthWrap code="manage:deploy:delete">
                            <CleanupDialog onSuccess={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState kind="empty" title={t("暂无部署版本", "No deployment versions")} />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("部署版本", "Deployment versions")}
            description={t(
                "上传签名的 rz 完整发行包并应用到四个服务。",
                "Upload a signed complete rz release bundle and apply it to all four services.",
            )}
            actions={
                <div className="flex flex-wrap gap-2">
                    <AuthWrap code="manage:deploy:create">
                        <UploadVersionDialog onSuccess={refresh} />
                    </AuthWrap>
                    <AuthWrap code="manage:deploy:delete">
                        <CleanupDialog onSuccess={refresh} />
                    </AuthWrap>
                </div>
            }
        >
            {error ? (
                <DataState
                    kind="error"
                    title={t("部署版本刷新失败", "Failed to refresh deployment versions")}
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
                    compact
                />
            ) : null}
            <DataTableShell ariaLabel={t("部署记录", "Deployments table")}>
                <ProTable<Deploy.Item>
                    rowKey="id"
                    columns={columns}
                    dataSource={rows}
                    loading={isFetching}
                    search={false}
                    options={false}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    pagination={{
                        current: currentPage,
                        pageSize: PAGE_SIZE,
                        total,
                        showSizeChanger: false,
                        onChange: (page) => {
                            setCurrentPage(page);
                        },
                    }}
                    locale={{
                        emptyText: (
                            <DataState
                                kind="empty"
                                title={t("暂无部署版本", "No deployment versions")}
                            />
                        ),
                    }}
                />
            </DataTableShell>
        </PageCard>
    );
}

function DeployActions({ record, onSuccess }: { record: Deploy.Item; onSuccess: () => void }) {
    return (
        <div className="flex justify-end gap-2">
            <AuthWrap code="manage:deploy:run">
                <DeployVersionDialog record={record} onSuccess={onSuccess} />
            </AuthWrap>
            <AuthWrap code="manage:deploy:update">
                <ExpireVersionDialog version={record} onSuccess={onSuccess} />
            </AuthWrap>
            <AuthWrap code="manage:deploy:delete">
                <DeleteVersionDialog record={record} onSuccess={onSuccess} />
            </AuthWrap>
        </div>
    );
}

function UploadVersionDialog({ onSuccess }: { onSuccess?: () => void }) {
    const [open, setOpen] = useState(false);
    const [version, setVersion] = useState("");
    const [arch, setArch] = useState("");
    const [notes, setNotes] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const [fileList, setFileList] = useState<UploadFile[]>([]);

    const selectedFile = fileList[0]?.originFileObj ?? null;

    const reset = () => {
        setVersion("");
        setArch("");
        setNotes("");
        setFileList([]);
    };

    const submit = async () => {
        if (!version.trim()) {
            appMessage.error(t("请输入版本号", "Enter a version number"));
            return;
        }
        if (!selectedFile) {
            appMessage.error(t("请选择部署文件", "Select a deployment file"));
            return;
        }

        setSubmitting(true);
        try {
            await manageAPI.deploy.upload({
                version: version.trim(),
                arch: arch.trim() || undefined,
                notes: notes.trim() || undefined,
                file: selectedFile,
            });
            appMessage.success(t("上传成功", "Upload completed"));
            onSuccess?.();
            reset();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button type="default" icon={<UploadOutlined />} onClick={() => setOpen(true)}>
                {t("上传版本", "Upload version")}
            </Button>
            <Modal
                open={open}
                onCancel={() => {
                    setOpen(false);
                    reset();
                }}
                footer={null}
                title={t("上传完整发行包", "Upload complete release bundle")}
                width="650px"
                destroyOnHidden
            >
                <Form layout="vertical">
                    <Form.Item
                        label={t("版本", "Version")}
                        required
                        tooltip={t("用于展示与回滚核验", "Used for tracking and rollback checks")}
                    >
                        <Input
                            value={version}
                            placeholder="0.5.0"
                            onChange={(event) => setVersion(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("架构", "Architecture")}>
                        <Input
                            value={arch}
                            placeholder="x86_64"
                            onChange={(event) => setArch(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item
                        label={t("文件", "File")}
                        required
                        extra={t(
                            "上传一个包含 Admin、Monitor、Insights、Reports、Web，及部署文件的签名 tar 完整包。",
                            "Upload a signed complete tar bundle containing Admin, Monitor, Insights, Reports, Web, and deployment files.",
                        )}
                    >
                        <Upload.Dragger
                            multiple={false}
                            maxCount={1}
                            accept=".tar,application/x-tar"
                            beforeUpload={() => false}
                            fileList={fileList}
                            onChange={(info) => {
                                setFileList(info.fileList.slice(-1));
                            }}
                            onRemove={() => {
                                setFileList([]);
                            }}
                        >
                            <p className="ant-upload-drag-icon">
                                <FileSearchOutlined />
                            </p>
                            <p className="ant-upload-text">
                                {t("拖拽或点击选择文件", "Drag or click to select a file")}
                            </p>
                            <p className="ant-upload-hint">
                                {t("仅支持 .tar 文件", "Only .tar files are supported")}
                            </p>
                        </Upload.Dragger>
                    </Form.Item>
                    <Form.Item label={t("备注", "Notes")}>
                        <Input.TextArea
                            rows={3}
                            value={notes}
                            placeholder={t("可选备注", "Optional notes")}
                            onChange={(event) => setNotes(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button
                            type="default"
                            onClick={() => {
                                setOpen(false);
                                reset();
                            }}
                        >
                            {t("取消", "Cancel")}
                        </Button>
                        <Button type="primary" loading={submitting} onClick={submit}>
                            {t("上传", "Upload")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function DeployVersionDialog({
    record,
    onSuccess,
}: {
    record: Deploy.Item;
    onSuccess: () => void;
}) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const disabled = record.isExpired;
    const description = t(
        "rz 符号链接只切换一次，随后监控、分析、报表和管理服务依次通过健康检查门禁重启。门禁失败时会恢复原链接，并还原已进入重启流程的服务数据库。",
        "The rz symbolic link switches once, then Monitoring, Insights, Reports, and Admin restart in sequence behind health-check gates. If a gate fails, the original link and databases for services already in the restart flow are restored.",
    );

    const submit = async () => {
        setSubmitting(true);
        try {
            await manageAPI.deploy.deploy(record.id);
            appMessage.success(t("部署任务已提交", "Deployment task submitted"));
            onSuccess();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<CloudUploadOutlined />}
                disabled={disabled}
                onClick={() => setOpen(true)}
                aria-label={t("部署版本", "Deploy version")}
            />
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                onOk={submit}
                okText={t("部署", "Deploy")}
                okButtonProps={{ loading: submitting }}
                cancelText={t("取消", "Cancel")}
                title={
                    <span>
                        {t(
                            `部署 ${componentLabel(record.component)} ${record.version}？`,
                            `Deploy ${componentLabel(record.component)} ${record.version}?`,
                        )}
                    </span>
                }
                centered
            >
                <p>{description}</p>
            </Modal>
        </>
    );
}

function ExpireVersionDialog({
    version,
    onSuccess,
}: {
    version: Deploy.Item;
    onSuccess?: () => void;
}) {
    const [open, setOpen] = useState(false);
    const [notes, setNotes] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const disabled = version.isCurrent || version.isExpired;

    const submit = async () => {
        setSubmitting(true);
        try {
            await manageAPI.deploy.expire(version.id, {
                notes: notes.trim() || null,
            });
            appMessage.success(t("版本已设为过期", "Version expired"));
            onSuccess?.();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<FileDoneOutlined />}
                disabled={disabled}
                onClick={() => setOpen(true)}
                aria-label={t("将版本设为过期", "Expire version")}
            />
            <Modal
                open={open}
                onCancel={() => {
                    setOpen(false);
                    setNotes("");
                }}
                footer={null}
                title={
                    <span>
                        {t(
                            `将 ${componentLabel(version.component)} ${version.version} 设为过期`,
                            `Expire ${componentLabel(version.component)} ${version.version}`,
                        )}
                    </span>
                }
            >
                <Form layout="vertical">
                    <Form.Item label={t("备注", "Notes")}>
                        <Input.TextArea
                            rows={3}
                            value={notes}
                            placeholder={t("可选原因", "Optional reason")}
                            onChange={(event) => setNotes(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button
                            type="default"
                            onClick={() => {
                                setOpen(false);
                                setNotes("");
                            }}
                        >
                            {t("取消", "Cancel")}
                        </Button>
                        <Button type="primary" danger loading={submitting} onClick={submit}>
                            {t("设为过期", "Expire")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function DeleteVersionDialog({
    record,
    onSuccess,
}: {
    record: Deploy.Item;
    onSuccess: () => void;
}) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const disabled = record.isCurrent;

    const submit = async () => {
        setSubmitting(true);
        try {
            await manageAPI.deploy.remove(record.id);
            appMessage.success(t("版本已删除", "Version deleted"));
            onSuccess();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<DeleteOutlined />}
                disabled={disabled}
                danger
                onClick={() => setOpen(true)}
                aria-label={t("删除版本", "Delete version")}
            />
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                onOk={submit}
                okText={t("删除", "Delete")}
                okType="primary"
                okButtonProps={{ loading: submitting, danger: true }}
                cancelText={t("取消", "Cancel")}
                centered
                title={t("删除版本", "Delete version")}
            >
                <p>
                    {t(
                        `确定删除 ${componentLabel(record.component)} ${record.version}？系统会尽可能清理已保存的文件。`,
                        `Delete ${componentLabel(record.component)} ${record.version}? The system will clean up saved files where possible.`,
                    )}
                </p>
            </Modal>
        </>
    );
}

function CleanupDialog({
    component,
    onSuccess,
}: {
    component?: Deploy.Component;
    onSuccess?: () => void;
}) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);

    const submit = async () => {
        setSubmitting(true);
        try {
            const count = await manageAPI.deploy.cleanup(component);
            appMessage.success(
                t(`已清理 ${count} 个过期版本`, `Cleaned ${count} expired versions`),
            );
            onSuccess?.();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button type="default" onClick={() => setOpen(true)}>
                {t("清理过期版本", "Clean expired versions")}
            </Button>
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                onOk={submit}
                okText={t("清理过期版本", "Clean expired versions")}
                okButtonProps={{ loading: submitting, danger: true }}
                cancelText={t("取消", "Cancel")}
                centered
                title={t("清理过期版本？", "Clean expired versions?")}
            >
                <p>
                    {t(
                        "将从列表中移除非当前的过期版本，并尽可能清理已保存的文件。",
                        "Remove non-current expired versions from the list and clean up saved files where possible.",
                    )}
                </p>
            </Modal>
        </>
    );
}

function DeployStatusBadge({ record }: { record: Deploy.Item }) {
    if (record.isCurrent) {
        return <Tag color="blue">{t("当前", "Current")}</Tag>;
    }
    if (record.isExpired) {
        return <Tag color="red">{t("已过期", "Expired")}</Tag>;
    }
    if (record.isDeployed) {
        return <Tag color="green">{t("已部署", "Deployed")}</Tag>;
    }
    return <Tag>{t("已上传", "Uploaded")}</Tag>;
}

function componentLabel(component: Deploy.Component) {
    return component === "release" ? t("发行包", "Release bundle") : component;
}

function formatFileSize(value: number) {
    if (value < 1024 * 1024) {
        return `${(value / 1024).toFixed(1)} KB`;
    }
    return `${(value / 1024 / 1024).toFixed(1)} MB`;
}
