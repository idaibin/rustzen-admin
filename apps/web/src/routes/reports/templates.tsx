import { CopyOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button } from "antd";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryKeys, reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { getSchedulePermissionState } from "./-schedule-permissions";
import { DeleteFlowDialog } from "./-templates/delete-flow-dialog";
import { FlowDialog } from "./-templates/flow-dialog";
import { SchedulePanel } from "./-templates/schedule-panel";
import { TargetDialog } from "./-templates/target-dialog";

export const Route = createFileRoute("/reports/templates")({ component: FlowsPage });

function FlowsPage() {
    const client = useQueryClient();
    const checkPermissions = useAuthStore((state) => state.checkPermissions);
    const { canViewFlows, canViewSchedules } = getSchedulePermissionState(checkPermissions);
    const { data: systems = [] } = useQuery({
        ...reportsQueryOptions.systems(),
        enabled: canViewFlows,
    });
    const {
        data: flows = [],
        error,
        isPending,
        refetch,
    } = useQuery({
        ...reportsQueryOptions.flows(),
        enabled: canViewFlows,
    });
    const refresh = () =>
        Promise.all([
            client.invalidateQueries({ queryKey: reportsQueryKeys.flows() }),
            client.invalidateQueries({ queryKey: reportsQueryKeys.flowOptions() }),
        ]);
    const refreshSystems = () => client.invalidateQueries({ queryKey: reportsQueryKeys.systems() });

    const clone = useMutation({
        mutationFn: (flow: Reports.Flow) =>
            reportsAPI.createFlow({
                systemId: flow.systemId,
                name: t(`${flow.name} 副本`, `${flow.name} copy`),
                steps: flow.steps,
            }),
        onSuccess: async () => {
            await refresh();
            appMessage.success(t("流程已复制", "Template copied"));
        },
    });

    if (!canViewFlows && !canViewSchedules) {
        return (
            <PageCard
                title={t("定时报表", "Scheduled reports")}
                description={t(
                    "查看现有报表模板的执行计划和最近结果。",
                    "Inspect schedules and recent outcomes for existing report templates.",
                )}
            >
                <DataState
                    kind="permission"
                    title={t("无报表查看权限", "Report view permission required")}
                />
            </PageCard>
        );
    }

    if (!canViewFlows) {
        return (
            <PageCard
                title={t("定时报表", "Scheduled reports")}
                description={t(
                    "查看现有报表模板的执行计划和最近结果。",
                    "Inspect schedules and recent outcomes for existing report templates.",
                )}
            >
                <AuthWrap
                    code="reports:schedule:view"
                    fallback={
                        <DataState
                            kind="permission"
                            title={t("无计划查看权限", "Schedule view permission required")}
                        />
                    }
                >
                    <SchedulePanel />
                </AuthWrap>
            </PageCard>
        );
    }

    const columns: ProColumns<Reports.Flow>[] = [
        {
            title: t("名称", "Name"),
            dataIndex: "name",
            key: "name",
            ellipsis: true,
            render: (_: unknown, row: Reports.Flow) => row.name,
        },
        {
            title: t("系统", "System"),
            dataIndex: "systemId",
            key: "system",
            width: 190,
            render: (_: unknown, row: Reports.Flow) =>
                systems.find((s) => s.id === row.systemId)?.name ?? row.systemId,
        },
        {
            title: t("步骤", "Steps"),
            dataIndex: "steps",
            key: "steps",
            width: 110,
            render: (_: unknown, row: Reports.Flow) => row.steps.length,
        },
        {
            title: t("更新时间", "Updated at"),
            dataIndex: "updatedAt",
            key: "updatedAt",
            width: 190,
            render: (_: unknown, row: Reports.Flow) => formatDateTime(row.updatedAt),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 180,
            render: (_: unknown, row: Reports.Flow) => (
                <AuthWrap code="reports:flow:manage">
                    <div className="flex justify-end gap-1">
                        <FlowDialog systems={systems} flow={row} onSaved={refresh} />
                        <Button
                            type="text"
                            icon={<CopyOutlined />}
                            aria-label={t("复制流程", "Copy template")}
                            onClick={() => clone.mutate(row)}
                        />
                        <DeleteFlowDialog flow={row} onDeleted={async () => refresh()} />
                    </div>
                </AuthWrap>
            ),
        },
    ];

    if (!flows.length && isPending) {
        return (
            <PageCard
                title={t("报表模板", "Report templates")}
                description={t(
                    "定义每个填报流程使用的目标系统和已验证步骤。",
                    "Define the target system and verified steps for each report workflow.",
                )}
                actions={
                    <div className="flex gap-2">
                        <AuthWrap code="reports:system:manage">
                            <TargetDialog onSaved={refreshSystems} />
                        </AuthWrap>
                        <AuthWrap code="reports:flow:manage">
                            <FlowDialog systems={systems} onSaved={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="loading"
                    title={t("正在加载报表模板", "Loading report templates")}
                />
            </PageCard>
        );
    }

    if (!flows.length && error) {
        return (
            <PageCard
                title={t("报表模板", "Report templates")}
                description={t(
                    "定义每个填报流程使用的目标系统和已验证步骤。",
                    "Define the target system and verified steps for each report workflow.",
                )}
                actions={
                    <div className="flex gap-2">
                        <AuthWrap code="reports:system:manage">
                            <TargetDialog onSaved={refreshSystems} />
                        </AuthWrap>
                        <AuthWrap code="reports:flow:manage">
                            <FlowDialog systems={systems} onSaved={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="error"
                    title={t("报表模板加载失败", "Failed to load report templates")}
                    description={t(
                        "无法读取模板，请检查 Reports 服务后重试。",
                        "Unable to read templates. Check the Reports service and try again.",
                    )}
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </PageCard>
        );
    }

    if (!flows.length) {
        return (
            <PageCard
                title={t("报表模板", "Report templates")}
                description={t(
                    "定义每个填报流程使用的目标系统和已验证步骤。",
                    "Define the target system and verified steps for each report workflow.",
                )}
                actions={
                    <div className="flex gap-2">
                        <AuthWrap code="reports:system:manage">
                            <TargetDialog onSaved={refreshSystems} />
                        </AuthWrap>
                        <AuthWrap code="reports:flow:manage">
                            <FlowDialog systems={systems} onSaved={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="empty"
                    title={t("暂无报表模板", "No report templates")}
                    description={t(
                        "先添加目标系统，再创建包含已验证步骤的填报模板。",
                        "Add a target system, then create a report template with verified steps.",
                    )}
                />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("报表模板", "Report templates")}
            description={t(
                "定义每个填报流程使用的目标系统和已验证步骤。",
                "Define the target system and verified steps for each report workflow.",
            )}
            actions={
                <div className="flex gap-2">
                    <AuthWrap code="reports:system:manage">
                        <TargetDialog onSaved={refreshSystems} />
                    </AuthWrap>
                    <AuthWrap code="reports:flow:manage">
                        <FlowDialog systems={systems} onSaved={refresh} />
                    </AuthWrap>
                </div>
            }
        >
            {error ? (
                <DataState
                    kind="error"
                    title={t("报表模板刷新失败", "Failed to refresh report templates")}
                    description={t("请稍后重试。", "Please try again later.")}
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                    compact
                />
            ) : null}
            <DataTableShell ariaLabel={t("报表流程", "Report flows table")}>
                <ProTable<Reports.Flow>
                    rowKey="id"
                    columns={columns}
                    dataSource={flows}
                    loading={isPending}
                    search={false}
                    options={false}
                    pagination={false}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText: (
                            <DataState
                                kind="empty"
                                title={t("暂无报表模板", "No report templates")}
                            />
                        ),
                    }}
                />
            </DataTableShell>
            <AuthWrap
                code="reports:schedule:view"
                fallback={
                    <DataState
                        kind="permission"
                        title={t("无计划查看权限", "Schedule view permission required")}
                        description={t(
                            "当前角色不能查看定时报表计划。",
                            "Your role cannot view scheduled reports.",
                        )}
                        compact
                    />
                }
            >
                <SchedulePanel />
            </AuthWrap>
        </PageCard>
    );
}
