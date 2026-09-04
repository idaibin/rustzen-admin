import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
    Button,
    Card,
    Drawer,
    Form,
    InputNumber,
    Progress,
    Space,
    Switch,
    Tag,
    Typography,
} from "antd";
import { useEffect } from "react";
import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import { appMessage, monitorAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export function NodeDetails({ node, onClose }: { node?: Monitor.Node; onClose: () => void }) {
    const { data, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "metrics", node?.nodeId],
        queryFn: () => monitorAPI.metrics(node!.nodeId, { bucket: "5m" }),
        enabled: Boolean(node),
    });
    return (
        <Drawer
            open={Boolean(node)}
            onClose={onClose}
            title={node?.hostname ?? t("节点详情", "Node details")}
            size="large"
            styles={{ wrapper: { maxWidth: "100vw" }, body: { overflowY: "auto" } }}
            destroyOnHidden
        >
            {node ? (
                <Space orientation="vertical" className="w-full" size="middle">
                    <Typography.Text type="secondary">
                        {node.nodeId} · v{node.agentVersion} · {formatDateTime(node.lastReportAt)}
                    </Typography.Text>
                    <NodeAlertPolicy nodeId={node.nodeId} />
                    <div className="grid gap-3 md:grid-cols-3">
                        <Usage usage={node.memory} label={t("内存", "Memory")} />
                        <Card size="small">
                            <Typography.Text type="secondary">CPU</Typography.Text>
                            <div className="text-xl font-semibold">
                                {node.cpuPercent.toFixed(1)}%
                            </div>
                        </Card>
                        <Card size="small">
                            <Typography.Text type="secondary">
                                {t("磁盘挂载点", "Disk mounts")}
                            </Typography.Text>
                            {node.disks.map((disk) => (
                                <div key={disk.mountPoint}>
                                    {disk.mountPoint}: {disk.usagePercent.toFixed(1)}%
                                </div>
                            ))}
                        </Card>
                    </div>
                    <Card className="h-80">
                        {isPending ? (
                            <DataState
                                kind="loading"
                                title={t("正在加载指标", "Loading metrics")}
                                compact
                            />
                        ) : error ? (
                            <DataState
                                kind="error"
                                title={t("指标加载失败", "Failed to load metrics")}
                                action={
                                    <Button onClick={() => void refetch()}>
                                        {t("重试", "Retry")}
                                    </Button>
                                }
                                compact
                            />
                        ) : data?.points.length ? (
                            <ResponsiveContainer width="100%" height="100%">
                                <LineChart data={data.points}>
                                    <XAxis
                                        dataKey="collectedAt"
                                        tickFormatter={(value) =>
                                            new Date(String(value)).toLocaleTimeString()
                                        }
                                    />
                                    <YAxis domain={[0, 100]} />
                                    <Tooltip />
                                    <Line
                                        type="monotone"
                                        dataKey="cpuPercent"
                                        name="CPU %"
                                        stroke="var(--chart-1)"
                                        dot={false}
                                    />
                                    <Line
                                        type="monotone"
                                        dataKey="memoryPercent"
                                        name={t("内存 %", "Memory %")}
                                        stroke="var(--chart-2)"
                                        dot={false}
                                    />
                                </LineChart>
                            </ResponsiveContainer>
                        ) : (
                            <DataState
                                kind="empty"
                                title={t("最近 24 小时暂无指标", "No metrics in the last 24 hours")}
                                compact
                            />
                        )}
                    </Card>
                    {data?.disks.map((series) => (
                        <Card key={series.mountPoint} size="small" title={series.mountPoint}>
                            {series.points.map((point) => (
                                <div key={point.collectedAt}>
                                    {formatDateTime(point.collectedAt)} · {point.percent.toFixed(1)}
                                    %
                                </div>
                            ))}
                        </Card>
                    ))}
                </Space>
            ) : null}
        </Drawer>
    );
}

export function PolicySourceTag({ source }: { source: "global" | "custom" }) {
    return source === "custom" ? (
        <Tag color="blue">{t("节点自定义", "Custom")}</Tag>
    ) : (
        <Tag>{t("全局默认", "Global default")}</Tag>
    );
}

function NodeAlertPolicy({ nodeId }: { nodeId: string }) {
    const [form] = Form.useForm<Monitor.UpdateAlertSettings>();
    const client = useQueryClient();
    const canManage = useAuthStore((state) => state.checkPermissions("monitor:manage"));
    const { data, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "node-alert-settings", nodeId],
        queryFn: () => monitorAPI.nodeAlertSettings(nodeId),
    });
    useEffect(() => {
        if (data) {
            form.setFieldsValue({
                cpu: data.cpu,
                memory: data.memory,
                disk: data.disk,
                offline: data.offline,
            });
        }
    }, [data, form]);
    const save = useMutation({
        mutationFn: (values: Monitor.UpdateAlertSettings) =>
            monitorAPI.updateNodeAlertSettings(nodeId, values),
        onSuccess: async (value) => {
            form.setFieldsValue(value);
            await Promise.all([
                client.invalidateQueries({ queryKey: ["monitor", "node-alert-settings", nodeId] }),
                client.invalidateQueries({ queryKey: ["monitor", "nodes"] }),
            ]);
            appMessage.success(t("节点告警策略已保存", "Node alert policy saved"));
        },
    });
    const reset = useMutation({
        mutationFn: () => monitorAPI.resetNodeAlertSettings(nodeId),
        onSuccess: async (value) => {
            form.setFieldsValue(value);
            await Promise.all([
                client.invalidateQueries({ queryKey: ["monitor", "node-alert-settings", nodeId] }),
                client.invalidateQueries({ queryKey: ["monitor", "nodes"] }),
            ]);
            appMessage.success(t("已恢复全局默认策略", "Global defaults restored"));
        },
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
            extra={<PolicySourceTag source={data.source} />}
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
            <Form
                form={form}
                layout="vertical"
                disabled={!canManage || busy}
                onFinish={(values) => {
                    if (!busy) save.mutate(values);
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
                        <Button type="primary" htmlType="submit" loading={save.isPending} disabled={busy}>
                            {t("保存为节点策略", "Save node policy")}
                        </Button>
                        {data.isCustom ? (
                            <Button loading={reset.isPending} disabled={busy} onClick={() => {
                                if (!busy) reset.mutate();
                            }}>
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

function Usage({ usage, label }: { usage: Monitor.Usage; label?: string }) {
    return (
        <Card size="small" className="min-w-32">
            {label ? <Typography.Text type="secondary">{label}</Typography.Text> : null}
            <div className="text-sm">
                {formatBytes(usage.usedBytes)} / {formatBytes(usage.totalBytes)}
            </div>
            <Progress percent={usage.usagePercent} size="small" />
        </Card>
    );
}

function formatBytes(bytes: number) {
    const units = ["B", "KB", "MB", "GB", "TB"];
    let value = bytes;
    let index = 0;
    while (value >= 1024 && index < units.length - 1) {
        value /= 1024;
        index += 1;
    }
    return `${value.toFixed(1)} ${units[index]}`;
}
