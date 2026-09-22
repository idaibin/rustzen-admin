import { EyeOutlined } from "@ant-design/icons";
import type { ProColumns } from "@ant-design/pro-components";
import { Button, Space, Tag, Typography } from "antd";

import type { ModuleLogFile } from "@/api/system/status/module-logs";
import { actionColumnWidth } from "@/components/table/action-column";
import { formatBytes } from "@/lib/format";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export function getModuleLogColumns(
    onOpen: (file: ModuleLogFile) => void,
): ProColumns<ModuleLogFile>[] {
    return [
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
                <Space size="small" className="min-w-0">
                    <Typography.Text ellipsis={{ tooltip: record.fileName }} className="min-w-0">
                        {record.fileName}
                    </Typography.Text>
                    <Typography.Text type="secondary" className="shrink-0">
                        {record.date}
                    </Typography.Text>
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
            width: actionColumnWidth(1),
            fixed: "right",
            render: (_value, record) => (
                <Button
                    data-testid={`module-log-tail-${record.module}-${record.date}`}
                    type="text"
                    icon={<EyeOutlined />}
                    aria-label={t("查看日志", "View log")}
                    disabled={!record.readable}
                    onClick={() => onOpen(record)}
                />
            ),
        },
    ];
}
