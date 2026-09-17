import { type ProColumns } from "@ant-design/pro-components";
import { Button, Space, Tag } from "antd";

import { appMessage, reportsAPI } from "@/api";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { ScheduleDialog } from "./schedule-dialog";
import { ScheduleToggle } from "./schedule-toggle";
import {
    formatScheduleCadence,
    formatScheduleOccurrenceDue,
    ScheduleDecisionTag,
} from "./schedule-utils";

type ScheduleColumnsOptions = {
    flowOptions: Reports.FlowOption[];
    canManageSchedules: boolean;
    onSaved: () => Promise<unknown>;
};

export function createScheduleColumns({
    flowOptions,
    canManageSchedules,
    onSaved,
}: ScheduleColumnsOptions): ProColumns<Reports.Schedule>[] {
    const columns: ProColumns<Reports.Schedule>[] = [
        {
            title: t("流程", "Template"),
            key: "flow",
            ellipsis: true,
            render: (_value: unknown, row: Reports.Schedule) => (
                <div>
                    <div className="font-medium">
                        {flowOptions.find((flow) => flow.id === row.flowId)?.name ?? row.flowId}
                    </div>
                    {row.description ? (
                        <div className="text-xs text-muted-foreground">{row.description}</div>
                    ) : null}
                </div>
            ),
        },
        {
            title: t("计划", "Schedule"),
            key: "cadence",
            width: 190,
            render: (_value: unknown, row: Reports.Schedule) => (
                <div>
                    <div>{formatScheduleCadence(row)}</div>
                    <div
                        className="text-xs text-muted-foreground"
                        data-testid={`schedule-timezone-${row.id}`}
                    >
                        {row.dueTime} · {row.timezone}
                    </div>
                </div>
            ),
        },
        {
            title: t("状态", "Status"),
            key: "enabled",
            width: 130,
            render: (_value: unknown, row: Reports.Schedule) => (
                <Tag color={row.enabled ? "success" : "default"}>
                    {row.enabled ? t("已启用", "Enabled") : t("已停用", "Disabled")}
                </Tag>
            ),
        },
        {
            title: t("下次执行", "Next due"),
            key: "nextDue",
            width: 190,
            render: (_value: unknown, row: Reports.Schedule) =>
                formatDateTime(row.nextDue, row.timezone),
        },
        {
            title: t("最近结果", "Last outcome"),
            key: "lastOccurrence",
            width: 220,
            render: (_value: unknown, row: Reports.Schedule) => {
                const occurrence = row.lastOccurrence;
                if (!occurrence) return "-";
                return (
                    <div data-testid={`schedule-occurrence-${row.id}`}>
                        <ScheduleDecisionTag decision={occurrence.decision} />
                        {occurrence.decision === "skipped" ? (
                            <div
                                className="text-xs text-muted-foreground"
                                data-testid={`schedule-occurrence-due-${row.id}`}
                                data-due-at={occurrence.dueAt ?? undefined}
                                data-due-local={occurrence.dueLocal}
                            >
                                {formatScheduleOccurrenceDue(occurrence, row.timezone)}
                            </div>
                        ) : null}
                        <div
                            className="text-xs text-muted-foreground"
                            data-testid={
                                occurrence.decision === "skipped"
                                    ? `schedule-occurrence-skipped-${row.id}`
                                    : undefined
                            }
                        >
                            {occurrence.reason ||
                                formatDateTime(occurrence.decidedAt, row.timezone)}
                        </div>
                        {occurrence.runId ? (
                            <Button
                                data-testid={`schedule-occurrence-run-${row.id}`}
                                type="link"
                                className="h-auto p-0 text-xs"
                                href={`/reports/runs?runId=${encodeURIComponent(occurrence.runId)}`}
                            >
                                {t("查看执行", "View run")}
                            </Button>
                        ) : null}
                    </div>
                );
            },
        },
    ];

    if (canManageSchedules) {
        columns.push({
            title: <span data-testid="schedule-actions-column">{t("操作", "Actions")}</span>,
            key: "actions",
            width: 230,
            fixed: "right",
            render: (_value: unknown, row: Reports.Schedule) => (
                <Space size="small">
                    <ScheduleDialog flowOptions={flowOptions} schedule={row} onSaved={onSaved} />
                    <ScheduleToggle schedule={row} onSaved={onSaved} />
                    <ConfirmDialog
                        trigger={
                            <Button data-testid="schedule-delete" type="link" danger>
                                {t("删除", "Delete")}
                            </Button>
                        }
                        title={t("删除计划？", "Delete schedule?")}
                        description={t(
                            "删除计划不会删除历史执行记录。",
                            "Deleting a schedule does not delete historical runs.",
                        )}
                        confirmLabel={t("删除计划", "Delete schedule")}
                        destructive
                        confirmTestId="schedule-delete-confirm"
                        onConfirm={async () => {
                            await reportsAPI.deleteSchedule(row.id);
                            await onSaved();
                            appMessage.success(t("计划已删除", "Schedule deleted"));
                        }}
                    />
                </Space>
            ),
        });
    }

    return columns;
}
