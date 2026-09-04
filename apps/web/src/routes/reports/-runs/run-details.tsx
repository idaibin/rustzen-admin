import { type ProColumns, ProTable } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { Button, Modal, Tag } from "antd";
import { useMemo } from "react";

import { reportsAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { LiveFrame } from "./live-frame";
import { getRunStatusMeta, getStepStatusMeta, isActiveRun } from "./status";

export function RunDetails({ run, onClose }: { run?: Reports.Run; onClose: () => void }) {
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
        refetchInterval: (query) => (isActiveRun(query.state.data?.status) ? 1000 : false),
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
    const runStatusMeta = useMemo(getRunStatusMeta, [locale]);
    const stepStatusMeta = useMemo(getStepStatusMeta, [locale]);
    const stepColumns: ProColumns<Reports.RunStep>[] = [
        {
            title: t("步骤", "Step"),
            dataIndex: "stepIndex",
            key: "stepIndex",
            width: 88,
            render: (_: unknown, row) => `${row.stepIndex + 1}. ${row.action}`,
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 120,
            render: (_: unknown, row) => {
                const meta = stepStatusMeta[row.status as keyof typeof stepStatusMeta];
                return <Tag color={meta?.color ?? "default"}>{meta?.label ?? row.status}</Tag>;
            },
        },
        {
            title: t("耗时", "Duration"),
            key: "durationMs",
            width: 110,
            render: (_: unknown, row) => `${row.durationMs ?? 0} ms`,
        },
        {
            title: t("消息", "Message"),
            dataIndex: "message",
            key: "message",
            render: (_: unknown, row) => row.message || "-",
        },
    ];
    const artifactColumns: ProColumns<Reports.Artifact>[] = [
        {
            title: t("文件", "File"),
            dataIndex: "fileName",
            key: "fileName",
            render: (_: unknown, row) => (
                <Button
                    type="link"
                    onClick={() =>
                        void reportsAPI.downloadArtifact(row.runId, row.id, row.fileName)
                    }
                >
                    {row.fileName}
                </Button>
            ),
        },
        { title: t("类型", "Kind"), dataIndex: "kind", key: "kind" },
        {
            title: t("创建时间", "Created at"),
            dataIndex: "createdAt",
            key: "createdAt",
            render: (_: unknown, row) => formatDateTime(row.createdAt),
        },
    ];

    return (
        <Modal
            open={Boolean(run)}
            onCancel={onClose}
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
            {run ? (
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
            ) : null}
            <div className="mb-5">
                <LiveFrame run={currentRun} />
            </div>
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
