import type { StatusTagMeta } from "@/components/status-tag";
import { t } from "@/lib/i18n";

export const getEnableOptions = () => [
    { label: t("启用", "Enabled"), value: 1 },
    { label: t("禁用", "Disabled"), value: 2 },
];

export const getEnableStatusMeta = (): Record<number, StatusTagMeta> => ({
    1: { label: t("启用", "Enabled"), color: "success" },
    2: { label: t("禁用", "Disabled"), color: "default" },
});

export const getUserStatusMeta = (): Record<number, StatusTagMeta> => ({
    1: { label: t("启用", "Enabled"), color: "success" },
    2: { label: t("禁用", "Disabled"), color: "default" },
    3: { label: t("待审核", "Pending"), color: "warning" },
    4: { label: t("已锁定", "Locked"), color: "error" },
});

export const getMenuTypeOptions = () => [
    { label: t("目录", "Directory"), value: 1 },
    { label: t("菜单", "Menu"), value: 2 },
    { label: t("按钮", "Button"), value: 3 },
];

export const getMenuTypeMeta = (): Record<number, StatusTagMeta> => ({
    1: { label: t("目录", "Directory"), color: "default" },
    2: { label: t("菜单", "Menu"), color: "blue" },
    3: { label: t("按钮", "Button"), color: "green" },
});

export const getModuleIconOptions = () => [
    { label: t("监控", "Monitoring"), value: "monitor" },
    { label: t("分析", "Insights"), value: "chart-no-axes-combined" },
    { label: t("报表", "Reports"), value: "file-text" },
];
