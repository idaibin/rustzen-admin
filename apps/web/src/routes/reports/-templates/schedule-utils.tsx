import { Tag } from "antd";

import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export function toSaveSchedule(schedule: Reports.Schedule): Reports.SaveSchedule {
    return {
        flowId: schedule.flowId,
        cadence: schedule.cadence,
        weekday: schedule.weekday ?? undefined,
        dueTime: schedule.dueTime,
        input: schedule.input,
        description: schedule.description,
        enabled: schedule.enabled,
    };
}

export function formatScheduleCadence(schedule: Reports.Schedule): string {
    if (schedule.cadence === "daily") return t("每日", "Daily");
    const days = [
        t("周一", "Monday"),
        t("周二", "Tuesday"),
        t("周三", "Wednesday"),
        t("周四", "Thursday"),
        t("周五", "Friday"),
        t("周六", "Saturday"),
        t("周日", "Sunday"),
    ];
    return `${t("每周", "Weekly")} ${days[schedule.weekday ?? 0]}`;
}

export function formatScheduleOccurrenceDue(
    occurrence: Pick<Reports.ScheduleOccurrence, "dueAt" | "dueLocal">,
    timezone: string,
): string {
    return occurrence.dueAt
        ? formatDateTime(occurrence.dueAt, timezone)
        : `${occurrence.dueLocal} · ${timezone}`;
}

export function ScheduleDecisionTag({ decision }: { decision: Reports.ScheduleDecision }) {
    return (
        <Tag color={decision === "enqueued" ? "success" : "warning"}>
            {decision === "enqueued" ? t("已入队", "Enqueued") : t("已跳过", "Skipped")}
        </Tag>
    );
}
