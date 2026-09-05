import {
    CheckCircleOutlined,
    DisconnectOutlined,
    PlusOutlined,
    ReloadOutlined,
    SettingOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Drawer, Tag } from "antd";
import { useState } from "react";

import { monitorAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { GlobalAlertSettings } from "./-global-alert-settings";
import { NodeDetails, PolicySourceTag } from "./-node-details";
import { NodeOnboarding } from "./-node-onboarding";
import { hasNodesBackgroundRefreshFailure } from "./-save-state";

export const Route = createFileRoute("/monitoring/nodes")({ component: MonitoringNodesPage });

function MonitoringNodesPage() {
    const [panel, setPanel] = useState<"add" | "settings">();
    const [selected, setSelected] = useState<Monitor.Node>();
    const canManage = useAuthStore((state) => state.checkPermissions("monitor:manage"));
    const canViewSettings = useAuthStore((state) => state.checkPermissions("monitor:node:view"));
    const { data, dataUpdatedAt, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["monitor", "nodes"],
        queryFn: monitorAPI.nodes,
        refetchInterval: 30_000,
    });
    const columns: ProColumns<Monitor.Node>[] = [
        {
            title: t("节点", "Node"),
            key: "node",
            render: (_, row) => (
                <div>
                    <div className="font-medium">{row.hostname}</div>
                    <div className="text-xs text-muted-foreground">
                        {row.nodeId} · v{row.agentVersion}
                    </div>
                </div>
            ),
        },
        {
            title: t("状态", "Status"),
            dataIndex: "status",
            width: 110,
            render: (_, row) => (
                <Tag
                    color={row.status === "online" ? "success" : "error"}
                    icon={
                        row.status === "online" ? <CheckCircleOutlined /> : <DisconnectOutlined />
                    }
                >
                    {row.status === "online" ? t("在线", "Online") : t("离线", "Offline")}
                </Tag>
            ),
        },
        {
            title: "CPU",
            dataIndex: "cpuPercent",
            width: 100,
            render: (_, row) => `${row.cpuPercent.toFixed(1)}%`,
        },
        {
            title: t("告警策略", "Alert policy"),
            dataIndex: "alertPolicySource",
            width: 110,
            render: (_, row) => <PolicySourceTag source={row.alertPolicySource} />,
        },
        {
            title: t("内存", "Memory"),
            dataIndex: "memory",
            render: (_, row) => `${row.memory.usagePercent.toFixed(1)}%`,
        },
        {
            title: t("磁盘挂载点", "Disk mounts"),
            key: "disks",
            render: (_, row) => (
                <div className="min-w-44 space-y-1 text-xs">
                    {row.disks.map((disk) => (
                        <div key={disk.mountPoint}>
                            {disk.mountPoint} {disk.usagePercent.toFixed(1)}%
                        </div>
                    ))}
                </div>
            ),
        },
        {
            title: t("最后上报", "Last report"),
            dataIndex: "lastReportAt",
            width: 180,
            render: (_, row) => formatDateTime(row.lastReportAt),
        },
        {
            title: t("详情", "Details"),
            key: "actions",
            width: 88,
            fixed: "right",
            render: (_, row) => (
                <Button
                    data-testid="monitor-node-view"
                    type="link"
                    onClick={() => setSelected(row)}
                >
                    {t("查看", "View")}
                </Button>
            ),
        },
    ];
    return (
        <PageCard
            title={t("节点", "Nodes")}
            description={t(
                "查看 Agent 上报的最新 CPU、内存和各磁盘挂载点。",
                "View the latest CPU, memory, and disk mounts reported by agents.",
            )}
            actions={
                <>
                    <Button
                        icon={<ReloadOutlined />}
                        loading={isFetching}
                        onClick={() => void refetch()}
                    >
                        {t("刷新", "Refresh")}
                    </Button>
                    {canViewSettings ? (
                        <Button
                            data-testid="monitor-global-settings"
                            icon={<SettingOutlined />}
                            onClick={() => setPanel("settings")}
                        >
                            {t("全局配置", "Global settings")}
                        </Button>
                    ) : null}
                    {canManage ? (
                        <Button
                            type="primary"
                            icon={<PlusOutlined />}
                            onClick={() => setPanel("add")}
                        >
                            {t("添加节点", "Add node")}
                        </Button>
                    ) : null}
                </>
            }
        >
            {!data ? (
                <DataState
                    kind={isPending ? "loading" : "error"}
                    title={
                        isPending
                            ? t("正在加载节点", "Loading nodes")
                            : t("节点加载失败", "Failed to load nodes")
                    }
                    action={
                        !isPending ? (
                            <Button onClick={() => void refetch()}>
                                {t("重新加载", "Reload")}
                            </Button>
                        ) : undefined
                    }
                />
            ) : (
                <>
                    {hasNodesBackgroundRefreshFailure(data, error) ? (
                        <BackgroundRefreshNotice
                            updatedAt={dataUpdatedAt}
                            onRetry={() => void refetch()}
                        />
                    ) : null}
                    <ProTable
                        rowKey="nodeId"
                        columns={columns}
                        dataSource={data}
                        loading={isFetching}
                        search={false}
                        options={false}
                        pagination={false}
                        locale={{
                            emptyText: (
                                <DataState
                                    kind="empty"
                                    title={t("暂无监控节点", "No monitored nodes")}
                                    description={t(
                                        "节点 Agent 首次上报后会自动出现在列表中。",
                                        "A node appears after its agent's first report.",
                                    )}
                                />
                            ),
                        }}
                    />
                </>
            )}
            <Drawer
                open={Boolean(panel)}
                onClose={() => setPanel(undefined)}
                title={
                    panel === "add" ? t("添加节点", "Add node") : t("全局配置", "Global settings")
                }
                size="large"
                styles={{ wrapper: { maxWidth: "100vw" } }}
                destroyOnHidden
            >
                {panel === "add" && canManage ? <NodeOnboarding /> : null}
                {panel === "settings" && canViewSettings ? <GlobalAlertSettings /> : null}
            </Drawer>
            <NodeDetails node={selected} onClose={() => setSelected(undefined)} />
        </PageCard>
    );
}
