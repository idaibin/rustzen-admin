import { t } from "@/lib/i18n";

export const getRunStatusMeta = () =>
    ({
        queued: { label: t("排队中", "Queued"), color: "default" },
        running: { label: t("执行中", "Running"), color: "processing" },
        cancelling: { label: t("取消中", "Cancelling"), color: "warning" },
        succeeded: { label: t("已成功", "Succeeded"), color: "success" },
        failed: { label: t("失败", "Failed"), color: "error" },
        cancelled: { label: t("已取消", "Cancelled"), color: "warning" },
    }) satisfies Record<Reports.Run["status"], { label: string; color: string }>;

export const getStepStatusMeta = () =>
    ({
        running: { label: t("执行中", "Running"), color: "processing" },
        succeeded: { label: t("已成功", "Succeeded"), color: "success" },
        failed: { label: t("失败", "Failed"), color: "error" },
        cancelled: { label: t("已取消", "Cancelled"), color: "warning" },
    }) satisfies Record<Reports.RunStep["status"], { label: string; color: string }>;

export const isActiveRun = (status?: Reports.Run["status"]) =>
    status === "queued" || status === "running" || status === "cancelling";
