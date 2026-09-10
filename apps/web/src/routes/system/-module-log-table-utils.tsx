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

export function formatBytes(bytes: number) {
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

export function formatDateTime(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return "-";
    }
    return date.toLocaleString();
}
