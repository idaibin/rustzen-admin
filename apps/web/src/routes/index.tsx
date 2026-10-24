import {
    ClockCircleFilled,
    ContactsFilled,
    IdcardFilled,
    UserSwitchOutlined,
} from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card, Progress, Typography } from "antd";
import type { ReactNode } from "react";

import { dashboardAPI, systemAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageHeader } from "@/components/page/page-header";
import { formatBytes, formatPercent } from "@/lib/format";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { ModuleHealthPanel } from "./-module-health-panel";

export const Route = createFileRoute("/")({
    component: DashboardPage,
});

function DashboardPage() {
    useLocale();
    const moduleQuery = useQuery({
        queryKey: ["dashboard", "modules"],
        queryFn: dashboardAPI.modules,
        refetchInterval: 15_000,
    });
    return (
        <div className="flex min-h-full flex-col gap-5">
            <PageHeader
                title={t("仪表盘", "Dashboard")}
                description={t(
                    "账号、角色权限与业务管理概览，含本机资源和扩展示例状态。",
                    "A business admin overview of accounts, roles, and permissions, with local resource and extension-example status.",
                )}
            />
            <AccountMetricCards />
            <div
                className={`grid min-w-0 gap-5 ${moduleQuery.data?.length === 0 ? "xl:grid-cols-1" : "xl:grid-cols-[minmax(0,1.7fr)_minmax(340px,1fr)]"}`}
            >
                <SystemResourceCards />
                <ModuleHealthPanel
                    data={moduleQuery.data}
                    dataUpdatedAt={moduleQuery.dataUpdatedAt}
                    error={moduleQuery.error}
                    isPending={moduleQuery.isPending}
                    isFetching={moduleQuery.isFetching}
                    onRetry={() => void moduleQuery.refetch()}
                />
            </div>
        </div>
    );
}

function SystemResourceCards() {
    useLocale();
    const canViewSystemStatus = useAuthStore((state) =>
        state.checkPermissions("system:status:view"),
    );
    const { data, dataUpdatedAt, error, isPending, isFetching, refetch } = useQuery({
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
                  value: data.resource.cpu.usagePercent,
                  hint: t(`${data.resource.cpu.cores} 核`, `${data.resource.cpu.cores} cores`),
                  tone: "blue" as const,
              },
              {
                  title: t("内存", "Memory"),
                  value: data.resource.memory.usagePercent,
                  hint: `${formatBytes(data.resource.memory.usedBytes)} / ${formatBytes(data.resource.memory.totalBytes)}`,
                  tone: "green" as const,
              },
              {
                  title: t("磁盘", "Disk"),
                  value: data.resource.disk.usagePercent,
                  hint: `${formatBytes(data.resource.disk.usedBytes)} / ${formatBytes(data.resource.disk.totalBytes)}`,
                  tone: "amber" as const,
              },
          ]
        : [];

    return (
        <Card className="page-panel h-full" variant="outlined" styles={{ body: { padding: 24 } }}>
            <div className="mb-6">
                <Typography.Title level={5} className="!mb-1.5">
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
                retrying={isFetching}
                onRetry={() => void refetch()}
            >
                <div className="grid gap-6 md:grid-cols-3">
                    {cards.map((item) => (
                        <div
                            key={item.title}
                            className="min-w-0 border-b border-border pb-5 last:border-b-0 last:pb-0 md:border-e md:border-b-0 md:pb-0 md:pe-6 md:last:border-e-0 md:last:pe-0"
                        >
                            <div className="mb-3 flex items-baseline justify-between gap-3">
                                <Typography.Text strong>{item.title}</Typography.Text>
                                <span className="text-2xl font-semibold tracking-tight text-foreground">
                                    {formatPercent(item.value)}
                                </span>
                            </div>
                            <Progress
                                aria-label={t(`${item.title}使用率`, `${item.title} usage`)}
                                percent={Number(item.value.toFixed(1))}
                                showInfo={false}
                                strokeColor={`var(--metric-${item.tone}-foreground)`}
                                railColor={`var(--metric-${item.tone}-surface)`}
                                size={["100%", 7]}
                            />
                            <div className="mt-3 truncate text-xs text-muted-foreground">
                                {item.hint}
                            </div>
                        </div>
                    ))}
                </div>
            </DashboardQueryBoundary>
        </Card>
    );
}

function AccountMetricCards() {
    useLocale();
    const {
        data: stats,
        dataUpdatedAt,
        error,
        isPending,
        isFetching,
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
        <section>
            <DashboardQueryBoundary
                isPending={isPending}
                error={error}
                hasData={stats !== undefined}
                loadingTitle={t("正在加载账号统计", "Loading account statistics")}
                errorTitle={t("账号统计加载失败", "Failed to load account statistics")}
                updatedAt={dataUpdatedAt}
                retrying={isFetching}
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
        </section>
    );
}

function DashboardQueryBoundary({
    isPending,
    retrying,
    error,
    hasData,
    loadingTitle,
    errorTitle,
    updatedAt,
    onRetry,
    children,
}: {
    isPending: boolean;
    retrying: boolean;
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
                action={
                    <Button loading={retrying} onClick={onRetry}>
                        {t("重新加载", "Reload")}
                    </Button>
                }
                compact
            />
        );
    }
    return (
        <>
            {children}
            {error ? (
                <BackgroundRefreshNotice
                    retrying={retrying}
                    updatedAt={updatedAt}
                    onRetry={onRetry}
                />
            ) : null}
        </>
    );
}
