import {
    DeleteOutlined,
    EditOutlined,
    PlayCircleOutlined,
    PlusOutlined,
    PoweroffOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, InputNumber, Modal, Space, Tag } from "antd";
import { useEffect, useState } from "react";

import { appMessage, monitorAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { checkIncidentHref } from "./-check-context";

export const Route = createFileRoute("/monitoring/checks")({ component: MonitoringChecksPage });

const pageSize = 20;

function MonitoringChecksPage() {
    const [current, setCurrent] = useState(1);
    const queryClient = useQueryClient();
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["monitor", "checks", current],
        queryFn: () => monitorAPI.checks({ current, pageSize }),
        refetchInterval: 10_000,
    });
    const refresh = () => queryClient.invalidateQueries({ queryKey: ["monitor", "checks"] });
    const enabledMutation = useMutation({
        mutationFn: ({ id, enabled }: { id: string; enabled: boolean }) =>
            monitorAPI.setCheckEnabled(id, enabled),
        onSuccess: async () => {
            await refresh();
            appMessage.success(t("检查状态已更新", "Check status updated"));
        },
    });
    const deleteMutation = useMutation({
        mutationFn: monitorAPI.deleteCheck,
        onSuccess: async () => {
            await refresh();
            appMessage.success(t("检查已删除", "Check deleted"));
        },
    });

    if (!data && isPending) {
        return (
            <PageCard
                title={t("服务监控", "Service monitoring")}
                description={t(
                    "按固定间隔探测 TCP 服务并查看留存结果。",
                    "Probe TCP services at fixed intervals and view retained results.",
                )}
                actions={
                    <AuthWrap code="monitor:check:manage">
                        <CheckDialog onSaved={refresh} />
                    </AuthWrap>
                }
            >
                <DataState kind="loading" title={t("正在加载服务检查", "Loading service checks")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                title={t("服务监控", "Service monitoring")}
                description={t(
                    "按固定间隔探测 TCP 服务并查看留存结果。",
                    "Probe TCP services at fixed intervals and view retained results.",
                )}
                actions={
                    <AuthWrap code="monitor:check:manage">
                        <CheckDialog onSaved={refresh} />
                    </AuthWrap>
                }
            >
                <DataState
                    kind="error"
                    title={t("服务检查加载失败", "Failed to load service checks")}
                    description={t(
                        "无法读取 TCP 检查，请检查 Monitor 服务后重试。",
                        "Unable to read TCP checks. Check the Monitor service and try again.",
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

    const checks = data?.data ?? [];
    const total = data?.total ?? 0;

    const columns: ProColumns<Monitor.Check>[] = [
        {
            title: t("名称", "Name"),
            dataIndex: "name",
            key: "name",
        },
        {
            title: t("目标", "Target"),
            key: "target",
            render: (_: unknown, row: Monitor.Check) => `${row.host}:${row.port}`,
        },
        {
            title: t("状态", "Status"),
            key: "status",
            render: (_: unknown, row: Monitor.Check) => {
                if (!row.enabled) {
                    return <Tag color="default">{t("已停用", "Disabled")}</Tag>;
                }
                if (row.lastStatus === "down") {
                    return <Tag color="error">{t("异常", "Down")}</Tag>;
                }
                if (row.lastStatus === "up") {
                    return <Tag color="success">{t("正常", "Up")}</Tag>;
                }
                return <Tag>{t("等待检查", "Pending")}</Tag>;
            },
        },
        {
            title: t("间隔", "Interval"),
            dataIndex: "intervalSeconds",
            key: "intervalSeconds",
            render: (_: unknown, row: Monitor.Check) => `${row.intervalSeconds}s`,
        },
        {
            title: t("失败次数", "Failures"),
            key: "failures",
            render: (_: unknown, row: Monitor.Check) =>
                `${row.consecutiveFailures}/${row.failureThreshold}`,
        },
        {
            title: t("最后检查", "Last checked"),
            dataIndex: "lastCheckedAt",
            key: "lastCheckedAt",
            render: (_: unknown, row: Monitor.Check) => formatDateTime(row.lastCheckedAt),
        },
        {
            title: t("事件", "Incidents"),
            key: "incidents",
            width: 110,
            render: (_: unknown, row: Monitor.Check) => (
                <AuthWrap code="monitor:incident:view">
                    <Button type="link" size="small" href={checkIncidentHref(row.id)}>
                        {t("查看事件", "View incidents")}
                    </Button>
                </AuthWrap>
            ),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 240,
            render: (_: unknown, row: Monitor.Check) => (
                <AuthWrap code="monitor:check:manage">
                    <Space size="small" wrap>
                        <CheckDialog check={row} onSaved={refresh} />
                        <Button
                            icon={<PoweroffOutlined />}
                            size="small"
                            danger={row.enabled}
                            loading={enabledMutation.isPending}
                            onClick={() =>
                                enabledMutation.mutate({
                                    id: row.id,
                                    enabled: !row.enabled,
                                })
                            }
                        >
                            {row.enabled ? t("禁用", "Disable") : t("启用", "Enable")}
                        </Button>
                        <ConfirmDialog
                            trigger={
                                <Button size="small" danger icon={<DeleteOutlined />}>
                                    {t("删除", "Delete")}
                                </Button>
                            }
                            title={t("删除 TCP 检查？", "Delete TCP check?")}
                            description={t(
                                "已留存的检查结果也会一并删除。",
                                "Retained check results will also be deleted.",
                            )}
                            confirmLabel={t("删除", "Delete")}
                            destructive
                            disabled={deleteMutation.isPending}
                            onConfirm={() => deleteMutation.mutateAsync(row.id).then(() => {})}
                        />
                    </Space>
                </AuthWrap>
            ),
        },
    ];

    return (
        <PageCard
            title={t("服务监控", "Service monitoring")}
            description={t(
                "按固定间隔探测 TCP 服务并查看留存结果。",
                "Probe TCP services at fixed intervals and view retained results.",
            )}
            actions={
                <AuthWrap code="monitor:check:manage">
                    <CheckDialog onSaved={refresh} />
                </AuthWrap>
            }
        >
            <ProTable<Monitor.Check>
                rowKey="id"
                columns={columns}
                dataSource={checks}
                loading={isFetching}
                search={false}
                options={false}
                pagination={{
                    current,
                    pageSize,
                    total,
                    showSizeChanger: false,
                    onChange: (page) => setCurrent(page),
                }}
                locale={{
                    emptyText:
                        checks.length === 0 ? (
                            <DataState
                                kind="empty"
                                title={t("暂无服务检查", "No service checks")}
                                description={t(
                                    "添加 TCP 检查后，系统会按设定间隔持续探测服务状态。",
                                    "Add a TCP check to probe the service continuously at the configured interval.",
                                )}
                            />
                        ) : undefined,
                }}
            />
        </PageCard>
    );
}

function CheckDialog({
    check,
    onSaved,
}: {
    check?: Monitor.Check;
    onSaved: () => Promise<unknown>;
}) {
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [host, setHost] = useState("");
    const [port, setPort] = useState("");
    const [interval, setIntervalValue] = useState("");
    const [timeout, setTimeoutValue] = useState("");
    const [threshold, setThreshold] = useState("");

    useEffect(() => {
        if (!open) {
            return;
        }
        setName(check?.name ?? "");
        setHost(check?.host ?? "");
        setPort(String(check?.port ?? 443));
        setIntervalValue(String(check?.intervalSeconds ?? 60));
        setTimeoutValue(String(check?.timeoutMs ?? 5000));
        setThreshold(String(check?.failureThreshold ?? 3));
    }, [check, open]);

    const saveMutation = useMutation({
        mutationFn: (input: Monitor.SaveCheck) =>
            check ? monitorAPI.updateCheck(check.id, input) : monitorAPI.createCheck(input),
        onSuccess: async () => {
            await onSaved();
            appMessage.success(
                check ? t("检查已更新", "Check updated") : t("检查已创建", "Check created"),
            );
            setOpen(false);
        },
    });

    const testMutation = useMutation({
        mutationFn: monitorAPI.testCheck,
        onSuccess: (result) => {
            if (result.status === "up") {
                appMessage.success(
                    t(
                        `连接成功，耗时 ${result.latencyMs ?? 0} ms`,
                        `Connection successful in ${result.latencyMs ?? 0} ms`,
                    ),
                );
            } else {
                appMessage.error(result.error ?? t("连接失败", "Connection failed"));
            }
        },
    });

    const input = {
        name: name.trim(),
        host: host.trim(),
        port: Number(port),
        intervalSeconds: Number(interval),
        timeoutMs: Number(timeout),
        failureThreshold: Number(threshold),
        enabled: check?.enabled ?? true,
    };

    const canSave =
        Boolean(input.name) &&
        Boolean(input.host) &&
        Number.isInteger(input.port) &&
        Number.isInteger(input.intervalSeconds) &&
        Number.isInteger(input.timeoutMs) &&
        Number.isInteger(input.failureThreshold);

    const canTest =
        Boolean(input.host) && Number.isInteger(input.port) && Number.isInteger(input.timeoutMs);

    return (
        <>
            {check ? (
                <Button
                    size="small"
                    icon={<EditOutlined />}
                    onClick={() => setOpen(true)}
                    aria-label={t("编辑检查", "Edit check")}
                />
            ) : (
                <Button type="primary" icon={<PlusOutlined />} onClick={() => setOpen(true)}>
                    {t("新建检查", "New check")}
                </Button>
            )}
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                title={
                    check
                        ? t("编辑 TCP 检查", "Edit TCP check")
                        : t("新建 TCP 检查", "New TCP check")
                }
                footer={
                    <div className="flex justify-end gap-2">
                        <Button
                            icon={<PlayCircleOutlined />}
                            onClick={() =>
                                testMutation.mutate({
                                    host: input.host,
                                    port: input.port,
                                    timeoutMs: input.timeoutMs,
                                })
                            }
                            disabled={!canTest || testMutation.isPending}
                        >
                            {t("测试", "Test")}
                        </Button>
                        <Button
                            type="primary"
                            onClick={() => saveMutation.mutate(input)}
                            disabled={!canSave || saveMutation.isPending}
                            loading={saveMutation.isPending}
                        >
                            {t("保存", "Save")}
                        </Button>
                    </div>
                }
                destroyOnHidden
            >
                <div className="grid gap-4 sm:grid-cols-2">
                    <div className="sm:col-span-2">
                        <label htmlFor={check ? `check-name-${check.id}` : "check-name"}>
                            {t("名称", "Name")}
                        </label>
                        <Input
                            id={check ? `check-name-${check.id}` : "check-name"}
                            value={name}
                            onChange={(event) => setName(event.target.value)}
                        />
                    </div>
                    <div>
                        <label htmlFor={check ? `check-host-${check.id}` : "check-host"}>
                            {t("主机", "Host")}
                        </label>
                        <Input
                            id={check ? `check-host-${check.id}` : "check-host"}
                            value={host}
                            onChange={(event) => setHost(event.target.value)}
                        />
                    </div>
                    <div>
                        <label htmlFor={check ? `check-port-${check.id}` : "check-port"}>
                            {t("端口", "Port")}
                        </label>
                        <InputNumber
                            id={check ? `check-port-${check.id}` : "check-port"}
                            className="w-full"
                            min={1}
                            step={1}
                            value={Number(port)}
                            onChange={(value) => setPort(String(value ?? ""))}
                        />
                    </div>
                    <div>
                        <label htmlFor={check ? `check-interval-${check.id}` : "check-interval"}>
                            {t("间隔秒数", "Interval (seconds)")}
                        </label>
                        <InputNumber
                            id={check ? `check-interval-${check.id}` : "check-interval"}
                            className="w-full"
                            min={1}
                            step={1}
                            value={Number(interval)}
                            onChange={(value) => setIntervalValue(String(value ?? ""))}
                        />
                    </div>
                    <div>
                        <label htmlFor={check ? `check-timeout-${check.id}` : "check-timeout"}>
                            {t("超时毫秒数", "Timeout (milliseconds)")}
                        </label>
                        <InputNumber
                            id={check ? `check-timeout-${check.id}` : "check-timeout"}
                            className="w-full"
                            min={1}
                            step={100}
                            value={Number(timeout)}
                            onChange={(value) => setTimeoutValue(String(value ?? ""))}
                        />
                    </div>
                    <div>
                        <label htmlFor={check ? `check-threshold-${check.id}` : "check-threshold"}>
                            {t("失败阈值", "Failure threshold")}
                        </label>
                        <InputNumber
                            id={check ? `check-threshold-${check.id}` : "check-threshold"}
                            className="w-full"
                            min={1}
                            step={1}
                            value={Number(threshold)}
                            onChange={(value) => setThreshold(String(value ?? ""))}
                        />
                    </div>
                </div>
            </Modal>
        </>
    );
}
