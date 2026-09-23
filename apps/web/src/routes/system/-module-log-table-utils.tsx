import { formatBytes } from "@/lib/format";
import { t } from "@/lib/i18n";

export function getCleanupCandidateColumns() {
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
