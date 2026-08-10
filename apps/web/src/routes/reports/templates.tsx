import {
    ClockCircleOutlined,
    CopyOutlined,
    DeleteOutlined,
    EditOutlined,
    GlobalOutlined,
    PlusCircleOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
    Alert,
    Button,
    Card,
    Form,
    Input,
    Modal,
    Select,
    Space,
    Switch,
    Tag,
    Typography,
} from "antd";
import { useEffect, useRef, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryKeys, reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { getSchedulePermissionState } from "./-schedule-permissions";
import {
    isCurrentScheduleToggle,
    settleScheduleToggle,
    startScheduleToggle,
    type ScheduleToggleRequest,
    type ScheduleToggleTracker,
} from "./-schedule-toggle";

export const Route = createFileRoute("/reports/templates")({ component: FlowsPage });

const example: Reports.FlowStep[] = [
    { action: "goto", url: "/report" },
    { action: "fill", selector: "#value", value: "{{input.value}}" },
    { action: "click", selector: "button[type=submit]" },
    { action: "waitFor", selector: "form" },
    { action: "screenshot", name: "submitted" },
];

function FlowsPage() {
    const client = useQueryClient();
    const { canViewFlows, canViewSchedules } = useAuthStore((state) =>
        getSchedulePermissionState(state.checkPermissions),
    );
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
            <DataTableShell>
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

function TargetDialog({ onSaved }: { onSaved: () => Promise<unknown> }) {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [baseUrl, setBaseUrl] = useState("");
    const mutation = useMutation({
        mutationFn: () =>
            reportsAPI.createSystem({
                name: name.trim(),
                baseUrl: baseUrl.trim(),
                enabled: true,
            }),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(t("目标系统已添加", "Target system added"));
            setOpen(false);
        },
    });

    return (
        <>
            <Button type="default" icon={<GlobalOutlined />} onClick={() => setOpen(true)}>
                {t("添加目标系统", "Add target system")}
            </Button>
            <Modal
                open={open}
                title={t("添加报表目标", "Add report target")}
                onCancel={() => {
                    setOpen(false);
                    setName("");
                    setBaseUrl("");
                }}
                footer={null}
                destroyOnHidden
            >
                <p className="mb-3 text-sm text-muted-foreground">
                    {t(
                        "模板只能在这个可信来源内导航。",
                        "Templates can only navigate within this trusted origin.",
                    )}
                </p>
                <Form layout="vertical">
                    <Form.Item label={t("名称", "Name")}>
                        <Input value={name} onChange={(event) => setName(event.target.value)} />
                    </Form.Item>
                    <Form.Item label={t("基础地址", "Base URL")}>
                        <Input
                            value={baseUrl}
                            placeholder="https://example.com"
                            onChange={(event) => setBaseUrl(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button type="default" onClick={() => setOpen(false)}>
                            {t("取消", "Cancel")}
                        </Button>
                        <Button
                            type="primary"
                            loading={mutation.isPending}
                            disabled={!name.trim() || !baseUrl.trim()}
                            onClick={() => mutation.mutate()}
                        >
                            {t("添加目标系统", "Add target system")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function FlowDialog({
    systems,
    flow,
    onSaved,
}: {
    systems: Reports.System[];
    flow?: Reports.Flow;
    onSaved: () => Promise<unknown>;
}) {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [systemId, setSystemId] = useState("");
    const [json, setJson] = useState("");

    useEffect(() => {
        if (open) {
            setName(flow?.name ?? "");
            setSystemId(flow?.systemId ?? systems[0]?.id ?? "");
            setJson(JSON.stringify(flow?.steps ?? example, null, 2));
        }
    }, [open, flow, systems]);

    const mutation = useMutation({
        mutationFn: (input: Reports.SaveFlow) =>
            flow ? reportsAPI.updateFlow(flow.id, input) : reportsAPI.createFlow(input),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(
                flow ? t("流程已更新", "Template updated") : t("流程已创建", "Template created"),
            );
            setOpen(false);
        },
    });

    const save = () => {
        try {
            const steps = JSON.parse(json) as Reports.FlowStep[];
            if (!Array.isArray(steps)) {
                throw new Error();
            }
            mutation.mutate({ name, systemId, steps });
        } catch {
            appMessage.error(t("步骤必须是有效的 JSON 数组", "Steps must be a valid JSON array"));
        }
    };

    return (
        <>
            <Button
                type={flow ? "text" : "primary"}
                icon={flow ? <EditOutlined /> : <PlusCircleOutlined />}
                disabled={!systems.length}
                onClick={() => setOpen(true)}
            >
                {flow ? null : t("新建模板", "New template")}
            </Button>
            <Modal
                open={open}
                title={
                    flow ? t("编辑模板", "Edit template") : t("新建报表模板", "New report template")
                }
                onCancel={() => setOpen(false)}
                footer={null}
                width={760}
                destroyOnHidden
            >
                <p className="mb-4 text-sm text-muted-foreground">
                    {t(
                        "支持的动作：goto、fill、click、waitFor、assertText、screenshot。",
                        "Supported actions: goto, fill, click, waitFor, assertText, screenshot.",
                    )}
                </p>
                <Form layout="vertical">
                    <div className="grid gap-4 sm:grid-cols-2">
                        <Form.Item label={t("名称", "Name")}>
                            <Input value={name} onChange={(event) => setName(event.target.value)} />
                        </Form.Item>
                        <Form.Item label={t("系统", "System")}>
                            <Select
                                value={systemId}
                                onChange={(value) => setSystemId(value)}
                                options={systems.map((s) => ({ value: s.id, label: s.name }))}
                                placeholder={t("选择系统", "Select a system")}
                                allowClear={false}
                            />
                        </Form.Item>
                    </div>
                    <Form.Item label={t("步骤 JSON", "Steps JSON")}>
                        <Input.TextArea
                            className="font-mono text-xs"
                            rows={15}
                            value={json}
                            onChange={(event) => setJson(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button type="default" onClick={() => setOpen(false)}>
                            {t("取消", "Cancel")}
                        </Button>
                        <Button
                            type="primary"
                            loading={mutation.isPending}
                            disabled={!name || !systemId}
                            onClick={save}
                        >
                            {t("校验并保存", "Validate and save")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function DeleteFlowDialog({
    flow,
    onDeleted,
}: {
    flow: Reports.Flow;
    onDeleted: () => Promise<unknown>;
}) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);

    const deleteFlow = async () => {
        setSubmitting(true);
        try {
            await reportsAPI.deleteFlow(flow.id);
            await onDeleted();
            appMessage.success(t("流程已删除", "Template deleted"));
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <Button
                type="text"
                danger
                icon={<DeleteOutlined />}
                aria-label={t("删除流程", "Delete template")}
                onClick={() => setOpen(true)}
            />
            <Modal
                open={open}
                title={t("删除流程？", "Delete template?")}
                onCancel={() => setOpen(false)}
                footer={null}
                centered
                destroyOnHidden
            >
                <p className="mb-4">
                    {t(
                        "已有填报执行必须不再引用该流程。",
                        "Existing report runs must no longer reference this template.",
                    )}
                </p>
                <div className="flex justify-end gap-2">
                    <Button type="default" onClick={() => setOpen(false)}>
                        {t("取消", "Cancel")}
                    </Button>
                    <Button type="primary" danger loading={submitting} onClick={deleteFlow}>
                        {t("删除", "Delete")}
                    </Button>
                </div>
            </Modal>
        </>
    );
}

function SchedulePanel() {
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
                                <Button type="link" danger>
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
                <DataTableShell>
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

function ScheduleToggle({
    schedule,
    onSaved,
}: {
    schedule: Reports.Schedule;
    onSaved: () => Promise<unknown>;
}) {
    const tracker = useRef<ScheduleToggleTracker>({
        id: schedule.id,
        nextRequestId: 0,
        pendingRequestId: null,
    });
    const [pendingRequestId, setPendingRequestId] = useState<number | null>(null);
    const [errorMessage, setErrorMessage] = useState<string>();
    const mutation = useMutation({
        mutationFn: async ({ id, enabled, requestId }: ScheduleToggleRequest) => {
            if (id !== schedule.id) {
                throw new Error(t("计划标识已变化", "Schedule identity changed"));
            }
            await reportsAPI.updateSchedule(schedule.id, {
                ...toSaveSchedule(schedule),
                enabled,
            });
            return { id: schedule.id, requestId };
        },
        onSuccess: async (request) => {
            if (!isCurrentScheduleToggle(tracker.current, request)) return;
            await onSaved();
        },
        onError: (error, variables) => {
            if (!isCurrentScheduleToggle(tracker.current, variables)) return;
            setErrorMessage(
                error instanceof Error && error.message
                    ? error.message
                    : t("计划状态更新失败", "Failed to update schedule status"),
            );
        },
        onSettled: (_data, error, variables) => {
            if (variables && isCurrentScheduleToggle(tracker.current, variables)) {
                tracker.current = settleScheduleToggle(
                    tracker.current,
                    variables,
                    error instanceof Error && error.message ? error.message : undefined,
                );
                setPendingRequestId(null);
            }
        },
    });

    const pending = pendingRequestId !== null || mutation.isPending;
    const toggle = (enabled: boolean) => {
        if (pending) return;
        const started = startScheduleToggle(tracker.current, enabled);
        if (!started.request) return;
        tracker.current = started.tracker;
        const { requestId } = started.request;
        setPendingRequestId(requestId);
        setErrorMessage(undefined);
        mutation.mutate(started.request);
    };

    return (
        <span className="inline-flex items-center gap-2">
            <Switch
                size="small"
                checked={schedule.enabled}
                checkedChildren={t("启用", "On")}
                unCheckedChildren={t("停用", "Off")}
                loading={pending}
                disabled={pending}
                onChange={toggle}
            />
            {errorMessage ? (
                <Typography.Text type="danger" className="text-xs" aria-live="polite">
                    {errorMessage}
                </Typography.Text>
            ) : null}
        </span>
    );
}

function ScheduleDialog({
    flowOptions,
    schedule,
    onSaved,
}: {
    flowOptions: Reports.FlowOption[];
    schedule?: Reports.Schedule;
    onSaved: () => Promise<unknown>;
}) {
    const [open, setOpen] = useState(false);
    const [flowId, setFlowId] = useState("");
    const [cadence, setCadence] = useState<Reports.ScheduleCadence>("daily");
    const [weekday, setWeekday] = useState<number>(1);
    const [dueTime, setDueTime] = useState("09:00");
    const [inputJson, setInputJson] = useState("{}");
    const [description, setDescription] = useState("");
    const [enabled, setEnabled] = useState(true);

    useEffect(() => {
        if (!open) return;
        setFlowId(schedule?.flowId ?? flowOptions.find((flow) => flow.enabled)?.id ?? "");
        setCadence(schedule?.cadence ?? "daily");
        setWeekday(schedule?.weekday ?? 1);
        setDueTime(schedule?.dueTime ?? "09:00");
        setInputJson(JSON.stringify(schedule?.input ?? {}, null, 2));
        setDescription(schedule?.description ?? "");
        setEnabled(schedule?.enabled ?? true);
    }, [flowOptions, open, schedule]);

    const mutation = useMutation({
        mutationFn: (input: Reports.SaveSchedule) =>
            schedule
                ? reportsAPI.updateSchedule(schedule.id, input)
                : reportsAPI.createSchedule(input),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(
                schedule
                    ? t("计划已更新", "Schedule updated")
                    : t("计划已创建", "Schedule created"),
            );
            setOpen(false);
        },
    });

    const save = () => {
        try {
            const input = JSON.parse(inputJson) as Record<string, unknown>;
            if (!input || Array.isArray(input) || typeof input !== "object") throw new Error();
            if (!flowId || !dueTime || (cadence === "weekly" && (weekday < 0 || weekday > 6))) {
                throw new Error();
            }
            const selectedFlow = flowOptions.find((flow) => flow.id === flowId);
            if (!selectedFlow?.enabled) {
                throw new Error(t("所选流程目标已停用。", "The selected flow target is disabled."));
            }
            mutation.mutate({
                flowId,
                cadence,
                weekday: cadence === "weekly" ? weekday : undefined,
                dueTime,
                input,
                description: description.trim(),
                enabled,
            });
        } catch {
            appMessage.error(
                t(
                    "请填写完整的计划字段，并提供有效的 JSON 对象。",
                    "Complete the schedule fields and provide a valid JSON object.",
                ),
            );
        }
    };

    return (
        <>
            <Button
                type={schedule ? "link" : "primary"}
                onClick={() => setOpen(true)}
                disabled={!flowOptions.some((flow) => flow.enabled)}
            >
                {schedule ? t("编辑", "Edit") : t("新建计划", "New schedule")}
            </Button>
            <Modal
                open={open}
                title={
                    schedule
                        ? t("编辑定时报表计划", "Edit scheduled report")
                        : t("新建定时报表计划", "New scheduled report")
                }
                onCancel={() => setOpen(false)}
                footer={null}
                width={760}
                destroyOnHidden
            >
                <Alert
                    className="mb-4"
                    type="warning"
                    showIcon
                    message={t(
                        "不要提交密码、Token、密钥或其他敏感信息。",
                        "Do not submit passwords, tokens, keys, or other sensitive information.",
                    )}
                />
                <Form layout="vertical">
                    <Form.Item label={t("流程", "Template")} required>
                        <Select
                            value={flowId || undefined}
                            onChange={(value) => setFlowId(value)}
                            options={flowOptions.map((flow) => ({
                                value: flow.id,
                                label: flow.enabled
                                    ? flow.name
                                    : `${flow.name} (${t("目标已停用", "Target disabled")})`,
                                disabled: !flow.enabled,
                            }))}
                            placeholder={t("选择流程", "Select a template")}
                        />
                    </Form.Item>
                    <div className="grid gap-4 sm:grid-cols-2">
                        <Form.Item label={t("频率", "Cadence")} required>
                            <Select
                                value={cadence}
                                onChange={(value: Reports.ScheduleCadence) => setCadence(value)}
                                options={[
                                    { value: "daily", label: t("每日", "Daily") },
                                    { value: "weekly", label: t("每周", "Weekly") },
                                ]}
                            />
                        </Form.Item>
                        {cadence === "weekly" ? (
                            <Form.Item label={t("星期", "Weekday")} required>
                                <Select
                                    value={weekday}
                                    onChange={(value) => setWeekday(value)}
                                    options={[
                                        { value: 0, label: t("周一", "Monday") },
                                        { value: 1, label: t("周二", "Tuesday") },
                                        { value: 2, label: t("周三", "Wednesday") },
                                        { value: 3, label: t("周四", "Thursday") },
                                        { value: 4, label: t("周五", "Friday") },
                                        { value: 5, label: t("周六", "Saturday") },
                                        { value: 6, label: t("周日", "Sunday") },
                                    ]}
                                />
                            </Form.Item>
                        ) : null}
                    </div>
                    <Form.Item label={t("本地执行时间", "Local due time")} required>
                        <Input
                            type="time"
                            value={dueTime}
                            onChange={(event) => setDueTime(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("安全输入 JSON", "Safe input JSON")}>
                        <Input.TextArea
                            className="font-mono text-xs"
                            rows={7}
                            value={inputJson}
                            onChange={(event) => setInputJson(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("描述", "Description")}>
                        <Input
                            value={description}
                            onChange={(event) => setDescription(event.target.value)}
                        />
                    </Form.Item>
                    <Form.Item label={t("启用计划", "Enable schedule")}>
                        <Switch checked={enabled} onChange={setEnabled} />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button onClick={() => setOpen(false)}>{t("取消", "Cancel")}</Button>
                        <Button type="primary" loading={mutation.isPending} onClick={save}>
                            {t("校验并保存", "Validate and save")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function toSaveSchedule(schedule: Reports.Schedule): Reports.SaveSchedule {
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

function formatScheduleCadence(schedule: Reports.Schedule): string {
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

function ScheduleDecisionTag({ decision }: { decision: Reports.ScheduleDecision }) {
    return (
        <Tag color={decision === "enqueued" ? "success" : "warning"}>
            {decision === "enqueued" ? t("已入队", "Enqueued") : t("已跳过", "Skipped")}
        </Tag>
    );
}
