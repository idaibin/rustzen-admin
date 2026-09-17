import { DeleteOutlined } from "@ant-design/icons";
import { useMutation, useQuery } from "@tanstack/react-query";
import { Alert, Button, Typography } from "antd";
import { useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";

import {
    MODULE_LOG_MODULES,
    type ModuleLogCleanupPreview as ModuleLogCleanupPreviewData,
    type ModuleLogCleanupResult,
    type ModuleLogFile,
    type ModuleLogModule,
    moduleLogAPI,
} from "@/api/system/status/module-logs";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { t } from "@/lib/i18n";

import { ModuleLogCleanupPreview } from "./-module-log-cleanup-preview";
import { getModuleLogColumns } from "./-module-log-columns";
import { ModuleLogActions, ModuleLogToolbar } from "./-module-log-controls";
import { ModuleLogFileList } from "./-module-log-file-list";
import { ModuleLogTailDrawer } from "./-module-log-tail-drawer";

const ALL_MODULES = "all" as const;
const EMPTY_FILES: ModuleLogFile[] = [];

export function ModuleLogDiagnostics() {
    const [moduleFilter, setModuleFilter] = useState<ModuleLogModule | typeof ALL_MODULES>(
        ALL_MODULES,
    );
    const [dateFilter, setDateFilter] = useState("");
    const [selectedKeys, setSelectedKeys] = useState<React.Key[]>([]);
    const [selectedFiles, setSelectedFiles] = useState<ModuleLogFile[]>([]);
    const [tailFile, setTailFile] = useState<ModuleLogFile | null>(null);
    const [tailCursor, setTailCursor] = useState<string | undefined>();
    const [tailOpen, setTailOpen] = useState(false);
    const [cleanupPreview, setCleanupPreview] = useState<ModuleLogCleanupPreviewData | null>(null);
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

    const columns = useMemo(
        () =>
            getModuleLogColumns((record) =>
                openTail(record, setTailFile, setTailCursor, setTailOpen),
            ),
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

    return (
        <div data-testid="module-log-panel" className="flex h-full min-h-0 flex-col">
            <PageCard
                title={t("模块日志诊断", "Module log diagnostics")}
                actions={
                    <ModuleLogActions
                        backupPending={backupMutation.isPending}
                        onBackup={() => backupMutation.mutate()}
                        onPreview={() => previewMutation.mutate()}
                        previewPending={previewMutation.isPending}
                        selectedCount={selectedFiles.length}
                    />
                }
                toolbar={
                    <ModuleLogToolbar
                        dateFilter={dateFilter}
                        isFetching={fileQuery.isFetching}
                        moduleFilter={moduleFilter}
                        onDateChange={setDateFilter}
                        onModuleChange={setModuleFilter}
                        onRefresh={() => void fileQuery.refetch()}
                    />
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
                <ModuleLogCleanupPreviewSection
                    confirmAction={
                        cleanupPreview?.candidates.length && !previewError ? (
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
                        ) : null
                    }
                    error={previewError}
                    isExpired={cleanupExpired}
                    isPending={previewMutation.isPending}
                    preview={cleanupPreview}
                    result={cleanupResult}
                />
                <ModuleLogFileList
                    columns={columns}
                    error={metadataError}
                    files={files}
                    isFetching={fileQuery.isFetching}
                    isPending={fileQuery.isPending}
                    onReload={() => void fileQuery.refetch()}
                    onSelectionChange={(keys, rows) => {
                        setSelectedKeys(keys);
                        setSelectedFiles(rows);
                    }}
                    selectedKeys={selectedKeys}
                />
                {openTailButton}
            </PageCard>
        </div>
    );
}

interface ModuleLogCleanupPreviewSectionProps {
    confirmAction: ReactNode;
    error: string | null;
    isExpired: boolean;
    isPending: boolean;
    preview: ModuleLogCleanupPreviewData | null;
    result: ModuleLogCleanupResult | null;
}

export function ModuleLogCleanupPreviewSection({
    confirmAction,
    error,
    isExpired,
    isPending,
    preview,
    result,
}: ModuleLogCleanupPreviewSectionProps) {
    return (
        <>
            {isPending ? (
                <DataState
                    kind="processing"
                    title={t("正在生成清理预览", "Preparing cleanup preview")}
                    compact
                />
            ) : null}
            <ModuleLogCleanupPreview
                confirmAction={error ? null : confirmAction}
                error={error}
                isExpired={isExpired}
                preview={preview}
                result={result}
            />
        </>
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
