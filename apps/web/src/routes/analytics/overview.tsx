import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card } from "antd";
import {
    CartesianGrid,
    Line,
    LineChart,
    ResponsiveContainer,
    Tooltip,
    XAxis,
    YAxis,
} from "recharts";

import { insightsAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageCard } from "@/components/page/page-card";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/analytics/overview")({
    component: AnalyticsOverviewPage,
});

function AnalyticsOverviewPage() {
    const {
        data: overview,
        error,
        isPending,
        refetch,
    } = useQuery({
        queryKey: ["insights", "overview"],
        queryFn: () => insightsAPI.overview({}),
        refetchInterval: 30_000,
    });
    if (isPending) {
        return (
            <PageCard
                title={t("分析概览", "Analytics overview")}
                description={t(
                    "查看当前实例的页面、接口、事件和访客活动。",
                    "View page, API, event, and visitor activity for the current instance.",
                )}
            >
                <DataState
                    kind="loading"
                    title={t("正在加载分析概览", "Loading analytics overview")}
                />
            </PageCard>
        );
    }

    if (!overview) {
        return (
            <PageCard
                title={t("分析概览", "Analytics overview")}
                description={t(
                    "查看当前实例的页面、接口、事件和访客活动。",
                    "View page, API, event, and visitor activity for the current instance.",
                )}
            >
                <DataState
                    kind="error"
                    title={
                        error
                            ? t("分析概览加载失败", "Failed to load analytics overview")
                            : t("分析概览暂不可用", "Analytics overview is unavailable")
                    }
                    description={t(
                        "无法读取分析数据，请检查 Insights 服务后重试。",
                        "Unable to read analytics data. Check the Insights service and try again.",
                    )}
                    action={
                        <Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>
                    }
                />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("分析概览", "Analytics overview")}
            description={t(
                "查看当前实例的页面、接口、事件和访客活动。",
                "View page, API, event, and visitor activity for the current instance.",
            )}
        >
            <>
                <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                    <MetricCard label={t("页面浏览量", "Page views")} value={overview.pv} />
                    <MetricCard label={t("独立访客", "Unique visitors")} value={overview.uv} />
                    <MetricCard label={t("全部事件", "Total events")} value={overview.eventCount} />
                    <MetricCard
                        label={t("接口请求", "API requests")}
                        value={overview.requestCount}
                    />
                    <MetricCard label={t("错误数", "Errors")} value={overview.errorCount} />
                    <MetricCard
                        label={t("平均耗时", "Average duration")}
                        value={`${Math.round(overview.averageDurationMs)} ms`}
                    />
                    <MetricCard
                        label={t("P95 耗时", "P95 duration")}
                        value={`${overview.p95DurationMs} ms`}
                    />
                </div>
                <Card>
                    <Card title={t("每日活动", "Daily activity")} className="h-72">
                        <div className="h-60">
                            <ResponsiveContainer width="100%" height="100%">
                                <LineChart data={overview.trend}>
                                    <CartesianGrid strokeDasharray="3 3" />
                                    <XAxis dataKey="date" />
                                    <YAxis allowDecimals={false} />
                                    <Tooltip />
                                    <Line
                                        type="monotone"
                                        dataKey="pv"
                                        name="PV"
                                        stroke="var(--chart-1)"
                                    />
                                    <Line
                                        type="monotone"
                                        dataKey="uv"
                                        name="UV"
                                        stroke="var(--chart-2)"
                                    />
                                    <Line
                                        type="monotone"
                                        dataKey="requestCount"
                                        name={t("请求数", "Requests")}
                                        stroke="var(--chart-3)"
                                    />
                                </LineChart>
                            </ResponsiveContainer>
                        </div>
                    </Card>
                </Card>
            </>
        </PageCard>
    );
}
