import {
    DeleteOutlined,
    DownloadOutlined,
    EyeOutlined,
    ReloadOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Alert, Button, Input, Select, Space, Table, Tag, Typography } from "antd";
import { useEffect, useMemo, useState } from "react";

import {
    MODULE_LOG_MODULES,
    type ModuleLogCleanupPreview,
    type ModuleLogCleanupResult,
    type ModuleLogFile,
    type ModuleLogModule,
    moduleLogAPI,
} from "@/api/system/status/module-logs";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { ModuleLogTailDrawer } from "./-module-log-tail-drawer";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

const ALL_MODULES = "all" as const;
const EMPTY_FILES: ModuleLogFile[] = [];

export function ModuleLogDiagnostics() {
    const canView = useAuthStore((state) => state.checkPermissions("system:module:log:view"));

    // The System Status route is owner-only. Keep this boundary silent for every other user.
    if (!canView) {
        return null;
    }

    return <ModuleLogDiagnosticsContent />;
}

function ModuleLogDiagnosticsContent() {
    const [moduleFilter, setModuleFilter] = useState<ModuleLogModule | typeof ALL_MODULES>(
        ALL_MODULES,
    );
    const [dateFilter, setDateFilter] = useState("");
    const [selectedKeys, setSelectedKeys] = useState<React.Key[]>([]);
    const [selectedFiles, setSelectedFiles] = useState<ModuleLogFile[]>([]);
    const [tailFile, setTailFile] = useState<ModuleLogFile | null>(null);
    const [tailCursor, setTailCursor] = useState<string | undefined>();
    const [tailOpen, setTailOpen] = useState(false);
    const [cleanupPreview, setCleanupPreview] = useState<ModuleLogCleanupPreview | null>(null);
    const [cleanupResult, setCleanupResult] = useState<ModuleLogCleanupResult | null>(null);
    const [cleanupClock, setCleanupClock] = useState(() => Date.now());

    const listParams = useMemo(
        () => ({
            module: moduleFilter === ALL_MODULES ? undefined : moduleFilter,
            date: dateFilter || undefined,
        }),
        [dateFilter, moduleFilter],
    );
    const fileQuery = useQuery({
        queryKey: ["system", "moduleLogs", listParams],
        queryFn: () => moduleLogAPI.list(listParams),
    });
    const files = fileQuery.data ?? EMPTY_FILES;

    const tailQuery = useQuery({
        queryKey: ["system", "moduleLogs", "tail", tailFile?.module, tailFile?.date, tailCursor],
        queryFn: () =>
            moduleLogAPI.tail(
                { module: toModule(tailFile?.module), date: tailFile?.date ?? "" },
                tailCursor,
            ),
        enabled: tailOpen && tailFile !== null,
    });

    const backupMutation = useMutation({
        mutationFn: () =>
            moduleLogAPI.backup(
                selectedFiles.map((file) => ({ module: toModule(file.module), date: file.date })),
            ),
    });
    const previewMutation = useMutation({
        mutationFn: moduleLogAPI.previewCleanup,
        onSuccess: (preview) => {
            setCleanupPreview(preview);
            setCleanupResult(null);
            setCleanupClock(Date.now());
        },
    });
    const confirmMutation = useMutation({
        mutationFn: (token: string) => moduleLogAPI.confirmCleanup(token),
        onSuccess: (result) => {
            setCleanupResult(result);
            setCleanupPreview(null);
            void fileQuery.refetch();
        },
    });

    useEffect(() => {
        setSelectedKeys([]);
        setSelectedFiles([]);
    }, [dateFilter, moduleFilter]);

    useEffect(() => {
        if (!cleanupPreview) {
            return;
        }
        const timer = window.setInterval(() => setCleanupClock(Date.now()), 1000);
        return () => window.clearInterval(timer);
    }, [cleanupPreview]);

    const cleanupExpired = cleanupPreview
        ? Date.parse(cleanupPreview.expiresAt) <= cleanupClock
        : false;
    const metadataError = fileQuery.error ? errorMessage(fileQuery.error) : null;
    const tailError = tailQuery.error ? errorMessage(tailQuery.error) : null;
    const backupError = backupMutation.error ? errorMessage(backupMutation.error) : null;
    const previewError = previewMutation.error ? errorMessage(previewMutation.error) : null;
    const confirmError = confirmMutation.error ? errorMessage(confirmMutation.error) : null;

    const columns = useMemo<ProColumns<ModuleLogFile>[]>(
        () => [
            {
                title: t("模块", "Module"),
                dataIndex: "module",
                key: "module",
                width: 112,
                render: (_value, record) => <Typography.Text code>{record.module}</Typography.Text>,
            },
            {
                title: t("文件 / 日期", "File / date"),
                key: "file",
                ellipsis: true,
                render: (_value, record) => (
                    <Space direction="vertical" size={0}>
                        <Typography.Text ellipsis={{ tooltip: record.fileName }}>
                            {record.fileName}
                        </Typography.Text>
                        <Typography.Text type="secondary">{record.date}</Typography.Text>
                    </Space>
                ),
            },
            {
                title: t("大小", "Size"),
                key: "size",
                width: 110,
                render: (_value, record) => formatBytes(record.sizeBytes),
            },
            {
                title: t("状态", "Status"),
                key: "status",
                width: 150,
                render: (_value, record) => (
                    <Space size="small" wrap>
                        <Tag color={record.readable ? "green" : "red"}>
                            {record.readable ? t("可读", "Readable") : t("不可读", "Unreadable")}
                        </Tag>
                        {record.active ? <Tag color="orange">{t("当前文件", "Active")}</Tag> : null}
                    </Space>
                ),
            },
            {
                title: t("修改时间", "Modified"),
                key: "modifiedAt",
                width: 180,
                render: (_value, record) => formatDateTime(record.modifiedAt),
            },
            {
                title: t("操作", "Actions"),
                key: "actions",
                width: 116,
                fixed: "right",
                render: (_value, record) => (
                    <Button
                        data-testid={`module-log-tail-${record.module}-${record.date}`}
                        type="link"
                        size="small"
                        icon={<EyeOutlined />}
                        disabled={!record.readable}
                        onClick={() => openTail(record, setTailFile, setTailCursor, setTailOpen)}
                    >
                        {t("查看", "View")}
                    </Button>
                ),
            },
        ],
        [],
    );

    const openTailButton = (
        <ModuleLogTailDrawer
            error={tailError}
            file={tailFile}
            onClose={() => setTailOpen(false)}
            onLoadOlder={(cursor) => setTailCursor(cursor)}
            open={tailOpen}
            query={tailQuery}
        />
    );

    const renderCleanupPreview = () => {
        if (previewError) {
            return (
                <Alert
                    type="error"
                    showIcon
                    message={t("清理预览失败", "Cleanup preview failed")}
                    description={previewError}
                />
            );
        }
        if (cleanupPreview) {
            return (
                <div className="space-y-3">
                    <Alert
                        type={cleanupExpired ? "warning" : "info"}
                        showIcon
                        message={
                            cleanupExpired
                                ? t(
                                      "预览已过期，请重新生成。",
                                      "Preview expired; generate a new preview.",
                                  )
                                : t(
                                      `预览有效至 ${formatDateTime(cleanupPreview.expiresAt)}`,
                                      `Preview expires at ${formatDateTime(cleanupPreview.expiresAt)}`,
                                  )
                        }
                        description={t(
                            `仅处理 ${cleanupPreview.cutoffDate} 之前的固定模块日志；当前文件不会删除。`,
                            `Only fixed-module logs before ${cleanupPreview.cutoffDate} are eligible; active files are never deleted.`,
                        )}
                    />
                    {cleanupPreview.failures.length ? (
                        <FailureAlert
                            title={t("部分文件无法预览", "Some files could not be previewed")}
                            failures={cleanupPreview.failures}
                        />
                    ) : null}
                    {cleanupPreview.candidates.length ? (
                        <Table
                            data-testid="module-log-cleanup-candidates"
                            rowKey={(record) => `${record.module}:${record.date}`}
                            size="small"
                            pagination={false}
                            dataSource={cleanupPreview.candidates}
                            columns={getCleanupCandidateColumns()}
                            scroll={{ x: 520 }}
                        />
                    ) : (
                        <DataState
                            kind="empty"
                            title={t("没有符合条件的日志", "No eligible logs")}
                            description={t(
                                "当前保留策略下没有可清理文件。",
                                "No files match the current retention cutoff.",
                            )}
                            compact
                        />
                    )}
                    {cleanupPreview.candidates.length ? (
                        <ConfirmDialog
                            disabled={cleanupExpired || confirmMutation.isPending}
                            destructive
                            trigger={
                                <Button
                                    data-testid="module-log-cleanup-confirm-trigger"
                                    danger
                                    icon={<DeleteOutlined />}
                                    disabled={cleanupExpired || confirmMutation.isPending}
                                    loading={confirmMutation.isPending}
                                >
                                    {t("确认清理", "Confirm cleanup")}
                                </Button>
                            }
                            title={t("确认删除过期模块日志？", "Delete expired module logs?")}
                            description={t(
                                "确认后将重新校验文件安全性和预览快照。已变化、当前或不安全的文件会保留并列出失败原因。",
                                "Files are rechecked against the preview. Changed, active, or unsafe files are retained and reported.",
                            )}
                            confirmLabel={t("删除并记录结果", "Delete and record result")}
                            confirmTestId="module-log-cleanup-confirm"
                            onConfirm={async () => {
                                if (cleanupExpired) {
                                    throw new Error(
                                        t(
                                            "预览已过期，请重新生成。",
                                            "The preview expired; generate a new one.",
                                        ),
                                    );
                                }
                                await confirmMutation.mutateAsync(cleanupPreview.token);
                            }}
                        />
                    ) : null}
                </div>
            );
        }
        if (cleanupResult) {
            return <CleanupResult result={cleanupResult} />;
        }
        return null;
    };

    return (
        <div data-testid="module-log-panel">
            <PageCard
            headingLevel={2}
            className="shrink-0"
            title={t("模块日志诊断", "Module log diagnostics")}
            actions={
                <Space wrap>
                    <Button
                        data-testid="module-log-backup"
                        icon={<DownloadOutlined />}
                        disabled={!selectedFiles.length || backupMutation.isPending}
                        loading={backupMutation.isPending}
                        onClick={() => backupMutation.mutate()}
                    >
                        {t("备份选中文件", "Back up selected")}
                    </Button>
                    <Button
                        data-testid="module-log-cleanup-preview"
                        icon={<DeleteOutlined />}
                        loading={previewMutation.isPending}
                        onClick={() => previewMutation.mutate()}
                    >
                        {t("预览清理", "Preview cleanup")}
                    </Button>
                </Space>
            }
            toolbar={
                <div className="flex flex-wrap items-center gap-3">
                    <Select
                        aria-label={t("模块筛选", "Module filter")}
                        className="w-36"
                        value={moduleFilter}
                        options={[
                            { label: t("全部模块", "All modules"), value: ALL_MODULES },
                            ...MODULE_LOG_MODULES.map((module) => ({
                                label: module,
                                value: module,
                            })),
                        ]}
                        onChange={(value: ModuleLogModule | typeof ALL_MODULES) =>
                            setModuleFilter(value)
                        }
                    />
                    <Input
                        aria-label={t("日志日期", "Log date")}
                        type="date"
                        value={dateFilter}
                        onChange={(event) => setDateFilter(event.target.value)}
                        style={{ width: 160, maxWidth: "100%" }}
                    />
                    <Button
                        icon={<ReloadOutlined />}
                        loading={fileQuery.isFetching}
                        onClick={() => void fileQuery.refetch()}
                    >
                        {t("刷新", "Refresh")}
                    </Button>
                </div>
            }
        >
            <Typography.Text type="secondary">
                {t(
                    "查看四个本地服务的受限日志尾部，并通过预览安全备份或清理过期文件。操作日志仍保留在独立的日志页面。",
                    "Inspect bounded tails from the four local services, then preview safe backups or cleanup. Operation logs remain on their separate page.",
                )}
            </Typography.Text>
            <Typography.Text type="secondary">
                {t(
                    "备份上限 64 MiB；日志尾部最多 256 KiB / 2,000 行。",
                    "Backup cap: 64 MiB; tail cap: 256 KiB / 2,000 lines.",
                )}
            </Typography.Text>
            {backupError ? (
                <Alert
                    type="error"
                    showIcon
                    message={t("日志备份失败", "Log backup failed")}
                    description={backupError}
                />
            ) : backupMutation.data ? (
                <Alert
                    data-testid="module-log-backup-summary"
                    type="success"
                    showIcon
                    message={t("日志备份已下载", "Log backup downloaded")}
                    description={t(
                        `${backupMutation.data.filename}：${backupMutation.data.fileCount} 个文件，SHA-256 ${backupMutation.data.archiveSha256.slice(0, 12)}…。`,
                        `${backupMutation.data.filename}: ${backupMutation.data.fileCount} files, SHA-256 ${backupMutation.data.archiveSha256.slice(0, 12)}….`,
                    )}
                />
            ) : null}
            {confirmError ? (
                <Alert
                    type="warning"
                    showIcon
                    message={t("清理确认未完成", "Cleanup confirmation was not completed")}
                    description={confirmError}
                />
            ) : null}
            {previewMutation.isPending ? (
                <DataState
                    kind="processing"
                    title={t("正在生成清理预览", "Preparing cleanup preview")}
                    compact
                />
            ) : null}
            {renderCleanupPreview()}
            {fileQuery.error && files.length ? (
                <Alert
                    type="warning"
                    showIcon
                    message={t(
                        "日志列表刷新失败，仍显示上次结果",
                        "Refresh failed; showing the last result",
                    )}
                    description={metadataError ?? undefined}
                />
            ) : null}
            {!files.length && fileQuery.isPending ? (
                <DataState kind="loading" title={t("正在加载模块日志", "Loading module logs")} />
            ) : !files.length && fileQuery.error ? (
                <DataState
                    kind="error"
                    title={t("模块日志加载失败", "Failed to load module logs")}
                    description={metadataError ?? undefined}
                    action={
                        <Button type="primary" onClick={() => void fileQuery.refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            ) : !files.length ? (
                <DataState
                    kind="empty"
                    title={t("暂无模块日志文件", "No module log files")}
                    description={t(
                        "仅展示 admin、monitor、insights、reports 四个固定模块的日志文件。",
                        "Only the fixed admin, monitor, insights, and reports modules are shown.",
                    )}
                />
            ) : (
                <DataTableShell ariaLabel={t("模块日志文件", "Module log files table")}>
                    <ProTable<ModuleLogFile>
                        rowKey={(record) => `${record.module}:${record.date}`}
                        columns={columns}
                        dataSource={files}
                        loading={fileQuery.isFetching}
                        search={false}
                        options={false}
                        pagination={false}
                        rowSelection={{
                            selectedRowKeys: selectedKeys,
                            renderCell: (_checked, record, _index, originNode) => (
                                <span data-testid={`module-log-select-${record.module}-${record.date}`}>
                                    {originNode}
                                </span>
                            ),
                            onChange: (keys, rows) => {
                                setSelectedKeys(keys);
                                setSelectedFiles(rows);
                            },
                            getCheckboxProps: (record) => ({ disabled: !record.readable }),
                        }}
                        locale={{
                            emptyText: <DataState kind="empty" title={t("暂无日志", "No logs")} />,
                        }}
                        toolBarRender={false}
                        tableAlertOptionRender={false}
                    />
                </DataTableShell>
            )}
            {openTailButton}
            </PageCard>
        </div>
    );
}

function getCleanupCandidateColumns() {
    return [
        {
            title: t("模块", "Module"),
            dataIndex: "module",
            key: "module",
            width: 100,
        },
        {
            title: t("文件", "File"),
            dataIndex: "fileName",
            key: "fileName",
            ellipsis: true,
        },
        {
            title: t("日期", "Date"),
            dataIndex: "date",
            key: "date",
            width: 112,
        },
        {
            title: t("大小", "Size"),
            dataIndex: "sizeBytes",
            key: "sizeBytes",
            width: 100,
            render: (value: number) => formatBytes(value),
        },
    ];
}

function CleanupResult({ result }: { result: ModuleLogCleanupResult }) {
    const resultType = result.partial ? "warning" : "success";
    return (
        <Alert
            data-testid="module-log-cleanup-result"
            type={resultType}
            showIcon
            message={
                result.partial
                    ? t("清理完成，但需要复核部分结果", "Cleanup completed with items to review")
                    : t("清理完成", "Cleanup completed")
            }
            description={
                <Space direction="vertical" size="small">
                    <Typography.Text>
                        {t(
                            `已删除 ${result.removed.length} 个文件，保留 ${result.retained.length} 个文件，失败 ${result.failures.length} 个文件。`,
                            `Removed ${result.removed.length}, retained ${result.retained.length}, failed ${result.failures.length}.`,
                        )}
                    </Typography.Text>
                    {result.failures.length ? <FailureList failures={result.failures} /> : null}
                </Space>
            }
        />
    );
}

function FailureAlert({
    title,
    failures,
}: {
    title: string;
    failures: { module: string; fileName: string; reason: string }[];
}) {
    return (
        <Alert
            type="warning"
            showIcon
            message={title}
            description={<FailureList failures={failures} />}
        />
    );
}

function FailureList({
    failures,
}: {
    failures: { module: string; fileName: string; reason: string }[];
}) {
    return (
        <ul className="m-0 list-disc space-y-1 pl-5">
            {failures.map((failure) => (
                <li key={`${failure.module}:${failure.fileName}:${failure.reason}`}>
                    <Typography.Text>
                        {failure.module} / {failure.fileName}: {failure.reason}
                    </Typography.Text>
                </li>
            ))}
        </ul>
    );
}

function openTail(
    record: ModuleLogFile,
    setTailFile: (file: ModuleLogFile) => void,
    setTailCursor: (cursor: string | undefined) => void,
    setTailOpen: (open: boolean) => void,
) {
    setTailFile(record);
    setTailCursor(undefined);
    setTailOpen(true);
}

function toModule(module: string | undefined): ModuleLogModule {
    if (module && (MODULE_LOG_MODULES as readonly string[]).includes(module)) {
        return module as ModuleLogModule;
    }
    throw new Error(`Unsupported module log module: ${module ?? ""}`);
}

function errorMessage(error: unknown) {
    return error instanceof Error ? error.message : t("请稍后重试。", "Please try again later.");
}

function formatBytes(bytes: number) {
    if (!bytes) {
        return "0 B";
    }

    const units = ["B", "KB", "MB", "GB", "TB"] as const;
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
        value /= 1024;
        unitIndex += 1;
    }
    const precision = unitIndex === 0 ? 0 : 1;
    return `${Number(value.toFixed(precision))} ${units[unitIndex]}`;
}

function formatDateTime(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return "-";
    }
    return date.toLocaleString();
}
