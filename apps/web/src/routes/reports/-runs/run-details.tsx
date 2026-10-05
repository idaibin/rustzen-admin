import { type ProColumns, ProTable } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { Button, Modal, Tag } from "antd";
import { useMemo } from "react";

import { appMessage, reportsAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { displayTableProps, emptyTableLocale } from "@/components/table/table-presets";
import { useSubmission } from "@/hooks/use-submission";
import { formatDuration } from "@/lib/format";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { LiveFrame } from "./live-frame";
import { RetryRunButton } from "./retry-run-button";
import { getRunStatusMeta, getStepStatusMeta, isActiveRun } from "./status";

export function RunDetails({
    run,
    onClose,
    onRetried,
}: {
    run?: Reports.Run;
    onClose: () => void;
    onRetried: (run: Reports.Run) => void;
}) {
    const locale = useLocale();
    const {
        data: currentRun = run,
        error: runError,
        refetch: refetchRun,
        isFetching: runFetching,
    } = useQuery({
        queryKey: ["reports", "run", run?.id],
        queryFn: () => reportsAPI.run(run!.id),
        enabled: Boolean(run),
        retry: false,
        initialData: run,
        refetchInterval: (query) => (isActiveRun(query.state.data?.status) ? 1000 : false),
    });
    const {
        data: steps = [],
        error: stepsError,
        isPending: stepsPending,
        isFetching: stepsFetching,
        refetch: refetchSteps,
    } = useQuery({
        queryKey: ["reports", "run-steps", run?.id],
        queryFn: () => reportsAPI.runSteps(run!.id),
        enabled: Boolean(run),
        retry: false,
        refetchInterval: isActiveRun(currentRun?.status) ? 1000 : false,
    });
    const {
        data: artifacts = [],
        error: artifactsError,
        isPending: artifactsPending,
        isFetching: artifactsFetching,
        refetch: refetchArtifacts,
    } = useQuery({
        queryKey: ["reports", "run-artifacts", run?.id],
        queryFn: () => reportsAPI.runArtifacts(run!.id),
        enabled: Boolean(run),
        retry: false,
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
            render: (_: unknown, row) => formatDuration(row.durationMs),
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
            width: 280,
            render: (_: unknown, row) => <ArtifactDownloadButton artifact={row} />,
        },
        { title: t("类型", "Kind"), dataIndex: "kind", key: "kind", width: 120 },
        {
            title: t("创建时间", "Created at"),
            dataIndex: "createdAt",
            key: "createdAt",
            width: 180,
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
            <span hidden data-testid="run-audit" data-run-id={run?.id} />
            {runError ? (
                <DataState
                    kind="error"
                    title={t("执行状态加载失败", "Failed to load run status")}
                    description={t(
                        "实时刷新已暂停，请重新加载当前执行。",
                        "Live refresh is paused. Reload the current run.",
                    )}
                    action={
                        <Button
                            type="primary"
                            loading={runFetching}
                            onClick={() => void refetchRun()}
                        >
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
                    {currentRun?.error ? (
                        <p className="text-status-danger">{currentRun.error}</p>
                    ) : null}
                    {currentRun ? (
                        <RetryRunButton run={currentRun} onRetried={onRetried} surface="audit" />
                    ) : null}
                </div>
            ) : null}
            <div className="mb-5">
                <LiveFrame run={currentRun} />
            </div>
            <div className="mb-4">
                <div className="mb-2 flex items-center justify-between gap-2">
                    <h3 className="font-medium">{t("步骤", "Steps")}</h3>
                    <Button
                        type="text"
                        aria-label={t("重新加载步骤", "Reload steps")}
                        aria-busy={stepsFetching}
                        loading={stepsFetching}
                        onClick={() => void refetchSteps()}
                    >
                        {t("重新加载步骤", "Reload steps")}
                    </Button>
                </div>
                {stepsError ? (
                    <DataState
                        kind="error"
                        title={t("步骤加载失败", "Failed to load steps")}
                        description={
                            steps.length
                                ? t(
                                      "保留上次结果，请重新加载。",
                                      "Previous results are retained. Reload to refresh.",
                                  )
                                : undefined
                        }
                        compact
                    />
                ) : null}
                {!stepsError || steps.length > 0 ? (
                    <ProTable<Reports.RunStep>
                        rowKey="id"
                        columns={stepColumns}
                        dataSource={steps}
                        scroll={{ x: 480 }}
                        search={false}
                        options={false}
                        {...displayTableProps}
                        locale={{
                            emptyText:
                                steps.length === 0 ? (
                                    <DataState
                                        kind={
                                            stepsPending
                                                ? "loading"
                                                : isActiveRun(currentRun?.status)
                                                  ? "processing"
                                                  : "empty"
                                        }
                                        title={
                                            stepsPending
                                                ? t("正在加载步骤", "Loading steps")
                                                : isActiveRun(currentRun?.status)
                                                  ? t(
                                                        "正在等待步骤结果",
                                                        "Waiting for step results",
                                                    )
                                                  : t("暂无步骤记录", "No step records")
                                        }
                                        compact
                                    />
                                ) : undefined,
                        }}
                    />
                ) : null}
            </div>
            <div>
                <div className="mb-2 flex items-center justify-between gap-2">
                    <h3 className="font-medium">{t("产物", "Artifacts")}</h3>
                    <Button
                        type="text"
                        aria-label={t("重新加载产物", "Reload artifacts")}
                        aria-busy={artifactsFetching}
                        loading={artifactsFetching}
                        onClick={() => void refetchArtifacts()}
                    >
                        {t("重新加载产物", "Reload artifacts")}
                    </Button>
                </div>
                {artifactsError ? (
                    <DataState
                        kind="error"
                        title={t("产物加载失败", "Failed to load artifacts")}
                        description={
                            artifacts.length
                                ? t(
                                      "保留上次结果，请重新加载。",
                                      "Previous results are retained. Reload to refresh.",
                                  )
                                : undefined
                        }
                        compact
                    />
                ) : null}
                {!artifactsError || artifacts.length > 0 ? (
                    <ProTable<Reports.Artifact>
                        rowKey="id"
                        columns={artifactColumns}
                        dataSource={artifacts}
                        scroll={{ x: 580 }}
                        search={false}
                        options={false}
                        {...displayTableProps}
                        locale={{
                            emptyText: artifactsPending ? (
                                <DataState
                                    kind="loading"
                                    title={t("正在加载产物", "Loading artifacts")}
                                    compact
                                />
                            ) : (
                                emptyTableLocale(t("暂无产物", "No artifacts"), { compact: true })
                                    .emptyText
                            ),
                        }}
                    />
                ) : null}
            </div>
        </Modal>
    );
}

function ArtifactDownloadButton({ artifact }: { artifact: Reports.Artifact }) {
    const { submitting, beginSubmission, finishSubmission } = useSubmission();
    const download = async () => {
        if (!beginSubmission()) return;
        try {
            await reportsAPI.downloadArtifact(artifact.runId, artifact.id, artifact.fileName);
        } catch (error) {
            if (!(error instanceof Response))
                appMessage.error(t("下载失败，请重试。", "Download failed. Please retry."));
        } finally {
            finishSubmission();
        }
    };
    return (
        <Button
            type="link"
            style={{
                height: "auto",
                whiteSpace: "normal",
                overflowWrap: "anywhere",
                textAlign: "left",
            }}
            loading={submitting}
            aria-disabled={submitting}
            aria-busy={submitting}
            aria-label={artifact.fileName}
            onClick={() => void download()}
        >
            {artifact.fileName}
        </Button>
    );
}
