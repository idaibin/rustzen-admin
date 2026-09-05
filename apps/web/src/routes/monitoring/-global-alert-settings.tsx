import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Card, Form, InputNumber, Switch, Typography } from "antd";
import { useEffect, useState } from "react";

import { appMessage, monitorAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import {
    failedNetworkAction,
    retryFailedNetworkAction,
    type FailedNetworkAction,
} from "./-save-state";

export function GlobalAlertSettings() {
    const [form] = Form.useForm<Monitor.UpdateAlertSettings>();
    const [failedSave, setFailedSave] =
        useState<FailedNetworkAction<Monitor.UpdateAlertSettings>>();
    const client = useQueryClient();
    const canManage = useAuthStore((state) => state.checkPermissions("monitor:manage"));
    const { data, isPending, refetch } = useQuery({
        queryKey: ["monitor", "alert-settings"],
        queryFn: monitorAPI.alertSettings,
    });
    useEffect(() => {
        if (!canManage) setFailedSave(undefined);
    }, [canManage]);
    const mutation = useMutation({
        mutationFn: monitorAPI.updateAlertSettings,
        onMutate: () => setFailedSave(undefined),
        onSuccess: async () => {
            await client.invalidateQueries({ queryKey: ["monitor"] });
            appMessage.success(t("告警设置已保存", "Alert settings saved"));
        },
        onError: (error, values) =>
            setFailedSave(failedNetworkAction(error, { type: "save", values })),
    });
    if (!data)
        return (
            <div className="space-y-5">
                <DataState
                    kind={isPending ? "loading" : "error"}
                    title={
                        isPending
                            ? t("正在加载告警设置", "Loading alert settings")
                            : t("告警设置加载失败", "Failed to load alert settings")
                    }
                    action={
                        !isPending ? (
                            <Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>
                        ) : undefined
                    }
                />
            </div>
        );
    return (
        <div className="space-y-5">
            <Typography.Paragraph type="secondary">
                {t(
                    "统一设置 CPU、内存、磁盘和离线告警。修改后立即影响使用全局策略的节点；节点自定义策略优先。",
                    "Configure CPU, memory, disk and offline alerts together. Changes apply to inheriting nodes; custom policies take priority.",
                )}
            </Typography.Paragraph>
            {canManage && failedSave ? (
                <Alert
                    data-testid="monitor-global-save-error"
                    type="error"
                    showIcon
                    title={t("告警设置未保存", "Alert settings were not saved")}
                    description={t(
                        "无法连接监控服务。当前修改仍保留，可重试保存。",
                        "The monitoring service could not be reached. Your changes are still here; retry saving.",
                    )}
                    action={
                        <Button
                            data-testid="monitor-global-save-retry"
                            onClick={() =>
                                retryFailedNetworkAction(canManage, failedSave, {
                                    save: (values) => mutation.mutate(values),
                                    reset: () => undefined,
                                })
                            }
                        >
                            {t("重试", "Retry")}
                        </Button>
                    }
                />
            ) : null}
            <Form
                form={form}
                initialValues={{
                    cpu: data.cpu,
                    memory: data.memory,
                    disk: data.disk,
                    offline: data.offline,
                }}
                layout="vertical"
                className="flex flex-col gap-5"
                onFinish={(values) => {
                    if (canManage && !mutation.isPending) mutation.mutate(values);
                }}
                disabled={!canManage || mutation.isPending}
            >
                <Card size="small" title={t("告警策略", "Alert policy")}>
                    <div className="grid gap-4 sm:grid-cols-2">
                        <Threshold name="cpu" label="CPU" />
                        <Threshold name="memory" label={t("内存", "Memory")} />
                        <Threshold name="disk" label={t("磁盘", "Disk")} />
                        <Form.Item className="!mb-0" label={t("离线告警", "Offline alert")}>
                            <div className="flex flex-wrap items-center gap-4">
                                <Form.Item
                                    name={["offline", "enabled"]}
                                    valuePropName="checked"
                                    noStyle
                                >
                                    <Switch
                                        aria-label={t("启用离线告警", "Enable offline alert")}
                                    />
                                </Form.Item>
                                <Form.Item
                                    name={["offline", "afterSeconds"]}
                                    noStyle
                                    rules={[
                                        {
                                            required: true,
                                            message: t(
                                                "请输入离线告警时长",
                                                "Enter the offline alert duration",
                                            ),
                                        },
                                        {
                                            type: "number",
                                            min: 30,
                                            max: 3600,
                                            message: t(
                                                "离线告警时长范围为 30–3600 秒",
                                                "Offline alert duration must be 30–3600 seconds",
                                            ),
                                        },
                                    ]}
                                >
                                    <InputNumber
                                        variant="outlined"
                                        className="w-40"
                                        aria-label={t(
                                            "离线时长（秒）",
                                            "Offline duration (seconds)",
                                        )}
                                        min={30}
                                        max={3600}
                                        suffix="s"
                                    />
                                </Form.Item>
                            </div>
                        </Form.Item>
                    </div>
                </Card>
                <div className="flex flex-wrap items-center justify-between gap-4">
                    <Typography.Text type="secondary" className="text-xs">
                        {t("最近更新", "Last updated")}: {formatDateTime(data.updatedAt)}
                    </Typography.Text>
                    {canManage ? (
                        <Button
                            data-testid="monitor-global-save"
                            type="primary"
                            htmlType="submit"
                            loading={mutation.isPending}
                        >
                            {t("保存", "Save")}
                        </Button>
                    ) : null}
                </div>
            </Form>
        </div>
    );
}

function Threshold({ name, label }: { name: "cpu" | "memory" | "disk"; label: string }) {
    return (
        <Form.Item className="!mb-0" label={`${label} ${t("告警", "alert")}`}>
            <div className="flex flex-wrap items-center gap-3">
                <Form.Item name={[name, "enabled"]} valuePropName="checked" noStyle>
                    <Switch aria-label={`${label} ${t("启用告警", "Enable alert")}`} />
                </Form.Item>
                <Form.Item
                    name={[name, "thresholdPercent"]}
                    noStyle
                    rules={[
                        {
                            required: true,
                            message: t(
                                `请输入${label}告警阈值`,
                                `Enter the ${label} alert threshold`,
                            ),
                        },
                        {
                            type: "number",
                            min: 1,
                            max: 100,
                            message: t(
                                `${label}告警阈值范围为 1–100%`,
                                `${label} alert threshold must be 1–100%`,
                            ),
                        },
                    ]}
                >
                    <InputNumber
                        variant="outlined"
                        className="w-28"
                        aria-label={`${label} ${t("阈值（%）", "Threshold (%)")}`}
                        min={1}
                        max={100}
                        suffix="%"
                    />
                </Form.Item>
            </div>
        </Form.Item>
    );
}
