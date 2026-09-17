import { ProTable } from "@ant-design/pro-components";
import { Alert, Button } from "antd";
import type { Key } from "react";

import type { ModuleLogFile } from "@/api/system/status/module-logs";
import { DataState } from "@/components/feedback/data-state";
import { DataTableShell } from "@/components/table/data-table-shell";
import { t } from "@/lib/i18n";

import type { getModuleLogColumns } from "./-module-log-columns";

interface ModuleLogFileListProps {
    columns: ReturnType<typeof getModuleLogColumns>;
    error: string | null;
    files: ModuleLogFile[];
    isFetching: boolean;
    isPending: boolean;
    onReload: () => void;
    onSelectionChange: (keys: Key[], rows: ModuleLogFile[]) => void;
    selectedKeys: Key[];
}

export function ModuleLogFileList({
    columns,
    error,
    files,
    isFetching,
    isPending,
    onReload,
    onSelectionChange,
    selectedKeys,
}: ModuleLogFileListProps) {
    return (
        <>
            {error !== null && files.length ? (
                <Alert
                    type="warning"
                    showIcon
                    message={t(
                        "日志列表刷新失败，仍显示上次结果",
                        "Refresh failed; showing the last result",
                    )}
                    description={error}
                />
            ) : null}
            {!files.length && isPending ? (
                <DataState kind="loading" title={t("正在加载模块日志", "Loading module logs")} />
            ) : !files.length && error !== null ? (
                <DataState
                    kind="error"
                    title={t("模块日志加载失败", "Failed to load module logs")}
                    description={error}
                    action={
                        <Button type="primary" onClick={onReload}>
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
                        loading={isFetching}
                        search={false}
                        options={false}
                        pagination={false}
                        rowSelection={{
                            selectedRowKeys: selectedKeys,
                            renderCell: (_checked, record, _index, originNode) => (
                                <span
                                    data-testid={`module-log-select-${record.module}-${record.date}`}
                                >
                                    {originNode}
                                </span>
                            ),
                            onChange: onSelectionChange,
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
        </>
    );
}
