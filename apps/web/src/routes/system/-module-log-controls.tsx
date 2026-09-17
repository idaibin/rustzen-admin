import { DeleteOutlined, DownloadOutlined, ReloadOutlined } from "@ant-design/icons";
import { Button, DatePicker, Select, Space } from "antd";
import dayjs from "dayjs";

import { MODULE_LOG_MODULES, type ModuleLogModule } from "@/api/system/status/module-logs";
import { t } from "@/lib/i18n";

const ALL_MODULES = "all" as const;

interface ModuleLogActionsProps {
    backupPending: boolean;
    onBackup: () => void;
    onPreview: () => void;
    previewPending: boolean;
    selectedCount: number;
}

interface ModuleLogToolbarProps {
    dateFilter: string;
    isFetching: boolean;
    moduleFilter: ModuleLogModule | typeof ALL_MODULES;
    onDateChange: (value: string) => void;
    onModuleChange: (value: ModuleLogModule | typeof ALL_MODULES) => void;
    onRefresh: () => void;
}

export function ModuleLogActions({
    backupPending,
    onBackup,
    onPreview,
    previewPending,
    selectedCount,
}: ModuleLogActionsProps) {
    return (
        <Space wrap>
            <Button
                data-testid="module-log-backup"
                icon={<DownloadOutlined />}
                disabled={!selectedCount || backupPending}
                loading={backupPending}
                onClick={onBackup}
            >
                {t("备份选中文件", "Back up selected")}
            </Button>
            <Button
                data-testid="module-log-cleanup-preview"
                icon={<DeleteOutlined />}
                loading={previewPending}
                onClick={onPreview}
            >
                {t("预览清理", "Preview cleanup")}
            </Button>
        </Space>
    );
}

export function ModuleLogToolbar({
    dateFilter,
    isFetching,
    moduleFilter,
    onDateChange,
    onModuleChange,
    onRefresh,
}: ModuleLogToolbarProps) {
    return (
        <div className="flex flex-wrap items-center gap-3">
            <Select
                aria-label={t("模块筛选", "Module filter")}
                className="w-36"
                value={moduleFilter}
                options={[
                    { label: t("全部模块", "All modules"), value: ALL_MODULES },
                    ...MODULE_LOG_MODULES.map((module) => ({ label: module, value: module })),
                ]}
                onChange={onModuleChange}
            />
            <DatePicker
                aria-label={t("日志日期", "Log date")}
                allowClear={false}
                value={dateFilter ? dayjs(dateFilter) : null}
                onChange={(value) => onDateChange(value ? value.format("YYYY-MM-DD") : "")}
                getPopupContainer={(trigger) => trigger.parentElement ?? document.body}
                className="w-40 max-w-full"
            />
            <Button icon={<ReloadOutlined />} loading={isFetching} onClick={onRefresh}>
                {t("刷新", "Refresh")}
            </Button>
        </div>
    );
}
