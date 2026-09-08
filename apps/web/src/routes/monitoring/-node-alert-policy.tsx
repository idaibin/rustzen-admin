import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Alert, Button, Card, Form, InputNumber, Space, Switch, Tag, Typography } from "antd";
import { useEffect, useRef, useState } from "react";

import { appMessage, monitorAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import {
    failedNetworkAction,
    retryFailedNetworkAction,
    shouldHydrateNodePolicy,
    type FailedNetworkAction,
} from "./-save-state";

export function NodeAlertPolicySourceTag({ source }: { source: "global" | "custom" }) {
    return source === "custom" ? (
        <Tag data-testid="monitor-node-policy-source" color="blue">
            {t("节点自定义", "Custom")}
        </Tag>
    ) : (
        <Tag data-testid="monitor-node-policy-source">{t("全局默认", "Global default")}</Tag>
    );
}

export function NodeAlertPolicy({ nodeId }: { nodeId: string }) {
    const [form] = Form.useForm<Monitor.UpdateAlertSettings>();
    const [failedAction, setFailedAction] =
        useState<FailedNetworkAction<Monitor.UpdateAlertSettings>>();
    const hydratedNodeId = useRef<string | undefined>(undefined);
    const client = useQueryClient();
    const canManage = useAuthStore((state) => state.checkPermissions("monitor:manage"));
    const { data, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "node-alert-settings", nodeId],
        queryFn: () => monitorAPI.nodeAlertSettings(nodeId),
    });
    useEffect(() => {
        if (
            data &&
            shouldHydrateNodePolicy(hydratedNodeId.current !== nodeId, form.isFieldsTouched())
        ) {
            form.setFieldsValue({
                cpu: data.cpu,
                memory: data.memory,
                disk: data.disk,
                offline: data.offline,
            });
            hydratedNodeId.current = nodeId;
        }
    }, [data, form, nodeId]);
    useEffect(() => {
        if (!canManage) setFailedAction(undefined);
    }, [canManage]);
    const save = useMutation({
        mutationFn: (values: Monitor.UpdateAlertSettings) =>
            monitorAPI.updateNodeAlertSettings(nodeId, values),
        onMutate: () => setFailedAction(undefined),
        onSuccess: async (value) => {
            form.setFieldsValue(value);
            await Promise.all([
                client.invalidateQueries({ queryKey: ["monitor", "node-alert-settings", nodeId] }),
                client.invalidateQueries({ queryKey: ["monitor", "nodes"] }),
            ]);
            appMessage.success(t("节点告警策略已保存", "Node alert policy saved"));
        },
        onError: (error, values) =>
            setFailedAction(failedNetworkAction(error, { type: "save", values })),
    });
    const reset = useMutation({
        mutationFn: () => monitorAPI.resetNodeAlertSettings(nodeId),
        onMutate: () => setFailedAction(undefined),
        onSuccess: async (value) => {
            form.setFieldsValue(value);
            await Promise.all([
                client.invalidateQueries({ queryKey: ["monitor", "node-alert-settings", nodeId] }),
                client.invalidateQueries({ queryKey: ["monitor", "nodes"] }),
            ]);
            appMessage.success(t("已恢复全局默认策略", "Global defaults restored"));
        },
        onError: (error) => setFailedAction(failedNetworkAction(error, { type: "reset" })),
    });
    const busy = save.isPending || reset.isPending;
    if (!data) {
        return (
            <Card size="small" title={t("告警策略", "Alert policy")}>
                <DataState
                    kind={isPending ? "loading" : "error"}
                    title={
                        isPending
                            ? t("正在加载策略", "Loading policy")
                            : t("策略加载失败", "Failed to load policy")
                    }
                    action={
                        error ? (
                            <Button onClick={() => void refetch()}>{t("重试", "Retry")}</Button>
                        ) : undefined
                    }
                    compact
                />
            </Card>
        );
    }
    return (
        <Card
            size="small"
            title={t("告警策略", "Alert policy")}
            extra={<NodeAlertPolicySourceTag source={data.source} />}
        >
            <Typography.Paragraph type="secondary">
                {data.isCustom
                    ? t(
                          "当前节点使用自定义策略，全局修改不会覆盖这里的设置。",
                          "This node uses a custom policy and ignores later global changes.",
                      )
                    : t(
                          "当前节点动态继承全局默认策略。",
                          "This node dynamically inherits the global defaults.",
                      )}
            </Typography.Paragraph>
            {canManage && failedAction ? (
                <Alert
                    data-testid="monitor-node-save-error"
                    className="mb-4"
                    type="error"
                    showIcon
                    title={t("节点策略未保存", "Node policy was not saved")}
                    description={t(
                        "无法连接监控服务。当前策略仍保留，可重试操作。",
                        "The monitoring service could not be reached. The current policy is still here; retry the action.",
                    )}
                    action={
                        <Button
                            data-testid="monitor-node-save-retry"
                            onClick={() =>
                                retryFailedNetworkAction(canManage, failedAction, {
                                    save: (values) => save.mutate(values),
                                    reset: () => reset.mutate(),
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
                layout="vertical"
                disabled={!canManage || busy}
                onFinish={(values) => {
                    if (canManage && !busy) save.mutate(values);
                }}
            >
                <PolicyThreshold name="cpu" label="CPU" />
                <PolicyThreshold name="memory" label={t("内存", "Memory")} />
                <PolicyThreshold name="disk" label={t("磁盘", "Disk")} />
                <Form.Item label={t("离线告警", "Offline alert")}>
                    <div className="flex items-center gap-4">
                        <Form.Item name={["offline", "enabled"]} valuePropName="checked" noStyle>
                            <Switch aria-label={t("启用离线告警", "Enable offline alert")} />
                        </Form.Item>
                        <Form.Item
                            name={["offline", "afterSeconds"]}
                            noStyle
                            rules={[{ required: true }, { type: "number", min: 30, max: 3600 }]}
                        >
                            <InputNumber
                                aria-label={t("离线时长（秒）", "Offline duration (seconds)")}
                                min={30}
                                max={3600}
                                suffix="s"
                            />
                        </Form.Item>
                    </div>
                </Form.Item>
                {canManage ? (
                    <Space>
                        <Button
                            data-testid="monitor-node-save"
                            type="primary"
                            htmlType="submit"
                            loading={save.isPending}
                            disabled={busy}
                        >
                            {t("保存为节点策略", "Save node policy")}
                        </Button>
                        {data.isCustom ? (
                            <Button
                                data-testid="monitor-node-reset"
                                loading={reset.isPending}
                                disabled={busy}
                                onClick={() => {
                                    if (canManage && !busy) reset.mutate();
                                }}
                            >
                                {t("重置为全局默认", "Reset to global defaults")}
                            </Button>
                        ) : null}
                    </Space>
                ) : null}
            </Form>
        </Card>
    );
}

function PolicyThreshold({ name, label }: { name: "cpu" | "memory" | "disk"; label: string }) {
    return (
        <Form.Item label={`${label} ${t("告警", "alert")}`}>
            <div className="flex items-center gap-4">
                <Form.Item name={[name, "enabled"]} valuePropName="checked" noStyle>
                    <Switch aria-label={`${label} ${t("启用告警", "Enable alert")}`} />
                </Form.Item>
                <Form.Item
                    name={[name, "thresholdPercent"]}
                    noStyle
                    rules={[{ required: true }, { type: "number", min: 1, max: 100 }]}
                >
                    <InputNumber
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
