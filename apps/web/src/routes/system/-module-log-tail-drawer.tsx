import { ClockCircleOutlined, FileSearchOutlined } from "@ant-design/icons";
import type { UseQueryResult } from "@tanstack/react-query";
import { Alert, Button, Card, Drawer, Space, Tag, Typography } from "antd";

import type { ModuleLogFile, ModuleLogTail } from "@/api/system/status/module-logs";
import { DataState } from "@/components/feedback/data-state";
import { t } from "@/lib/i18n";

interface Props {
    error: string | null;
    file: ModuleLogFile | null;
    onClose: () => void;
    onLoadOlder: (cursor: string) => void;
    open: boolean;
    query: UseQueryResult<ModuleLogTail>;
}

export function ModuleLogTailDrawer({ error, file, onClose, onLoadOlder, open, query }: Props) {
    if (!file) {
        return null;
    }

    const data = query.data;
    return (
        <Drawer
            data-testid="module-log-tail-drawer"
            title={<DrawerTitle file={file} />}
            open={open}
            width={720}
            onClose={onClose}
            destroyOnHidden
        >
            {error ? (
                <DataState
                    kind="error"
                    title={t("模块日志读取失败", "Failed to read module log")}
                    description={error}
                    action={
                        <Button type="primary" onClick={() => void query.refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                    compact
                />
            ) : query.isPending ? (
                <DataState
                    kind="loading"
                    title={t("正在读取日志尾部", "Loading log tail")}
                    compact
                />
            ) : data ? (
                <TailContent data={data} isFetching={query.isFetching} onLoadOlder={onLoadOlder} />
            ) : (
                <DataState
                    kind="empty"
                    title={t("未找到日志内容", "No log content found")}
                    compact
                />
            )}
        </Drawer>
    );
}

function DrawerTitle({ file }: { file: ModuleLogFile }) {
    return (
        <Space>
            <FileSearchOutlined />
            <span>
                {file.module} / {file.date}
            </span>
        </Space>
    );
}

function TailContent({
    data,
    isFetching,
    onLoadOlder,
}: {
    data: ModuleLogTail;
    isFetching: boolean;
    onLoadOlder: (cursor: string) => void;
}) {
    const cursor = data.nextCursor;
    return (
        <div className="flex h-full flex-col gap-4">
            <Space wrap>
                <Tag>{data.module}</Tag>
                <Tag>{data.date}</Tag>
                <Typography.Text type="secondary">
                    {t(
                        `${data.lineCount} 行 / ${formatBytes(data.byteCount)}`,
                        `${data.lineCount} lines / ${formatBytes(data.byteCount)}`,
                    )}
                </Typography.Text>
            </Space>
            {data.truncated ? (
                <Alert
                    type="warning"
                    showIcon
                    message={t(
                        "内容已按安全上限截断。可继续读取更早内容。",
                        "Content was truncated at the safety limit. Load older content to continue.",
                    )}
                />
            ) : null}
            {data.content ? (
                <Card
                    data-testid="module-log-tail-content"
                    className="flex min-h-0 flex-1 flex-col"
                    size="small"
                    title={t("受限日志尾部", "Bounded log tail")}
                    styles={{ body: { display: "flex", minHeight: 0, flex: 1, padding: 12 } }}
                >
                    <pre className="m-0 min-h-0 flex-1 overflow-auto whitespace-pre-wrap break-words text-xs leading-5">
                        {data.content}
                    </pre>
                </Card>
            ) : (
                <DataState kind="empty" title={t("日志文件为空", "Log file is empty")} compact />
            )}
            {cursor ? (
                <Button
                    icon={<ClockCircleOutlined />}
                    loading={isFetching}
                    onClick={() => onLoadOlder(cursor)}
                >
                    {t("读取更早内容", "Load older content")}
                </Button>
            ) : null}
        </div>
    );
}

function formatBytes(bytes: number) {
    if (!bytes) return "0 B";
    const units = ["B", "KB", "MB", "GB", "TB"] as const;
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
        value /= 1024;
        unitIndex += 1;
    }
    return `${Number(value.toFixed(unitIndex === 0 ? 0 : 1))} ${units[unitIndex]}`;
}
