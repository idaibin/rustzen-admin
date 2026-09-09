import { ClockCircleOutlined } from "@ant-design/icons";
import { ProTable } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Card } from "antd";

import { reportsQueryKeys, reportsQueryOptions } from "@/api/reports/query-options";
import { DataState } from "@/components/feedback/data-state";
import { DataTableShell } from "@/components/table/data-table-shell";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { REPORTS_SCHEDULE_MANAGE } from "../-schedule-permissions";
import { ScheduleDialog } from "./schedule-dialog";
import { createScheduleColumns } from "./schedule-columns";

export function SchedulePanel() {
    const client = useQueryClient();
    const canManageSchedules = useAuthStore((state) =>
        state.checkPermissions(REPORTS_SCHEDULE_MANAGE),
    );
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

    const columns = createScheduleColumns({ flowOptions, canManageSchedules, onSaved: refresh });

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
                canManageSchedules ? <ScheduleDialog flowOptions={flowOptions} onSaved={refresh} /> : null
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
