import { EyeOutlined, PlayCircleOutlined, StopOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Alert, Button, Card, Form, Input, Modal, Select, Tag, Typography } from "antd";
import { useEffect, useMemo, useState } from "react";

import { appMessage, reportsAPI } from "@/api";
import { reportsQueryOptions } from "@/api/reports/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

export const Route = createFileRoute("/reports/runs")({ component: RunsPage });

const size = 20;

const getRunStatusMeta = () =>
    ({
        queued: { label: t("排队中", "Queued"), color: "default" },
        running: { label: t("执行中", "Running"), color: "processing" },
        cancelling: { label: t("取消中", "Cancelling"), color: "warning" },
        succeeded: { label: t("已成功", "Succeeded"), color: "success" },
        failed: { label: t("失败", "Failed"), color: "error" },
        cancelled: { label: t("已取消", "Cancelled"), color: "warning" },
    }) satisfies Record<Reports.Run["status"], { label: string; color: string }>;

const getStepStatusMeta = () =>
    ({
        running: { label: t("执行中", "Running"), color: "processing" },
        succeeded: { label: t("已成功", "Succeeded"), color: "success" },
        failed: { label: t("失败", "Failed"), color: "error" },
        cancelled: { label: t("已取消", "Cancelled"), color: "warning" },
    }) satisfies Record<Reports.RunStep["status"], { label: string; color: string }>;

const defaultRunInput = JSON.stringify({ value: "" }, null, 2);
const isActiveRun = (status?: Reports.Run["status"]) =>
    status === "queued" || status === "running" || status === "cancelling";

