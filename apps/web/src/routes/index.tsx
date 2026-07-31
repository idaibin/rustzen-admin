import {
    BarChartOutlined,
    ClockCircleFilled,
    CloudServerOutlined,
    ContactsFilled,
    DashboardOutlined,
    DatabaseOutlined,
    FileTextOutlined,
    HddOutlined,
    IdcardFilled,
    UserSwitchOutlined,
} from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Badge, Button, Card, Typography, theme } from "antd";
import type { ReactNode } from "react";

import { dashboardAPI, systemAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageHeader } from "@/components/page/page-header";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/")({
    component: DashboardPage,
});

function DashboardPage() {
    return (
        <div className="flex min-h-full flex-col gap-5">
            <PageHeader
                title={t("仪表盘", "Dashboard")}
                description={t(
                    "账号、系统资源与运行模块的运维概览。",
                    "An operational overview of accounts, system resources, and runtime modules.",
                )}
            />
            <AccountMetricCards />
            <SystemResourceCards />
            <ModuleHealthCards />
        </div>
    );
}

function SystemResourceCards() {
    const canViewSystemStatus = useAuthStore((state) =>
        state.checkPermissions("system:status:view"),
    );
    const { data, dataUpdatedAt, error, isPending, refetch } = useQuery({
        queryKey: ["system", "status"],
        queryFn: systemAPI.status.overview,
        enabled: canViewSystemStatus,
    });

    if (!canViewSystemStatus) {
        return null;
    }

    const cards = data
        ? [
              {
                  title: "CPU",
                  value: formatPercent(data.resource.cpu.usagePercent),
                  hint: t(`${data.resource.cpu.cores} 核`, `${data.resource.cpu.cores} cores`),
                  icon: <DashboardOutlined />,
                  tone: "blue" as const,
              },
              {
                  title: t("内存", "Memory"),
                  value: formatPercent(data.resource.memory.usagePercent),
                  hint: `${formatBytes(data.resource.memory.usedBytes)} / ${formatBytes(data.resource.memory.totalBytes)}`,
                  icon: <DatabaseOutlined />,
                  tone: "green" as const,
              },
              {
                  title: t("磁盘", "Disk"),
                  value: formatPercent(data.resource.disk.usagePercent),
                  hint: `${formatBytes(data.resource.disk.usedBytes)} / ${formatBytes(data.resource.disk.totalBytes)}`,
                  icon: <HddOutlined />,
                  tone: "amber" as const,
              },
          ]
        : [];

    return (
        <Card styles={{ body: { padding: 20 } }}>
            <div className="mb-4">
                <Typography.Title level={5} className="!mb-1">
                    {t("系统运行状况", "System health")}
                </Typography.Title>
                <Typography.Text type="secondary">
                    {t(
                        "当前 Admin 服务器的关键资源使用情况。",
                        "Key resource usage for the current Admin host.",
                    )}
                </Typography.Text>
            </div>
            <DashboardQueryBoundary
                isPending={isPending}
                error={error}
                hasData={data !== undefined}
                loadingTitle={t("正在加载系统状态", "Loading system health")}
                errorTitle={t("系统状态加载失败", "Failed to load system health")}
                updatedAt={dataUpdatedAt}
                onRetry={() => void refetch()}
            >
                <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                    {cards.map((item) => (
                        <MetricCard
                            key={item.title}
                            label={item.title}
                            value={item.value}
                            hint={item.hint}
                            icon={item.icon}
                            tone={item.tone}
                        />
                    ))}
                </div>
            </DashboardQueryBoundary>
        </Card>
    );
}

