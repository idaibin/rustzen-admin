import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Select, Tag } from "antd";
import { useState } from "react";

import { insightsAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

type EventRow = Insights.Event;
type EventKindFilter = "all" | "page" | "api" | "other";

export const Route = createFileRoute("/analytics/details")({
    component: AnalyticsEventsPage,
});

const PAGE_SIZE = 20;

function AnalyticsEventsPage() {
    const [eventKind, setEventKind] = useState<EventKindFilter>("all");
    const [pathInput, setPathInput] = useState("");
    const [path, setPath] = useState("");
    const [current, setCurrent] = useState(1);
    const query: Insights.EventQuery = {
        eventKind: eventKind === "all" ? undefined : eventKind,
        path: path || undefined,
        current,
        pageSize: PAGE_SIZE,
    };
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
                    "查看页面访问、接口请求和其他操作上报。",
                    "View page visits, API requests, and other reported operations.",
                )}
            >
                <DataState kind="loading" title={t("正在加载访问记录", "Loading activity")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                title={t("分析明细", "Analytics details")}
                description={t(
                    "查看页面访问、接口请求和其他操作上报。",
                    "View page visits, API requests, and other reported operations.",
                )}
            >
                <DataState
                    kind="error"
                    title={t("访问记录加载失败", "Failed to load activity")}
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
            title: t("类型", "Type"),
            key: "type",
            width: 110,
            render: (_: unknown, row: EventRow) => <EventKindTag event={row} />,
        },
        {
            title: t("访问内容", "Target"),
            key: "target",
            ellipsis: true,
            render: (_: unknown, row: EventRow) => <EventTarget event={row} />,
        },
        {
            title: t("访客 / 用户", "Visitor / User"),
            key: "user",
            width: 180,
            render: (_: unknown, row: EventRow) => (
                <div>
                    <div className="truncate">{row.visitorId}</div>
                    <div className="truncate text-xs text-muted-foreground">
                        {row.userId || t("匿名", "Anonymous")}
                    </div>
                </div>
            ),
        },
        {
            title: t("结果", "Result"),
            key: "result",
            width: 110,
            render: (_: unknown, row: EventRow) => <EventResultTag event={row} />,
        },
        {
            title: t("平台", "Platform"),
            key: "platform",
            width: 100,
            render: (_: unknown, row: EventRow) => row.platform || "-",
        },
        {
            title: t("耗时", "Duration"),
            key: "durationMs",
            width: 100,
            render: (_: unknown, row: EventRow) =>
                row.durationMs == null ? "-" : `${row.durationMs} ms`,
        },
        {
            title: t("发生时间", "Occurred at"),
            key: "occurredAt",
            width: 180,
            render: (_: unknown, row: EventRow) => formatDateTime(row.occurredAt),
        },
    ];

    const hasFilters = eventKind !== "all" || Boolean(path);

    return (
        <PageCard
            title={t("分析明细", "Analytics details")}
            description={t(
                "查看页面访问、接口请求和其他操作上报。",
                "View page visits, API requests, and other reported operations.",
            )}
            toolbar={
                <div className="flex flex-wrap items-center gap-3">
                    <Select<EventKindFilter>
                        className="w-40"
                        aria-label={t("记录类型", "Activity type")}
                        value={eventKind}
                        options={[
                            { value: "all", label: t("全部类型", "All types") },
                            { value: "page", label: t("页面访问", "Page visits") },
                            { value: "api", label: t("接口请求", "API requests") },
                            { value: "other", label: t("其他上报", "Other reports") },
                        ]}
                        onChange={(value) => {
                            setEventKind(value);
                            if (value === "other") {
                                setPathInput("");
                                setPath("");
                            }
                            setCurrent(1);
                        }}
                    />
                    <Input.Search
                        className="w-full sm:w-80"
                        allowClear
                        disabled={eventKind === "other"}
                        aria-label={t("搜索页面或接口路径", "Search page or API path")}
                        placeholder={t("页面或接口路径", "Page or API path")}
                        value={pathInput}
                        onChange={(event) => {
                            const value = event.target.value;
                            setPathInput(value);
                            if (!value) {
                                setPath("");
                                setCurrent(1);
                            }
                        }}
                        onSearch={(value) => {
                            setPath(value.trim());
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
                    pageSize: PAGE_SIZE,
                    total: data?.total ?? 0,
                    onChange: (page) => setCurrent(page),
                    showSizeChanger: false,
                }}
                locale={{
                    emptyText: !eventRows.length ? (
                        <DataState
                            kind="empty"
                            title={
                                hasFilters
                                    ? t("没有匹配的访问记录", "No matching activity")
                                    : t("暂无访问记录", "No activity yet")
                            }
                            description={
                                hasFilters
                                    ? t(
                                          "请调整类型或路径筛选条件。",
                                          "Adjust the type or path filters.",
                                      )
                                    : t(
                                          "接收到页面访问、接口请求或其他操作上报后，记录会显示在这里。",
                                          "Page visits, API requests, and other reports will appear here after they are received.",
                                      )
                            }
                        />
                    ) : undefined,
                }}
            />
        </PageCard>
    );
}

function eventKind(event: EventRow): Exclude<EventKindFilter, "all"> {
    if (event.eventName === "page_view") return "page";
    if (event.eventName === "api_request") return "api";
    return "other";
}

function EventKindTag({ event }: { event: EventRow }) {
    const kind = eventKind(event);
    const meta = {
        page: { color: "blue", label: t("页面访问", "Page visit") },
        api: { color: "cyan", label: t("接口请求", "API request") },
        other: { color: "purple", label: t("其他上报", "Other report") },
    }[kind];
    return <Tag color={meta.color}>{meta.label}</Tag>;
}

function EventTarget({ event }: { event: EventRow }) {
    const kind = eventKind(event);
    if (kind === "page") {
        return (
            <div>
                <div className="truncate font-mono text-xs">{event.pagePath || "-"}</div>
                {event.referrer ? (
                    <div className="truncate text-xs text-muted-foreground">
                        {t("来源", "Referrer")}：{event.referrer}
                    </div>
                ) : null}
            </div>
        );
    }
    if (kind === "api") {
        return (
            <div className="flex min-w-0 items-center gap-2">
                <Tag>{event.apiMethod || "API"}</Tag>
                <span className="truncate font-mono text-xs">{event.apiPath || "-"}</span>
            </div>
        );
    }
    return <span className="font-mono text-xs">{event.eventName}</span>;
}

function EventResultTag({ event }: { event: EventRow }) {
    if (event.statusCode != null) {
        return <Tag color={event.statusCode < 400 ? "green" : "red"}>{event.statusCode}</Tag>;
    }
    return (
        <Tag color={event.isError ? "red" : "green"}>
            {event.isError ? t("失败", "Failed") : t("已记录", "Recorded")}
        </Tag>
    );
}
