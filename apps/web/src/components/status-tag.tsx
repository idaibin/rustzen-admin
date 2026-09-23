import { Tag } from "antd";

import { t } from "@/lib/i18n";

export interface StatusTagMeta {
    label: string;
    color: string;
}

export function StatusTag({
    status,
    meta,
}: {
    status: number | string;
    meta: Readonly<Record<number | string, StatusTagMeta>>;
}) {
    const item = meta[status as keyof typeof meta] ?? {
        label: t("未知", "Unknown"),
        color: "default",
    };
    return <Tag color={item.color}>{item.label}</Tag>;
}
