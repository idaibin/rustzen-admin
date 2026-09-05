import { ClockCircleOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Card, Space, Tag } from "antd";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryKeys, reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { ScheduleDialog } from "./schedule-dialog";
import { ScheduleToggle } from "./schedule-toggle";
import { formatScheduleCadence, ScheduleDecisionTag } from "./schedule-utils";

export function SchedulePanel() {
    const client = useQueryClient();
    const {
        data: schedules = [],
        error,
        isFetching,
        isPending,
        refetch,
    } = useQuery(reportsQueryOptions.schedules());
    const {
        data: installationSettings,
        error: settingsError,
        isPending: settingsPending,
        refetch: refetchSettings,
    } = useQuery(reportsQueryOptions.settings());
    const {
        data: flowOptions = [],
        error: flowOptionsError,
        isPending: flowOptionsPending,
        refetch: refetchFlowOptions,
    } = useQuery(reportsQueryOptions.flowOptions());
    const refresh = () =>
        Promise.all([
            client.invalidateQueries({ queryKey: reportsQueryKeys.schedules() }),
            client.invalidateQueries({ queryKey: reportsQueryKeys.settings() }),
            client.invalidateQueries({ queryKey: reportsQueryKeys.flowOptions() }),
        ]);
    const timezone = installationSettings?.timezone;
    const decisions = new Set(
        schedules
            .map((schedule) => schedule.lastOccurrence?.decision)
            .filter((decision): decision is Reports.ScheduleDecision => Boolean(decision)),
    );

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
                    <div className="text-xs text-muted-foreground">
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
                    <div>
                        <ScheduleDecisionTag decision={occurrence.decision} />
                        <div className="text-xs text-muted-foreground">
                            {occurrence.reason ||
                                formatDateTime(occurrence.decidedAt, row.timezone)}
                        </div>
                        {occurrence.runId ? (
                            <Button
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
        {
            title: t("操作", "Actions"),
            key: "actions",
            width: 230,
            fixed: "right",
            render: (_value: unknown, row: Reports.Schedule) => (
                <AuthWrap code="reports:schedule:manage">
                    <Space size="small">
                        <ScheduleDialog
                            flowOptions={flowOptions}
                            schedule={row}
                            onSaved={refresh}
                        />
                        <ScheduleToggle schedule={row} onSaved={refresh} />
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
                                await refresh();
                                appMessage.success(t("计划已删除", "Schedule deleted"));
                            }}
                        />
                    </Space>
                </AuthWrap>
            ),
        },
    ];

    return (
        <Card
            data-testid="schedule-panel"
            title={
                <span className="inline-flex items-center gap-2">
                    <ClockCircleOutlined />
                    {t("定时报表计划", "Scheduled reports")}
                </span>
            }
            extra={
                <AuthWrap code="reports:schedule:manage">
                    <ScheduleDialog flowOptions={flowOptions} onSaved={refresh} />
                </AuthWrap>
            }
        >
            <div className="mb-3 text-sm text-muted-foreground">
                {t(
                    `安装时区：${timezone ?? (settingsPending ? "读取中" : "不可用")}。仅支持每日或每周计划；错过的时间段会记录为跳过。`,
                    `Installation timezone: ${timezone ?? (settingsPending ? "Loading" : "Unavailable")}. Only daily and weekly schedules are supported; missed occurrences are recorded as skipped.`,
                )}
            </div>
            {settingsError ? (
                <Alert
                    className="mb-3"
                    type="warning"
                    showIcon
                    message={t("安装时区读取失败", "Installation timezone unavailable")}
                    description={t(
                        "无法确认安装时区；不会使用默认时区替代。",
                        "The installation timezone could not be confirmed; no default timezone is substituted.",
                    )}
                    action={
                        <Button size="small" onClick={() => void refetchSettings()}>
                            {t("重试", "Retry")}
                        </Button>
                    }
                />
            ) : null}
            {flowOptionsError ? (
                <Alert
                    className="mb-3"
                    type="error"
                    showIcon
                    message={t("流程选项读取失败", "Flow options unavailable")}
                    description={t(
                        "无法读取可用于定时计划的流程；不会使用流程列表替代。",
                        "Flow options for scheduled reports could not be loaded; the full flow list is not used as a fallback.",
                    )}
                    action={
                        <Button size="small" onClick={() => void refetchFlowOptions()}>
                            {t("重试", "Retry")}
                        </Button>
                    }
                />
            ) : null}
            {!flowOptionsPending && !flowOptionsError && flowOptions.length === 0 ? (
                <Alert
                    className="mb-3"
                    type="info"
                    showIcon
                    message={t("暂无可用流程", "No report flows available")}
                    description={t(
                        "创建定时计划前，需要至少一个启用的报表流程。",
                        "At least one enabled report flow is required before creating a schedule.",
                    )}
                />
            ) : null}
            {error && schedules.length ? (
                <DataState
                    kind="error"
                    title={t("计划刷新失败", "Failed to refresh schedules")}
                    description={t("请稍后重试。", "Please try again later.")}
                    action={<Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>}
                    compact
                />
            ) : null}
            {decisions.size > 1 ? (
                <Alert
                    className="mb-3"
                    type="warning"
                    showIcon
                    message={t(
                        "计划包含混合执行结果",
                        "Schedules contain mixed occurrence outcomes",
                    )}
                    description={t(
                        "请分别查看每个计划的入队或跳过原因。",
                        "Review each schedule's enqueued or skipped reason individually.",
                    )}
                />
            ) : null}
            {!schedules.length && isPending ? (
                <DataState kind="loading" title={t("正在加载计划", "Loading schedules")} compact />
            ) : !schedules.length && error ? (
                <DataState
                    kind="error"
                    title={t("计划加载失败", "Failed to load schedules")}
                    description={t(
                        "无法读取定时报表计划，请检查 Reports 服务后重试。",
                        "Unable to read scheduled reports. Check the Reports service and try again.",
                    )}
                    action={
                        <Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>
                    }
                    compact
                />
            ) : !schedules.length ? (
                <DataState
                    kind="empty"
                    title={t("暂无定时报表计划", "No scheduled reports")}
                    description={t(
                        "为现有报表模板创建每日或每周计划。",
                        "Create a daily or weekly schedule for an existing report template.",
                    )}
                    compact
                />
            ) : (
                <DataTableShell ariaLabel={t("报表计划", "Report schedules table")}>
                    <ProTable<Reports.Schedule>
                        rowKey="id"
                        columns={columns}
                        dataSource={schedules}
                        loading={isFetching}
                        search={false}
                        options={false}
                        pagination={false}
                        toolBarRender={false}
                        tableAlertOptionRender={false}
                        rowSelection={false}
                    />
                </DataTableShell>
            )}
        </Card>
    );
}
