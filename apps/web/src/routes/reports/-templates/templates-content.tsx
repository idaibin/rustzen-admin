import { CopyOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { Button } from "antd";
import type { ReactNode } from "react";

import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { DeleteFlowDialog } from "./delete-flow-dialog";
import { FlowDialog } from "./flow-dialog";
import { SchedulePanel } from "./schedule-panel";
import { TargetDialog } from "./target-dialog";

type Props = {
    systems: Reports.System[];
    flows: Reports.Flow[];
    error: Error | null;
    isPending: boolean;
    refetch: () => void;
    onClone: (flow: Reports.Flow) => void;
    onRefresh: () => Promise<unknown>;
    onRefreshSystems: () => Promise<unknown>;
};

export function TemplatesContent({
    systems,
    flows,
    error,
    isPending,
    refetch,
    onClone,
    onRefresh,
    onRefreshSystems,
}: Props) {
    const actions = (
        <div className="flex gap-2">
            <AuthWrap code="reports:system:manage">
                <TargetDialog onSaved={onRefreshSystems} />
            </AuthWrap>
            <AuthWrap code="reports:flow:manage">
                <FlowDialog systems={systems} onSaved={onRefresh} />
            </AuthWrap>
        </div>
    );
    const columns: ProColumns<Reports.Flow>[] = [
        {
            title: t("名称", "Name"),
            dataIndex: "name",
            key: "name",
            ellipsis: true,
            render: (_: unknown, row) => row.name,
        },
        {
            title: t("系统", "System"),
            dataIndex: "systemId",
            key: "system",
            width: 190,
            render: (_: unknown, row) => systems.find((system) => system.id === row.systemId)?.name ?? row.systemId,
        },
        {
            title: t("步骤", "Steps"),
            dataIndex: "steps",
            key: "steps",
            width: 110,
            render: (_: unknown, row) => row.steps.length,
        },
        {
            title: t("更新时间", "Updated at"),
            dataIndex: "updatedAt",
            key: "updatedAt",
            width: 190,
            render: (_: unknown, row) => formatDateTime(row.updatedAt),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 180,
            render: (_: unknown, row) => (
                <AuthWrap code="reports:flow:manage">
                    <div className="flex justify-end gap-1">
                        <FlowDialog systems={systems} flow={row} onSaved={onRefresh} />
                        <Button
                            type="text"
                            icon={<CopyOutlined />}
                            aria-label={t("复制流程", "Copy template")}
                            onClick={() => onClone(row)}
                        />
                        <DeleteFlowDialog flow={row} onDeleted={async () => onRefresh()} />
                    </div>
                </AuthWrap>
            ),
        },
    ];
    const card = (children: ReactNode) => (
        <PageCard
            title={t("报表模板", "Report templates")}
            description={t("定义每个填报流程使用的目标系统和已验证步骤。", "Define the target system and verified steps for each report workflow.")}
            actions={actions}
        >
            {children}
        </PageCard>
    );

    if (!flows.length && isPending) {
        return card(<DataState kind="loading" title={t("正在加载报表模板", "Loading report templates")} />);
    }

    if (!flows.length && error) {
        return card(
            <DataState
                kind="error"
                title={t("报表模板加载失败", "Failed to load report templates")}
                description={t("无法读取模板，请检查 Reports 服务后重试。", "Unable to read templates. Check the Reports service and try again.")}
                action={<Button type="primary" onClick={refetch}>{t("重新加载", "Reload")}</Button>}
            />,
        );
    }

    if (!flows.length) {
        return card(
            <DataState
                kind="empty"
                title={t("暂无报表模板", "No report templates")}
                description={t("先添加目标系统，再创建包含已验证步骤的填报模板。", "Add a target system, then create a report template with verified steps.")}
            />,
        );
    }

    return card(
        <>
            {error ? (
                <DataState
                    kind="error"
                    title={t("报表模板刷新失败", "Failed to refresh report templates")}
                    description={t("请稍后重试。", "Please try again later.")}
                    action={<Button type="primary" onClick={refetch}>{t("重新加载", "Reload")}</Button>}
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
                    locale={{ emptyText: <DataState kind="empty" title={t("暂无报表模板", "No report templates")} /> }}
                />
            </DataTableShell>
            <AuthWrap
                code="reports:schedule:view"
                fallback={
                    <DataState
                        kind="permission"
                        title={t("无计划查看权限", "Schedule view permission required")}
                        description={t("当前角色不能查看定时报表计划。", "Your role cannot view scheduled reports.")}
                        compact
                    />
                }
            >
                <SchedulePanel />
            </AuthWrap>
        </>,
    );
}
