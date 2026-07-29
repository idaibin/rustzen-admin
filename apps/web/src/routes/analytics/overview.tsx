import {
    ApiOutlined,
    BarChartOutlined,
    BugOutlined,
    ClockCircleOutlined,
    EyeOutlined,
    FieldTimeOutlined,
    UserOutlined,
} from "@ant-design/icons";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card, Typography } from "antd";
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
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { MetricCard } from "@/components/page/metric-card";
import { PageHeader } from "@/components/page/page-header";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/analytics/overview")({
    component: AnalyticsOverviewPage,
});

function AnalyticsOverviewPage() {
    const {
        data: overview,
        dataUpdatedAt,
        error,
        isPending,
        refetch,
    } = useQuery({
        queryKey: ["insights", "overview"],
        queryFn: () => insightsAPI.overview({}),
        refetchInterval: 30_000,
    });
    if (isPending && !overview) {
        return (
            <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto pr-1">
                <PageHeader
                    title={t("分析概览", "Analytics overview")}
                    description={t(
                        "查看当前实例的页面、接口、事件和访客活动。",
                        "View page, API, event, and visitor activity for the current instance.",
                    )}
                />
                <DataState
                    kind="loading"
                    title={t("正在加载分析概览", "Loading analytics overview")}
                />
            </div>
        );
    }

    if (!overview) {
        return (
            <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto pr-1">
                <PageHeader
                    title={t("分析概览", "Analytics overview")}
                    description={t(
                        "查看当前实例的页面、接口、事件和访客活动。",
                        "View page, API, event, and visitor activity for the current instance.",
                    )}
                />
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
            </div>
        );
    }

    return (
        <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto pr-1">
            <PageHeader
                title={t("分析概览", "Analytics overview")}
                description={t(
                    "查看当前实例的页面、接口、事件和访客活动。",
                    "View page, API, event, and visitor activity for the current instance.",
                )}
            />
            <div className="flex flex-col gap-4">
                <Typography.Text strong>{t("核心活动", "Core activity")}</Typography.Text>
                <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
                    <MetricCard
                        label={t("页面浏览量", "Page views")}
                        value={overview.pv}
                        icon={<EyeOutlined />}
                        tone="primary"
                    />
                    <MetricCard
                        label={t("独立访客", "Unique visitors")}
                        value={overview.uv}
                        icon={<UserOutlined />}
                        tone="info"
                    />
                    <MetricCard
                        label={t("全部事件", "Total events")}
                        value={overview.eventCount}
                        icon={<BarChartOutlined />}
                        tone="primary"
                    />
                    <MetricCard
                        label={t("接口请求", "API requests")}
                        value={overview.requestCount}
                        icon={<ApiOutlined />}
                        tone="info"
                    />
                </div>
                <Typography.Text strong>
                    {t("质量与性能", "Quality and performance")}
                </Typography.Text>
                <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                    <MetricCard
                        label={t("错误数", "Errors")}
                        value={overview.errorCount}
                        icon={<BugOutlined />}
                        tone={overview.errorCount > 0 ? "danger" : "success"}
                        size="small"
                    />
                    <MetricCard
                        label={t("平均耗时", "Average duration")}
                        value={`${Math.round(overview.averageDurationMs)} ms`}
                        icon={<FieldTimeOutlined />}
                        tone="warning"
                        size="small"
                    />
                    <MetricCard
                        label={t("P95 耗时", "P95 duration")}
                        value={`${overview.p95DurationMs} ms`}
                        icon={<ClockCircleOutlined />}
                        tone="warning"
                        size="small"
                    />
                </div>
            </div>
            {error ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            <Card title={t("每日活动", "Daily activity")}>
                <div
                    className="h-60"
                    role="img"
                    aria-label={t(
                        "每日活动趋势图，包含页面浏览量、独立访客和请求数。",
                        "Daily activity trend chart with page views, unique visitors, and requests.",
                    )}
                >
                    <ResponsiveContainer width="100%" height="100%">
                        <LineChart data={overview.trend}>
                            <CartesianGrid strokeDasharray="3 3" />
                            <XAxis dataKey="date" />
                            <YAxis allowDecimals={false} />
                            <Tooltip />
                            <Line type="monotone" dataKey="pv" name="PV" stroke="var(--chart-1)" />
                            <Line type="monotone" dataKey="uv" name="UV" stroke="var(--chart-2)" />
                            <Line
                                type="monotone"
                                dataKey="requestCount"
                                name={t("请求数", "Requests")}
                                stroke="var(--chart-3)"
                            />
                        </LineChart>
                    </ResponsiveContainer>
                </div>
                <div className="sr-only">
                    <h2>{t("每日活动数据表", "Daily activity data table")}</h2>
                    <table>
                        <thead>
                            <tr>
                                <th scope="col">{t("日期", "Date")}</th>
                                <th scope="col">PV</th>
                                <th scope="col">UV</th>
                                <th scope="col">{t("请求数", "Requests")}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {overview.trend.map((item) => (
                                <tr key={item.date}>
                                    <td>{item.date}</td>
                                    <td>{item.pv}</td>
                                    <td>{item.uv}</td>
                                    <td>{item.requestCount}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            </Card>
        </div>
    );
}
