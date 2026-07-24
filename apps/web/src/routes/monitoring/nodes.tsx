import {
    CheckCircleOutlined,
    DisconnectOutlined,
    PlusOutlined,
    WarningOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Drawer, Progress, Space, Tag, Typography } from "antd";
import { useState } from "react";
import {
    CartesianGrid,
    Legend,
    Line,
    LineChart,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import { monitorAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/monitoring/nodes")({ component: MonitoringNodesPage });

const statusColorMap = {
    online: "success",
    offline: "error",
    unknown: "default",
} as const;

function statusLabel(status: string) {
    if (status === "online") {
        return t("在线", "Online");
    }
    if (status === "offline") {
        return t("离线", "Offline");
    }
    return status;
}

function statusIcon(status: string) {
    if (status === "online") {
        return <CheckCircleOutlined />;
    }
    if (status === "offline") {
        return <DisconnectOutlined />;
    }
    return <WarningOutlined />;
}

function MonitoringNodesPage() {
    const [selected, setSelected] = useState<Monitor.Node | null>(null);
    const { data, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["monitor", "nodes"],
        queryFn: monitorAPI.nodes,
        refetchInterval: 30_000,
    });

    if (!data && isPending) {
        return (
            <PageCard
                title={t("节点", "Nodes")}
                description={t(
                    "查看每个已注册节点的最新心跳和资源快照。",
                    "View the latest heartbeat and resource snapshot for each registered node.",
                )}
                actions={<AddNodeDialog />}
            >
                <DataState kind="loading" title={t("正在加载节点", "Loading nodes")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                title={t("节点", "Nodes")}
                description={t(
                    "查看每个已注册节点的最新心跳和资源快照。",
                    "View the latest heartbeat and resource snapshot for each registered node.",
                )}
                actions={<AddNodeDialog />}
            >
                <DataState
                    kind="error"
                    title={t("节点加载失败", "Failed to load nodes")}
                    description={t(
                        "无法读取节点列表，请检查 Monitor 服务后重试。",
                        "Unable to read the node list. Check the Monitor service and try again.",
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

    const nodes = data ?? [];
    const columns: ProColumns<Monitor.Node>[] = [
        {
            title: t("节点", "Node"),
            dataIndex: "hostname",
            key: "hostname",
            render: (_: unknown, node: Monitor.Node) => (
                <div>
                    <div className="font-medium">{node.hostname}</div>
                    <div className="text-xs text-muted-foreground">
                        {node.agentId} · v{node.agentVersion}
                    </div>
                </div>
            ),
        },
        {
            title: t("状态", "Status"),
            dataIndex: "status",
            key: "status",
            render: (_: unknown, node: Monitor.Node) => (
                <Tag
                    icon={statusIcon(node.status)}
                    color={statusColorMap[node.status as keyof typeof statusColorMap] ?? "default"}
                >
                    {statusLabel(node.status)}
                </Tag>
            ),
        },
        {
            title: "CPU",
            dataIndex: "cpuPercent",
            key: "cpuPercent",
            render: (_: unknown, node: Monitor.Node) => formatPercent(node.cpuPercent),
        },
        {
            title: t("内存", "Memory"),
            dataIndex: "memoryUsedBytes",
            key: "memoryUsedBytes",
            render: (_: unknown, node: Monitor.Node) => (
                <Usage used={node.memoryUsedBytes} total={node.memoryTotalBytes} />
            ),
        },
        {
            title: t("磁盘", "Disk"),
            dataIndex: "diskUsedBytes",
            key: "diskUsedBytes",
            render: (_: unknown, node: Monitor.Node) => (
                <Usage used={node.diskUsedBytes} total={node.diskTotalBytes} />
            ),
        },
        {
            title: t("最后在线", "Last seen"),
            dataIndex: "lastSeenAt",
            key: "lastSeenAt",
            render: (_: unknown, node: Monitor.Node) => formatDateTime(node.lastSeenAt),
        },
        {
            title: t("详情", "Details"),
            key: "actions",
            fixed: "right",
            render: (_: unknown, node: Monitor.Node) => (
                <Space>
                    <Button size="small" onClick={() => setSelected(node)}>
                        {t("查看", "View")}
                    </Button>
                </Space>
            ),
        },
    ];

    return (
        <PageCard
            title={t("节点", "Nodes")}
            description={t(
                "查看每个已注册节点的最新心跳和资源快照。",
                "View the latest heartbeat and resource snapshot for each registered node.",
            )}
            actions={<AddNodeDialog />}
        >
            <ProTable<Monitor.Node>
                rowKey="id"
                columns={columns}
                dataSource={nodes}
                loading={isFetching}
                search={false}
                options={false}
                pagination={false}
                locale={{
                    emptyText:
                        nodes.length === 0 ? (
                            <DataState
                                kind="empty"
                                title={t("暂无监控节点", "No monitored nodes")}
                                description={t(
                                    "启动节点 Agent 后，首次心跳会自动完成注册。",
                                    "Start the node agent. Its first heartbeat will register it automatically.",
                                )}
                            />
                        ) : undefined,
                }}
            />
            <NodeDetails node={selected} onOpenChange={(open) => !open && setSelected(null)} />
        </PageCard>
    );
}

function AddNodeDialog() {
    const [open, setOpen] = useState(false);

    return (
        <>
            <Button type="primary" icon={<PlusOutlined />} onClick={() => setOpen(true)}>
                {t("添加节点", "Add node")}
            </Button>
            <Drawer
                open={open}
                onClose={() => setOpen(false)}
                title={t("添加监控节点", "Add monitored node")}
                footer={null}
            >
                <div className="space-y-3 text-sm">
                    <Typography.Paragraph>
                        {t(
                            "在节点上启动随包提供的 Agent；首次心跳通过后，节点会自动加入列表。",
                            "Start the bundled agent on the node. It will join the list after its first heartbeat.",
                        )}
                    </Typography.Paragraph>
                    <p>
                        {t(
                            "配置控制器地址，并使用与 Monitor 服务一致的环境变量：",
                            "Configure the controller address and use the same environment variable as the Monitor service:",
                        )}
                        <code className="mx-1 rounded bg-muted px-1 py-0.5">
                            RUSTZEN_MONITOR_AGENT_TOKEN
                        </code>
                    </p>
                    <pre className="overflow-x-auto rounded-md bg-muted p-3 text-xs">
                        rz-monitor agent
                    </pre>
                    <Typography.Text type="secondary">
                        {t(
                            "节点 ID 由 Agent 主机名生成；后续心跳会更新现有记录，不会重复创建节点。",
                            "The node ID is generated from the agent hostname. Later heartbeats update the existing record instead of creating duplicates.",
                        )}
                    </Typography.Text>
                </div>
            </Drawer>
        </>
    );
}

function NodeDetails({
    node,
    onOpenChange,
}: {
    node: Monitor.Node | null;
    onOpenChange: (open: boolean) => void;
}) {
    const {
        data: metrics = [],
        error,
        isPending,
        isFetching,
        refetch,
    } = useQuery({
        queryKey: ["monitor", "nodes", node?.id, "metrics", "5m"],
        queryFn: () => monitorAPI.metrics(node?.id ?? "", { bucket: "5m" }),
        enabled: Boolean(node),
    });

    return (
        <Drawer
            open={Boolean(node)}
            onClose={() => onOpenChange(false)}
            width={960}
            title={node?.hostname ?? t("节点详情", "Node details")}
            destroyOnHidden
            footer={null}
        >
            <Typography.Paragraph type="secondary">
                {node ? `${node.agentId} · Agent ${node.agentVersion}` : ""}
            </Typography.Paragraph>
            <div className="grid grid-cols-2 gap-3 text-sm md:grid-cols-4">
                <Summary label={t("状态", "Status")} value={node?.status ?? "-"} />
                <Summary label="CPU" value={formatPercent(node?.cpuPercent ?? null)} />
                <Summary
                    label={t("内存", "Memory")}
                    value={
                        node?.memoryUsedBytes === null || node?.memoryUsedBytes === undefined
                            ? "-"
                            : formatBytes(node.memoryUsedBytes)
                    }
                />
                <Summary
                    label={t("磁盘", "Disk")}
                    value={
                        node?.diskUsedBytes === null || node?.diskUsedBytes === undefined
                            ? "-"
                            : formatBytes(node.diskUsedBytes)
                    }
                />
            </div>
            <div className="mt-4 h-80 rounded-lg border p-3">
                {isPending ? (
                    <DataState
                        kind="loading"
                        title={t("正在加载指标", "Loading metrics")}
                        compact
                        className="h-full min-h-0"
                    />
                ) : error && metrics.length === 0 ? (
                    <DataState
                        kind="error"
                        title={t("指标加载失败", "Failed to load metrics")}
                        action={
                            <Button
                                size="small"
                                onClick={() => void refetch()}
                                loading={isFetching}
                            >
                                {t("重新加载", "Reload")}
                            </Button>
                        }
                        compact
                        className="h-full min-h-0"
                    />
                ) : metrics.length ? (
                    <ResponsiveContainer width="100%" height="100%">
                        <LineChart data={metrics}>
                            <CartesianGrid strokeDasharray="3 3" vertical={false} />
                            <XAxis
                                dataKey="collectedAt"
                                tickFormatter={(value) =>
                                    new Date(String(value)).toLocaleTimeString()
                                }
                            />
                            <YAxis domain={[0, 100]} />
                            <Tooltip />
                            <Legend />
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
                            <Line
                                type="monotone"
                                dataKey="diskPercent"
                                name={t("磁盘 %", "Disk %")}
                                stroke="var(--chart-3)"
                                dot={false}
                            />
                        </LineChart>
                    </ResponsiveContainer>
                ) : (
                    <DataState
                        kind="empty"
                        title={t("最近 24 小时暂无指标", "No metrics in the last 24 hours")}
                        compact
                        className="h-full min-h-0"
                    />
                )}
            </div>
        </Drawer>
    );
}

function Summary({ label, value }: { label: string; value: string }) {
    return (
        <div className="rounded-lg border p-3">
            <div className="text-xs text-muted-foreground">{label}</div>
            <div className="mt-1 font-medium">{value}</div>
        </div>
    );
}

function Usage({ used, total }: { used: number | null; total: number | null }) {
    if (used === null || total === null || total <= 0) return <>-</>;
    const percent = Math.min(100, Math.round((used / total) * 100));

    return (
        <div className="min-w-32">
            <div className="mb-1 text-xs text-muted-foreground">
                {formatBytes(used)} / {formatBytes(total)}
            </div>
            <Progress percent={percent} size="small" />
        </div>
    );
}

function formatPercent(value: number | null) {
    return value === null ? "-" : `${value.toFixed(1)}%`;
}

function formatBytes(bytes: number) {
    if (bytes < 1024) return `${bytes} B`;
    const units = ["KB", "MB", "GB", "TB"];
    let value = bytes / 1024;
    let index = 0;
    while (value >= 1024 && index < units.length - 1) {
        value /= 1024;
        index += 1;
    }
    return `${value.toFixed(1)} ${units[index]}`;
}
