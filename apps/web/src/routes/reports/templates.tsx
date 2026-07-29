import {
    CopyOutlined,
    DeleteOutlined,
    EditOutlined,
    GlobalOutlined,
    PlusCircleOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Modal, Select, Form } from "antd";
import { useEffect, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryKeys, reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

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
    const { data: systems = [] } = useQuery(reportsQueryOptions.systems());
    const { data: flows = [], error, isPending, refetch } = useQuery(reportsQueryOptions.flows());
    const refresh = () => client.invalidateQueries({ queryKey: reportsQueryKeys.flows() });
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
