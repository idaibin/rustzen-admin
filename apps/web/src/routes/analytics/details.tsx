import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Tag } from "antd";
import { useState } from "react";

import { insightsAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

type EventRow = Insights.Event;

export const Route = createFileRoute("/analytics/details")({
    component: AnalyticsEventsPage,
});
const pageSize = 20;

function AnalyticsEventsPage() {
    const [eventName, setEventName] = useState("");
    const [current, setCurrent] = useState(1);
    const query = { eventName: eventName || undefined, current, pageSize };
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["insights", "events", query],
        queryFn: () => insightsAPI.events(query),
    });

    const eventRows: EventRow[] = data?.data ?? [];

    if (!data && isPending) {
        return (
            <PageCard
                title={t("分析明细", "Analytics details")}
                description={t(
                    "查看当前实例的页面、接口、用户和业务原始事件。",
                    "View raw page, API, user, and business events for the current instance.",
                )}
            >
                <DataState kind="loading" title={t("正在加载事件", "Loading events")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                title={t("分析明细", "Analytics details")}
                description={t(
                    "查看当前实例的页面、接口、用户和业务原始事件。",
                    "View raw page, API, user, and business events for the current instance.",
                )}
            >
                <DataState
                    kind="error"
                    title={t("事件加载失败", "Failed to load events")}
                    description={t(
                        "无法读取分析明细，请检查 Insights 服务后重试。",
                        "Unable to read analytics data. Check the Insights service and try again.",
                    )}
                    action={
                        <Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>
                    }
                />
            </PageCard>
        );
    }

    const columns: ProColumns<EventRow>[] = [
        {
            title: t("事件", "Event"),
            key: "eventName",
            render: (_: unknown, row: EventRow) => (
                <Tag color={row.isError ? "error" : "blue"}>{row.eventName}</Tag>
            ),
        },
        {
            title: t("访客 / 用户", "Visitor / User"),
            key: "user",
            render: (_: unknown, row: EventRow) => (
                <div>
                    <div>{row.visitorId}</div>
                    <div className="text-xs text-muted-foreground">
                        {row.userId || t("匿名", "Anonymous")}
                    </div>
                </div>
            ),
        },
        {
            title: t("位置", "Location"),
            key: "location",
            render: (_: unknown, row: EventRow) => (
                <div className="font-mono text-xs">{row.pagePath || row.apiPath || "-"}</div>
            ),
        },
        {
            title: t("平台", "Platform"),
            key: "platform",
            render: (_: unknown, row: EventRow) => row.platform || "-",
        },
        {
            title: t("耗时", "Duration"),
            key: "durationMs",
            render: (_: unknown, row: EventRow) =>
                row.durationMs == null ? "-" : `${row.durationMs} ms`,
        },
        {
            title: t("发生时间", "Occurred at"),
            key: "occurredAt",
            render: (_: unknown, row: EventRow) => formatDateTime(row.occurredAt),
        },
    ];

    return (
        <PageCard
            title={t("分析明细", "Analytics details")}
            description={t(
                "查看当前实例的页面、接口、用户和业务原始事件。",
                "View raw page, API, user, and business events for the current instance.",
            )}
            toolbar={
                <div className="flex flex-wrap gap-3">
                    <Input
                        className="mt-auto w-64"
                        placeholder={t("输入完整事件名称", "Enter the full event name")}
                        value={eventName}
                        onChange={(event) => {
                            setEventName(event.target.value);
                            setCurrent(1);
                        }}
                    />
                </div>
            }
        >
            <ProTable<EventRow>
                rowKey="id"
                dataSource={eventRows}
                columns={columns}
                loading={isFetching}
                search={false}
                options={false}
                pagination={{
                    current,
                    pageSize,
                    total: data?.total ?? 0,
                    onChange: (page) => setCurrent(page),
                    showSizeChanger: false,
                }}
                locale={{
                    emptyText: !eventRows.length ? (
                        <DataState
                            kind="empty"
                            title={
                                eventName
                                    ? t("没有匹配的事件", "No matching events")
                                    : t("暂无分析事件", "No analytics events")
                            }
                            description={
                                eventName
                                    ? t(
                                          "请检查完整事件名称或清除筛选条件。",
                                          "Check the full event name or clear the filter.",
                                      )
                                    : t(
                                          "接收到埋点数据后，原始事件会显示在这里。",
                                          "Raw events will appear here after tracking data is received.",
                                      )
                            }
                        />
                    ) : undefined,
                }}
            />
        </PageCard>
    );
}
