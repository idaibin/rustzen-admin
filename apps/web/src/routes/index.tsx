import {
    ClockCircleOutlined,
    IdcardOutlined,
    TeamOutlined,
    UserSwitchOutlined,
} from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card, Tag } from "antd";
import type { ReactNode } from "react";

import { dashboardAPI } from "@/api";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageHeader } from "@/components/page/page-header";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/")({
    component: DashboardPage,
});

function DashboardPage() {
    return (
        <div className="operations-ledger mx-auto flex h-full min-h-0 min-w-0 w-full flex-col gap-5 overflow-y-auto pr-1">
            <PageHeader
                title={t("仪表盘", "Dashboard")}
                description={t(
                    "账号与运行模块的运维概览。",
                    "An operational overview of accounts and runtime modules.",
                )}
            />

            <div className="grid gap-5 xl:grid-cols-[minmax(0,2fr)_minmax(320px,1fr)]">
                <ModuleHealthCards />
                <StatsCards />
            </div>
        </div>
    );
}

function ModuleHealthCards() {
    const { data, dataUpdatedAt, error, isPending, refetch } = useQuery({
        queryKey: ["dashboard", "modules"],
        queryFn: dashboardAPI.modules,
        refetchInterval: 15_000,
    });

    return (
        <DashboardSection
            title={t("模块可用性", "Module availability")}
            description={t(
                "当前各运行模块的版本与连通状态。",
                "Versions and connectivity status of the active modules.",
            )}
        >
            <DashboardQueryBoundary
                isPending={isPending}
                error={error}
                hasData={data !== undefined}
                loadingTitle={t("正在加载模块状态", "Loading module status")}
                errorTitle={t("模块状态加载失败", "Failed to load module status")}
                updatedAt={dataUpdatedAt}
                onRetry={() => void refetch()}
            >
                <div className="divide-y">
                    {(["monitor", "insights", "reports"] as const).map((module) => {
                        const health = data?.find((item) => item.module === module);
                        const moduleLabel = {
                            monitor: t("监控", "Monitor"),
                            insights: t("分析", "Insights"),
                            reports: t("报表", "Reports"),
                        }[module];
                        return (
                            <div
                                key={module}
                                className="grid grid-cols-[1fr_auto] items-center gap-4 px-6 py-4"
                            >
                                <div className="min-w-0">
                                    <div className="font-medium">{moduleLabel}</div>
                                    <div className="mt-1 text-xs text-muted-foreground">
                                        {t("版本", "Version")} {health?.releaseVersion ?? "-"}
                                    </div>
                                </div>
                                <Tag color={health?.available ? "success" : "error"}>
                                    {health?.available
                                        ? t("可用", "Available")
                                        : t("不可用", "Unavailable")}
                                </Tag>
                            </div>
                        );
                    })}
                </div>
            </DashboardQueryBoundary>
        </DashboardSection>
    );
}

function StatsCards() {
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
            description: t("全部已注册账号", "All registered accounts"),
            icon: <TeamOutlined className="text-lg" />,
        },
        {
            title: t("活跃用户", "Active users"),
            value: stats?.activeUsers ?? 0,
            description: t("最近七天登录", "Signed in during the last seven days"),
            icon: <UserSwitchOutlined className="text-lg" />,
        },
        {
            title: t("今日登录", "Today's logins"),
            value: stats?.todayLogins ?? 0,
            description: t("最近二十四小时", "During the last 24 hours"),
            icon: <ClockCircleOutlined className="text-lg" />,
        },
        {
            title: t("待审核用户", "Pending users"),
            value: stats?.pendingUsers ?? 0,
            description: t("等待管理员处理", "Awaiting administrator action"),
            icon: <IdcardOutlined className="text-lg" />,
        },
    ];

    return (
        <DashboardSection
            title={t("账号台账", "Account overview")}
            description={t(
                "注册、活跃、近期和待审核访问情况。",
                "Registration, activity, recent access, and pending reviews.",
            )}
        >
            <div className="grid gap-4 p-4 sm:grid-cols-2">
                <DashboardQueryBoundary
                    isPending={isPending}
                    error={error}
                    hasData={stats !== undefined}
                    loadingTitle={t("正在加载账号统计", "Loading account statistics")}
                    errorTitle={t("账号统计加载失败", "Failed to load account statistics")}
                    updatedAt={dataUpdatedAt}
                    onRetry={() => void refetch()}
                >
                    {cards.map((item) => (
                        <MetricCard
                            key={item.title}
                            label={item.title}
                            value={item.value}
                            icon={item.icon}
                            hint={item.description}
                        />
                    ))}
                </DashboardQueryBoundary>
            </div>
        </DashboardSection>
    );
}

function DashboardSection({
    title,
    description,
    children,
}: {
    title: string;
    description: string;
    children: ReactNode;
}) {
    return (
        <Card
            className="!rounded-xl"
            title={<h2 className="text-lg font-medium">{title}</h2>}
            extra={null}
        >
            <p className="-mt-2 mb-4 text-sm text-muted-foreground">{description}</p>
            {children}
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
        return <DataState kind="loading" title={loadingTitle} compact className="col-span-full" />;
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
                className="col-span-full"
            />
        );
    }
    return (
        <>
            {children}
            {error ? (
                <div className="col-span-full">
                    <BackgroundRefreshNotice updatedAt={updatedAt} onRetry={onRetry} />
                </div>
            ) : null}
        </>
    );
}
