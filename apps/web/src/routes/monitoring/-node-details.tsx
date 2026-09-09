import { useQuery } from "@tanstack/react-query";
import { Button, Card, Drawer, Progress, Space, Typography } from "antd";
import { Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from "recharts";

import { monitorAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import { NodeAlertPolicy } from "./-node-alert-policy";

export function NodeDetails({ node, onClose }: { node?: Monitor.Node; onClose: () => void }) {
    const { data, error, isPending, refetch } = useQuery({
        queryKey: ["monitor", "metrics", node?.nodeId],
        queryFn: () => monitorAPI.metrics(node!.nodeId, { bucket: "5m" }),
        enabled: Boolean(node),
    });
    return (
        <Drawer
            data-testid="monitor-node-details"
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
                    <Typography.Text
                        data-testid={`monitor-node-details-boot-id-${node.nodeId}`}
                        type="secondary"
                    >
                        {t("启动标识", "Boot ID")}: {node.bootId}
                    </Typography.Text>
                    <NodeAlertPolicy key={node.nodeId} nodeId={node.nodeId} />
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
                    <Card
                        className="h-80"
                        data-testid={`monitor-node-history-5m-${node.nodeId}`}
                        title={t("5 分钟聚合历史", "5-minute history")}
                    >
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
                            <div className="h-56 w-full">
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
                                            dot={data.points.length === 1}
                                        />
                                        <Line
                                            type="monotone"
                                            dataKey="memoryPercent"
                                            name={t("内存 %", "Memory %")}
                                            stroke="var(--chart-2)"
                                            dot={data.points.length === 1}
                                        />
                                        </LineChart>
                                </ResponsiveContainer>
                            </div>
                        ) : (
                            <DataState
                                kind="empty"
                                title={t("暂无 5 分钟聚合指标", "No 5-minute metrics available")}
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