function ModuleHealthCards() {
    const { token } = theme.useToken();
    const { data, dataUpdatedAt, error, isPending, refetch } = useQuery({
        queryKey: ["dashboard", "modules"],
        queryFn: dashboardAPI.modules,
        refetchInterval: 15_000,
    });

    return (
        <Card styles={{ body: { padding: 20 } }}>
            <div className="mb-4">
                <Typography.Title level={5} className="!mb-1">
                    {t("运行模块", "Runtime modules")}
                </Typography.Title>
                <Typography.Text type="secondary">
                    {t(
                        "实时查看各模块的服务连通状态。",
                        "View the live connectivity status of each module.",
                    )}
                </Typography.Text>
            </div>
            <DashboardQueryBoundary
                isPending={isPending}
                error={error}
                hasData={data !== undefined}
                loadingTitle={t("正在加载模块状态", "Loading module status")}
                errorTitle={t("模块状态加载失败", "Failed to load module status")}
                updatedAt={dataUpdatedAt}
                onRetry={() => void refetch()}
            >
                <div className="grid grid-cols-1 gap-3 md:grid-cols-3">
                    {(["monitor", "insights", "reports"] as const).map((module) => {
                        const health = data?.find((item) => item.module === module);
                        const available = health?.available ?? false;
                        const moduleMeta = {
                            monitor: {
                                label: t("监控", "Monitor"),
                                description: t("节点与服务监控", "Node and service monitoring"),
                                icon: <CloudServerOutlined />,
                            },
                            insights: {
                                label: t("分析", "Insights"),
                                description: t("访问与事件分析", "Traffic and event analytics"),
                                icon: <BarChartOutlined />,
                            },
                            reports: {
                                label: t("报表", "Reports"),
                                description: t("运营数据报表", "Operational data reports"),
                                icon: <FileTextOutlined />,
                            },
                        }[module];
                        return (
                            <div
                                key={module}
                                className="flex min-h-20 items-center gap-3 rounded-lg border border-border bg-muted/30 px-4 py-3"
                            >
                                <span
                                    className="flex size-10 shrink-0 items-center justify-center rounded-lg text-lg"
                                    style={{
                                        color: available ? token.colorSuccess : token.colorError,
                                        background: available
                                            ? token.colorSuccessBg
                                            : token.colorErrorBg,
                                    }}
                                >
                                    {moduleMeta.icon}
                                </span>
                                <div className="min-w-0 flex-1">
                                    <Typography.Text strong>{moduleMeta.label}</Typography.Text>
                                    <div className="truncate text-xs text-muted-foreground">
                                        {moduleMeta.description}
                                    </div>
                                </div>
                                <Badge
                                    status={available ? "success" : "error"}
                                    text={
                                        available
                                            ? t("运行中", "Online")
                                            : t("不可用", "Unavailable")
                                    }
                                />
                            </div>
                        );
                    })}
                </div>
            </DashboardQueryBoundary>
        </Card>
    );
}

function AccountMetricCards() {
    const {
        data: stats,
        dataUpdatedAt,
        error,
        isPending,
        refetch,
    } = useQuery({
        queryKey: ["dashboard", "stats"],
        queryFn: dashboardAPI.stats,
    });

    const cards = [
        {
            title: t("用户总数", "Total users"),
            value: stats?.totalUsers ?? 0,
            icon: <ContactsFilled />,
            tone: "blue" as const,
        },
        {
            title: t("活跃用户", "Active users"),
            value: stats?.activeUsers ?? 0,
            icon: <UserSwitchOutlined />,
            tone: "green" as const,
        },
        {
            title: t("今日登录", "Today's logins"),
            value: stats?.todayLogins ?? 0,
            icon: <ClockCircleFilled />,
            tone: "violet" as const,
        },
        {
            title: t("待审核用户", "Pending users"),
            value: stats?.pendingUsers ?? 0,
            icon: <IdcardFilled />,
            tone: "amber" as const,
        },
    ];

    return (
        <Card styles={{ body: { padding: 20 } }}>
            <div className="mb-4">
                <Typography.Title level={5} className="!mb-1">
                    {t("账号概览", "Account overview")}
                </Typography.Title>
                <Typography.Text type="secondary">
                    {t("当前账号规模与活跃情况。", "Current account volume and activity.")}
                </Typography.Text>
            </div>
            <DashboardQueryBoundary
                isPending={isPending}
                error={error}
                hasData={stats !== undefined}
                loadingTitle={t("正在加载账号统计", "Loading account statistics")}
                errorTitle={t("账号统计加载失败", "Failed to load account statistics")}
                updatedAt={dataUpdatedAt}
                onRetry={() => void refetch()}
            >
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
                    {cards.map((item) => (
                        <MetricCard
                            key={item.title}
                            label={item.title}
                            value={item.value}
                            icon={item.icon}
                            tone={item.tone}
                        />
                    ))}
                </div>
            </DashboardQueryBoundary>
        </Card>
    );
}

function DashboardQueryBoundary({
    isPending,
    error,
    hasData,
    loadingTitle,
    errorTitle,
    updatedAt,
    onRetry,
    children,
}: {
    isPending: boolean;
    error: Error | null;
    hasData: boolean;
    loadingTitle: string;
    errorTitle: string;
    updatedAt: number;
    onRetry: () => void;
    children: ReactNode;
}) {
    if (isPending && !hasData) {
        return <DataState kind="loading" title={loadingTitle} compact />;
    }
    if (error && !hasData) {
        return (
            <DataState
                kind="error"
                title={errorTitle}
                description={t(
                    "无法读取当前数据，请检查 Admin 服务后重试。",
                    "Unable to read the current data. Check the Admin service and try again.",
                )}
                action={<Button onClick={onRetry}>{t("重新加载", "Reload")}</Button>}
                compact
            />
        );
    }
    return (
        <>
            {children}
            {error ? <BackgroundRefreshNotice updatedAt={updatedAt} onRetry={onRetry} /> : null}
        </>
    );
}

function formatPercent(value: number) {
    return `${Number(value.toFixed(1))}%`;
}

function formatBytes(bytes: number) {
    if (!bytes) {
        return "0 B";
    }

    const units = ["B", "KB", "MB", "GB", "TB"] as const;
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
        value /= 1024;
        unitIndex += 1;
    }

    const precision = unitIndex === 0 ? 0 : 1;
    return `${Number(value.toFixed(precision))} ${units[unitIndex]}`;
}