function RunsPage() {
    const locale = useLocale();
    const [current, setCurrent] = useState(1);
    const [selected, setSelected] = useState<Reports.Run>();
    const client = useQueryClient();
    const { data: flows = [] } = useQuery(reportsQueryOptions.flows());
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["reports", "runs", current],
        queryFn: () => reportsAPI.runs({ current, pageSize: size }),
        refetchInterval: (q) =>
            q.state.data?.data.some((r) => isActiveRun(r.status)) ? 1000 : false,
    });
    const total = data?.total ?? 0;

    useEffect(() => {
        if (data === undefined || isFetching) {
            return;
        }
        const lastPage = Math.max(1, Math.ceil(total / size));
        if (current > lastPage) {
            setCurrent(lastPage);
        }
    }, [current, data, isFetching, total]);

    const cancel = useMutation({
        mutationFn: reportsAPI.cancelRun,
        onSuccess: async (run) => {
            await client.invalidateQueries({ queryKey: ["reports", "runs"] });
            appMessage.success(
                run.status === "cancelling"
                    ? t("正在停止填报执行", "Stopping report run")
                    : t("填报执行已取消", "Report run cancelled"),
            );
        },
    });
    const runStatusMeta = useMemo(() => getRunStatusMeta(), [locale]);

    const columns: ProColumns<Reports.Run>[] = useMemo(
        () => [
            {
                title: t("执行", "Run"),
                dataIndex: "id",
                key: "id",
                width: 150,
                render: (_: unknown, row: Reports.Run) => (
                    <span className="font-mono text-xs">{row.id.slice(0, 8)}</span>
                ),
            },
            {
                title: t("流程", "Template"),
                dataIndex: "flowId",
                key: "flow",
                width: 260,
                render: (_: unknown, row: Reports.Run) =>
                    flows.find((f) => f.id === row.flowId)?.name ?? row.flowId,
            },
            {
                title: t("状态", "Status"),
                dataIndex: "status",
                key: "status",
                width: 130,
                render: (_: unknown, row: Reports.Run) => {
                    const meta = runStatusMeta[row.status];
                    return <Tag color={meta.color}>{meta.label}</Tag>;
                },
            },
            {
                title: t("创建时间", "Created at"),
                dataIndex: "createdAt",
                key: "createdAt",
                width: 180,
                render: (_: unknown, row: Reports.Run) => formatDateTime(row.createdAt),
            },
            {
                title: t("错误", "Error"),
                dataIndex: "error",
                key: "error",
                ellipsis: true,
                render: (_: unknown, row: Reports.Run) => row.error ?? "-",
            },
            {
                title: t("操作", "Actions"),
                key: "actions",
                fixed: "right",
                width: 150,
                render: (_: unknown, row: Reports.Run) => (
                    <div className="flex items-center justify-end gap-1">
                        <Button
                            type="text"
                            icon={<EyeOutlined />}
                            aria-label={t("查看执行", "View run")}
                            onClick={() => setSelected(row)}
                        />
                        <AuthWrap code="reports:run:manage">
                            <Button
                                type="text"
                                icon={<StopOutlined />}
                                danger
                                aria-label={t("取消执行", "Cancel run")}
                                disabled={!(row.status === "queued" || row.status === "running")}
                                onClick={() => cancel.mutate(row.id)}
                            />
                        </AuthWrap>
                    </div>
                ),
            },
        ],
        [flows, cancel, locale],
    );

    if (!data?.data.length && isPending) {
        return (
            <PageCard
                title={t("填报执行", "Report runs")}
                description={t(
                    "通过报表模板写入所选数据，并实时查看执行过程。",
                    "Write selected data through a report template and monitor the run in real time.",
                )}
                actions={
                    <AuthWrap code="reports:run:manage">
                        <RunDialog flows={flows} />
                    </AuthWrap>
                }
            >
                <DataState kind="loading" title={t("正在加载填报执行", "Loading report runs")} />
            </PageCard>
        );
    }

    if (!data?.data.length && error) {
        return (
            <PageCard
                title={t("填报执行", "Report runs")}
                description={t(
                    "通过报表模板写入所选数据，并实时查看执行过程。",
                    "Write selected data through a report template and monitor the run in real time.",
                )}
                actions={
                    <AuthWrap code="reports:run:manage">
                        <RunDialog flows={flows} />
                    </AuthWrap>
                }
            >
                <DataState
                    kind="error"
                    title={t("填报执行加载失败", "Failed to load report runs")}
                    description={t(
                        "无法读取执行记录，请检查 Reports 服务后重试。",
                        "Unable to read run records. Check the Reports service and try again.",
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

    if (total === 0) {
        return (
            <PageCard
                title={t("填报执行", "Report runs")}
                description={t(
                    "通过报表模板写入所选数据，并实时查看执行过程。",
                    "Write selected data through a report template and monitor the run in real time.",
                )}
                actions={
                    <AuthWrap code="reports:run:manage">
                        <RunDialog flows={flows} />
                    </AuthWrap>
                }
            >
                <DataState
                    kind="empty"
                    title={t("暂无填报执行", "No report runs")}
                    description={""}
                />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("填报执行", "Report runs")}
            description={t(
                "通过报表模板写入所选数据，并实时查看执行过程。",
                "Write selected data through a report template and monitor the run in real time.",
            )}
            actions={
                <AuthWrap code="reports:run:manage">
                    <RunDialog flows={flows} />
                </AuthWrap>
            }
        >
            {error ? (
                <DataState
                    kind="error"
                    title={t("填报执行刷新失败", "Failed to refresh report runs")}
                    description={
                        error instanceof Error
                            ? error.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    compact
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            ) : null}
            <DataTableShell>
                <ProTable<Reports.Run>
                    rowKey="id"
                    columns={columns}
                    dataSource={data?.data ?? []}
                    loading={isFetching}
                    search={false}
                    options={false}
                    pagination={{
                        current,
                        pageSize: size,
                        total,
                        showSizeChanger: false,
                        onChange: (page) => setCurrent(page),
                    }}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText: (
                            <DataState kind="empty" title={t("暂无填报执行", "No report runs")} />
                        ),
                    }}
                />
            </DataTableShell>
            <RunDetails run={selected} onClose={() => setSelected(undefined)} />
        </PageCard>
    );
}

function RunDialog({ flows }: { flows: Reports.Flow[] }) {
    const client = useQueryClient();
    const [open, setOpen] = useState(false);
    const [flowId, setFlowId] = useState("");
    const [inputJson, setInputJson] = useState(defaultRunInput);
    const mutation = useMutation({
        mutationFn: reportsAPI.createRun,
        onSuccess: async () => {
            await client.invalidateQueries({ queryKey: ["reports", "runs"] });
            appMessage.success(t("填报执行已进入队列", "Report run queued"));
            setOpen(false);
        },
    });

    const save = () => {
        try {
            const input = JSON.parse(inputJson) as Record<string, unknown>;
            mutation.mutate({ flowId, input });
        } catch {
            appMessage.error(t("输入内容必须是有效的 JSON", "Input must be valid JSON"));
        }
    };

    const flowOptions = flows.map((f) => ({
        value: f.id,
        label: f.name,
    }));

    useEffect(() => {
        if (!open) {
            return;
        }
        if (!flows.length) {
            setFlowId("");
            return;
        }
        setFlowId((prev) => prev || flows[0]?.id || "");
    }, [open, flows]);

    return (
        <>
            <Button
                type="primary"
                disabled={!flows.length}
                icon={<PlayCircleOutlined />}
                onClick={() => setOpen(true)}
            >
                {t("新建填报", "New report run")}
            </Button>
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                footer={null}
                title={t("开始填报", "Start report run")}
                width={760}
                destroyOnHidden
            >
                <p className="mb-4 text-sm text-muted-foreground">
                    {t(
                        "选择已校验的流程，并填写本次写入使用的输入数据。",
                        "Select a verified template and enter the input data for this run.",
                    )}
                </p>
                <Alert
                    className="mb-4"
                    type="warning"
                    showIcon
                    title={t(
                        "不要提交密码、Token、密钥或其他敏感信息。",
                        "Do not submit passwords, tokens, keys, or other sensitive information.",
                    )}
                />
                <Form layout="vertical">
                    <Form.Item label={t("流程", "Template")}>
                        <Select
                            value={flowId || undefined}
                            onChange={(value) => setFlowId(value)}
                            options={flowOptions}
                            placeholder={t("选择模板", "Select template")}
                        />
                    </Form.Item>
                    <Form.Item label={t("输入 JSON", "Input JSON")}>
                        <Input.TextArea
                            className="font-mono"
                            rows={10}
                            value={inputJson}
                            onChange={(event) => setInputJson(event.target.value)}
                        />
                    </Form.Item>
                    <div className="flex justify-end gap-2">
                        <Button type="default" onClick={() => setOpen(false)}>
                            {t("取消", "Cancel")}
                        </Button>
                        <Button type="primary" loading={mutation.isPending} onClick={save}>
                            {t("提交执行", "Submit run")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function RunDetails({ run, onClose }: { run?: Reports.Run; onClose: () => void }) {
    const locale = useLocale();
    const {
        data: currentRun = run,
        error: runError,
        refetch: refetchRun,
    } = useQuery({
        queryKey: ["reports", "run", run?.id],
        queryFn: () => reportsAPI.run(run!.id),
        enabled: Boolean(run),
        initialData: run,
        refetchInterval: (query) => {
            const status = query.state.data?.status;
            return isActiveRun(status) ? 1000 : false;
        },
    });

    const { data: steps = [] } = useQuery({
        queryKey: ["reports", "run-steps", run?.id],
        queryFn: () => reportsAPI.runSteps(run!.id),
        enabled: Boolean(run),
        refetchInterval: isActiveRun(currentRun?.status) ? 1000 : false,
    });

    const { data: artifacts = [] } = useQuery({
        queryKey: ["reports", "run-artifacts", run?.id],
        queryFn: () => reportsAPI.runArtifacts(run!.id),
        enabled: Boolean(run),
        refetchInterval: isActiveRun(currentRun?.status) ? 1000 : false,
    });

    const runStatusMeta = useMemo(() => getRunStatusMeta(), [locale]);
    const stepStatusMeta = useMemo(() => getStepStatusMeta(), [locale]);

    const stepColumns: ProColumns<Reports.RunStep>[] = [
        {
            title: t("步骤", "Step"),
            dataIndex: "stepIndex",
            key: "stepIndex",
            width: 88,
            render: (_: unknown, row: Reports.RunStep) => `${row.stepIndex + 1}. ${row.action}`,
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 120,
            render: (_: unknown, row: Reports.RunStep) => {
                const meta = stepStatusMeta[row.status as keyof typeof stepStatusMeta];
                return <Tag color={meta?.color ?? "default"}>{meta?.label ?? row.status}</Tag>;
            },
        },
        {
            title: t("耗时", "Duration"),
            key: "durationMs",
            width: 110,
            render: (_: unknown, row: Reports.RunStep) => `${row.durationMs ?? 0} ms`,
        },
        {
            title: t("消息", "Message"),
            dataIndex: "message",
            key: "message",
            render: (_: unknown, row: Reports.RunStep) => row.message || "-",
        },
    ];

    const artifactColumns: ProColumns<Reports.Artifact>[] = [
        {
            title: t("文件", "File"),
            dataIndex: "fileName",
            key: "fileName",
            render: (_: unknown, row: Reports.Artifact) => (
                <Button
                    type="link"
                    onClick={() => {
                        void reportsAPI.downloadArtifact(row.runId, row.id, row.fileName);
                    }}
                >
                    {row.fileName}
                </Button>
            ),
        },
        {
            title: t("类型", "Kind"),
            dataIndex: "kind",
            key: "kind",
        },
        {
            title: t("创建时间", "Created at"),
            dataIndex: "createdAt",
            key: "createdAt",
            render: (_: unknown, row: Reports.Artifact) => formatDateTime(row.createdAt),
        },
    ];

    const liveFrame = <LiveFrame run={currentRun} />;

    return (
        <Modal
            open={Boolean(run)}
            onCancel={() => onClose()}
            footer={null}
            width={900}
            title={t("执行审计", "Run audit")}
        >
            {runError ? (
                <DataState
                    kind="error"
                    title={t("执行状态加载失败", "Failed to load run status")}
                    description={t(
                        "实时刷新已暂停，请重新加载当前执行。",
                        "Live refresh is paused. Reload the current run.",
                    )}
                    action={
                        <Button type="primary" onClick={() => void refetchRun()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                    compact
                />
            ) : isActiveRun(currentRun?.status) ? (
                <DataState
                    kind="processing"
                    title={
                        currentRun.status === "queued"
                            ? t("执行正在排队", "Run is queued")
                            : currentRun.status === "cancelling"
                              ? t("正在停止执行", "Stopping report run")
                              : t("填报正在执行", "Report run in progress")
                    }
                    description={t(
                        "页面会每秒刷新步骤、产物和实时画面。",
                        "Steps, artifacts, and the live view refresh every second.",
                    )}
                    compact
                />
            ) : null}
            {run && (
                <div className="mb-4 space-y-1 text-sm">
                    <p>
                        {t("状态：", "Status: ")}
                        {currentRun ? runStatusMeta[currentRun.status]?.label : "-"}
                    </p>
                    <p>
                        {t("开始时间：", "Started at: ")}
                        {formatDateTime(currentRun?.startedAt)}
                    </p>
                    <p>
                        {t("完成时间：", "Finished at: ")}
                        {formatDateTime(currentRun?.finishedAt)}
                    </p>
                    {currentRun?.error ? <p className="text-red-600">{currentRun.error}</p> : null}
                </div>
            )}

            <div className="mb-5">{liveFrame}</div>

            <div className="mb-4">
                <h3 className="mb-2 font-medium">{t("步骤", "Steps")}</h3>
                <ProTable<Reports.RunStep>
                    rowKey="id"
                    columns={stepColumns}
                    dataSource={steps}
                    search={false}
                    options={false}
                    pagination={false}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText:
                            steps.length === 0 ? (
                                <DataState
                                    kind={isActiveRun(currentRun?.status) ? "processing" : "empty"}
                                    title={
                                        isActiveRun(currentRun?.status)
                                            ? t("正在等待步骤结果", "Waiting for step results")
                                            : t("暂无步骤记录", "No step records")
                                    }
                                    compact
                                />
                            ) : undefined,
                    }}
                />
            </div>
            <div>
                <h3 className="mb-2 font-medium">{t("产物", "Artifacts")}</h3>
                <ProTable<Reports.Artifact>
                    rowKey="id"
                    columns={artifactColumns}
                    dataSource={artifacts}
                    search={false}
                    options={false}
                    pagination={false}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText: (
                            <DataState kind="empty" title={t("暂无产物", "No artifacts")} compact />
                        ),
                    }}
                />
            </div>
        </Modal>
    );
}

function LiveFrame({ run }: { run?: Reports.Run }) {
    const { data, error, refetch } = useQuery({
        queryKey: ["reports", "live-frame", run?.id],
        queryFn: ({ signal }) => reportsAPI.liveFrame(run!.id, signal),
        enabled: Boolean(run),
        refetchInterval: isActiveRun(run?.status) ? 1000 : false,
    });
    const [source, setSource] = useState<string>();

    useEffect(() => {
        setSource(undefined);
    }, [run?.id]);

    useEffect(() => {
        if (!data) {
            return;
        }
        const url = URL.createObjectURL(data);
        setSource(url);
        return () => URL.revokeObjectURL(url);
    }, [data]);

    return (
        <div>
            <Typography.Title level={5}>{t("实时画面", "Live view")}</Typography.Title>
            <Card className="h-80 overflow-auto">
                {source ? (
                    <img
                        src={source}
                        className="max-h-64 w-full object-contain"
                        alt={t("执行实时画面", "Live run view")}
                    />
                ) : error ? (
                    <DataState
                        kind="error"
                        title={t("实时画面加载失败", "Failed to load live view")}
                        action={
                            <Button type="primary" onClick={() => void refetch()}>
                                {t("重新加载", "Reload")}
                            </Button>
                        }
                        compact
                        className="h-full"
                    />
                ) : (
                    <DataState
                        kind={isActiveRun(run?.status) ? "processing" : "empty"}
                        title={
                            isActiveRun(run?.status)
                                ? t("正在等待浏览器画面", "Waiting for browser view")
                                : t("暂无实时画面", "No live view")
                        }
                        compact
                        className="h-full"
                    />
                )}
            </Card>
        </div>
    );
}
