import { Alert, Table } from "antd";
import type { ReactNode } from "react";

import type {
    ModuleLogCleanupPreview,
    ModuleLogCleanupResult,
} from "@/api/system/status/module-logs";
import { DataState } from "@/components/feedback/data-state";
import { t } from "@/lib/i18n";

import { CleanupResult, FailureList } from "./-module-log-cleanup-result";
import { formatDateTime, getCleanupCandidateColumns } from "./-module-log-table-utils";

interface ModuleLogCleanupPreviewProps {
    confirmAction: ReactNode;
    error: string | null;
    isExpired: boolean;
    preview: ModuleLogCleanupPreview | null;
    result: ModuleLogCleanupResult | null;
}

export function ModuleLogCleanupPreview({
    confirmAction,
    error,
    isExpired,
    preview,
    result,
}: ModuleLogCleanupPreviewProps) {
    if (error) {
        return (
            <Alert
                type="error"
                showIcon
                message={t("清理预览失败", "Cleanup preview failed")}
                description={error}
            />
        );
    }
    if (preview) {
        return (
            <div className="space-y-3">
                <Alert
                    type={isExpired ? "warning" : "info"}
                    showIcon
                    message={
                        isExpired
                            ? t(
                                  "预览已过期，请重新生成。",
                                  "Preview expired; generate a new preview.",
                              )
                            : t(
                                  `预览有效至 ${formatDateTime(preview.expiresAt)}`,
                                  `Preview expires at ${formatDateTime(preview.expiresAt)}`,
                              )
                    }
                    description={t(
                        `仅处理 ${preview.cutoffDate} 之前的固定模块日志；当前文件不会删除。`,
                        `Only fixed-module logs before ${preview.cutoffDate} are eligible; active files are never deleted.`,
                    )}
                />
                {preview.failures.length ? (
                    <Alert
                        type="warning"
                        showIcon
                        message={t("部分文件无法预览", "Some files could not be previewed")}
                        description={<FailureList failures={preview.failures} />}
                    />
                ) : null}
                {preview.candidates.length ? (
                    <Table
                        data-testid="module-log-cleanup-candidates"
                        rowKey={(record) => `${record.module}:${record.date}`}
                        size="small"
                        pagination={false}
                        dataSource={preview.candidates}
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
                {preview.candidates.length ? confirmAction : null}
            </div>
        );
    }
    if (result) {
        return <CleanupResult result={result} />;
    }
    return null;
}
