import {
    CheckCircleOutlined,
    ClockCircleOutlined,
    ExclamationCircleOutlined,
    FilterOutlined,
} from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Alert, Button, DatePicker, Drawer, Input, Select, Space, Tag, Typography } from "antd";
import type { Dayjs } from "dayjs";
import { useMemo, useState, type ReactNode } from "react";

import { monitorAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/monitoring/incidents")({
    component: MonitoringIncidentsPage,
});

const PAGE_SIZE = 20;

type StatusFilter = "all" | "open" | "acknowledged" | "resolved";
type SourceFilter = "all" | "node" | "check" | "resource";

const statusMeta = (status: Monitor.IncidentSummary["status"]) =>
    ({
        open: { label: t("活动", "Open"), color: "error", icon: <ExclamationCircleOutlined /> },
        acknowledged: {
            label: t("已确认", "Acknowledged"),
            color: "warning",
            icon: <ClockCircleOutlined />,
        },
        resolved: {
            label: t("已解决", "Resolved"),
            color: "success",
            icon: <CheckCircleOutlined />,
        },
    })[status];

const sourceLabel = (source: Monitor.IncidentSummary["sourceType"]) =>
    ({
        node: t("节点", "Node"),
        check: t("服务检查", "Check"),
        resource: t("资源", "Resource"),
    })[source];

function MonitoringIncidentsPage() {
    const canViewIncidents = useAuthStore((state) =>
        state.checkPermissions("monitor:incident:view"),
    );
    const [status, setStatus] = useState<StatusFilter>("all");
    const [sourceType, setSourceType] = useState<SourceFilter>(() => {
        if (typeof window === "undefined") return "all";
        const value = new URLSearchParams(window.location.search).get("sourceType");
        return value === "node" || value === "check" || value === "resource" ? value : "all";
    });
    const [sourceIdInput, setSourceIdInput] = useState(() => {
        if (typeof window === "undefined") return "";
        return new URLSearchParams(window.location.search).get("sourceId") ?? "";
    });
    const [sourceId, setSourceId] = useState(() => {
        if (typeof window === "undefined") return "";
        return new URLSearchParams(window.location.search).get("sourceId") ?? "";
    });
    const [fromValue, setFromValue] = useState<Dayjs | null>(null);
    const [toValue, setToValue] = useState<Dayjs | null>(null);
    const [from, setFrom] = useState<string>();
    const [to, setTo] = useState<string>();
    const [current, setCurrent] = useState(1);
    const [selected, setSelected] = useState<Monitor.IncidentSummary>();

    const query = useMemo<Monitor.IncidentQuery>(
        () => ({
            current,
            pageSize: PAGE_SIZE,
            status: status === "all" ? undefined : status,
            sourceType: sourceType === "all" ? undefined : sourceType,
            sourceId: sourceId || undefined,
            from,
            to,
        }),
        [current, from, sourceId, sourceType, status, to],
    );

    const { data, dataUpdatedAt, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["monitor", "incidents", query],
        queryFn: () => monitorAPI.incidents(query),
        enabled: canViewIncidents,
        refetchInterval: 30_000,
    });

    if (!canViewIncidents) return null;

    const rows = data?.data ?? [];
    const hasFilters = Boolean(status !== "all" || sourceType !== "all" || sourceId || from || to);
    const resetFilters = () => {
        setStatus("all");
        setSourceType("all");
        setSourceIdInput("");
        setSourceId("");
        setFrom(undefined);
        setTo(undefined);
        setFromValue(null);
        setToValue(null);
        setCurrent(1);
    };

    const columns: ProColumns<Monitor.IncidentSummary>[] = [
        {
            title: t("事件", "Incident"),
            key: "title",
            ellipsis: true,
            render: (_value: unknown, row: Monitor.IncidentSummary) => (
                <div>
                    <div className="font-medium">{row.title}</div>
                    <div className="text-xs text-muted-foreground">
                        {row.kind} · {row.sourceId}
                    </div>
                </div>
            ),
        },
        {
            title: t("来源", "Source"),
            key: "sourceType",
            width: 120,
            render: (_value: unknown, row: Monitor.IncidentSummary) => sourceLabel(row.sourceType),
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 140,
            render: (_value: unknown, row: Monitor.IncidentSummary) => {
                const meta = statusMeta(row.status);
                return (
                    <Tag icon={meta.icon} color={meta.color}>
                        {meta.label}
                    </Tag>
                );
            },
        },
        {
            title: t("最近观察", "Last observed"),
            key: "lastObservedAt",
            width: 190,
            render: (_value: unknown, row: Monitor.IncidentSummary) =>
                formatDateTime(row.lastObservedAt),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            width: 100,
            fixed: "right",
            render: (_value: unknown, row: Monitor.IncidentSummary) => (
                <Button type="link" onClick={() => setSelected(row)}>
                    {t("查看", "View")}
                </Button>
            ),
        },
    ];

    if (!data && isPending) {
        return (
            <PageCard
                title={t("监控事件", "Monitoring incidents")}
                description={t(
                    "查看 Monitor 已记录的活动与历史事件证据。",
                    "Inspect active and historical incident evidence recorded by Monitor.",
                )}
            >
                <DataState kind="loading" title={t("正在加载监控事件", "Loading incidents")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                title={t("监控事件", "Monitoring incidents")}
                description={t(
                    "查看 Monitor 已记录的活动与历史事件证据。",
                    "Inspect active and historical incident evidence recorded by Monitor.",
                )}
            >
                <DataState
                    kind="error"
                    title={t("监控事件加载失败", "Failed to load incidents")}
                    description={t(
                        "无法读取事件证据，请检查 Monitor 服务后重试。",
                        "Unable to read incident evidence. Check the Monitor service and try again.",
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

    return (
        <PageCard
            title={t("监控事件", "Monitoring incidents")}
            description={t(
                "查看 Monitor 已记录的活动与历史事件证据。",
                "Inspect active and historical incident evidence recorded by Monitor.",
            )}
            toolbar={
                <div className="flex flex-wrap items-center gap-3">
                    <Select<StatusFilter>
                        className="w-40"
                        aria-label={t("事件状态", "Incident status")}
                        value={status}
                        options={[
                            { value: "all", label: t("全部状态", "All statuses") },
                            { value: "open", label: t("活动", "Open") },
                            { value: "acknowledged", label: t("已确认", "Acknowledged") },
                            { value: "resolved", label: t("已解决", "Resolved") },
                        ]}
                        onChange={(value) => {
                            setStatus(value);
                            setCurrent(1);
                        }}
                    />
                    <Select<SourceFilter>
                        className="w-40"
                        aria-label={t("事件来源", "Incident source")}
                        value={sourceType}
                        options={[
                            { value: "all", label: t("全部来源", "All sources") },
                            { value: "node", label: t("节点", "Node") },
                            { value: "check", label: t("服务检查", "Check") },
                            { value: "resource", label: t("资源", "Resource") },
                        ]}
                        onChange={(value) => {
                            setSourceType(value);
                            setCurrent(1);
                        }}
                    />
                    <Input.Search
                        className="w-full sm:w-64"
                        allowClear
                        prefix={<FilterOutlined />}
                        aria-label={t("来源标识", "Source identifier")}
                        placeholder={t("来源标识", "Source identifier")}
                        value={sourceIdInput}
                        onChange={(event) => setSourceIdInput(event.target.value)}
                        onSearch={(value) => {
                            setSourceId(value.trim());
                            setCurrent(1);
                        }}
                    />
                    <DatePicker
                        value={fromValue}
                        showTime
                        allowClear
                        placeholder={t("开始时间", "From")}
                        onChange={(value) => {
                            setFromValue(value);
                            setFrom(value?.toISOString());
                            setCurrent(1);
                        }}
                    />
                    <DatePicker
                        value={toValue}
                        showTime
                        allowClear
                        placeholder={t("结束时间", "To")}
                        onChange={(value) => {
                            setToValue(value);
                            setTo(value?.toISOString());
                            setCurrent(1);
                        }}
                    />
                    {hasFilters ? (
                        <Button type="link" onClick={resetFilters}>
                            {t("重置筛选", "Reset filters")}
                        </Button>
                    ) : null}
                </div>
            }
        >
            {error ? (
                <BackgroundRefreshNotice updatedAt={dataUpdatedAt} onRetry={() => void refetch()} />
            ) : null}
            <DataTableShell>
                <ProTable<Monitor.IncidentSummary>
                    rowKey="id"
                    columns={columns}
                    dataSource={rows}
                    loading={isFetching}
                    search={false}
                    options={false}
                    pagination={{
                        current,
                        pageSize: PAGE_SIZE,
                        total: data?.total ?? 0,
                        showSizeChanger: false,
                        onChange: (page) => setCurrent(page),
                    }}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText: (
                            <DataState
                                kind="empty"
                                title={
                                    hasFilters
                                        ? t("没有匹配的监控事件", "No matching incidents")
                                        : t("暂无监控事件", "No incidents")
                                }
                                description={
                                    hasFilters
                                        ? t("请调整筛选条件。", "Adjust the filters and try again.")
                                        : t(
                                              "Monitor 记录事件后，证据会显示在这里。",
                                              "Incident evidence will appear here after Monitor records an event.",
                                          )
                                }
                                action={
                                    hasFilters ? (
                                        <Button onClick={resetFilters}>
                                            {t("清除筛选", "Clear filters")}
                                        </Button>
                                    ) : undefined
                                }
                            />
                        ),
                    }}
                />
            </DataTableShell>
            <IncidentDetailDrawer
                canViewIncidents={canViewIncidents}
                incident={selected}
                onClose={() => setSelected(undefined)}
            />
        </PageCard>
    );
}

function IncidentDetailDrawer({
    canViewIncidents,
    incident,
    onClose,
}: {
    canViewIncidents: boolean;
    incident?: Monitor.IncidentSummary;
    onClose: () => void;
}) {
    const { data, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["monitor", "incident", incident?.id],
        queryFn: () => monitorAPI.incident(incident!.id),
        enabled: canViewIncidents && Boolean(incident),
    });

    const partial = Boolean(
        data &&
        ((data.sourceType === "check" && !data.check) ||
            ((data.sourceType === "node" || data.sourceType === "resource") && !data.node)),
    );

    return (
        <Drawer
            open={Boolean(incident)}
            onClose={onClose}
            title={data?.title ?? incident?.title ?? t("事件详情", "Incident details")}
            size="large"
            destroyOnHidden
            footer={null}
        >
            {isPending ? (
                <DataState
                    kind="loading"
                    title={t("正在加载事件详情", "Loading incident details")}
                    compact
                />
            ) : error ? (
                <DataState
                    kind="error"
                    title={t("事件详情加载失败", "Failed to load incident details")}
                    description={t(
                        "选中的事件证据暂不可用，请重试或返回列表。",
                        "The selected incident evidence is unavailable. Retry or return to the list.",
                    )}
                    action={
                        <Space>
                            <Button onClick={onClose}>{t("返回列表", "Back to list")}</Button>
                            <Button
                                type="primary"
                                loading={isFetching}
                                onClick={() => void refetch()}
                            >
                                {t("重新加载", "Reload")}
                            </Button>
                        </Space>
                    }
                    compact
                />
            ) : data ? (
                <div className="space-y-4">
                    {partial ? (
                        <Alert
                            type="warning"
                            showIcon
                            message={t("事件上下文不完整", "Incident context is partial")}
                            description={t(
                                "事件记录仍可查看，但关联节点或检查上下文暂不可用。",
                                "The incident remains readable, but its related node or check context is unavailable.",
                            )}
                        />
                    ) : null}
                    <div className="grid gap-3 sm:grid-cols-2">
                        <Detail
                            label={t("状态", "Status")}
                            value={<IncidentStatusTag status={data.status} />}
                        />
                        <Detail
                            label={t("来源", "Source")}
                            value={`${sourceLabel(data.sourceType)} · ${data.sourceId}`}
                        />
                        <Detail
                            label={t("打开时间", "Opened")}
                            value={formatDateTime(data.openedAt)}
                        />
                        <Detail
                            label={t("最近观察", "Last observed")}
                            value={formatDateTime(data.lastObservedAt)}
                        />
                        <Detail
                            label={t("解决时间", "Resolved")}
                            value={formatDateTime(data.resolvedAt)}
                        />
                        <Detail label={t("事件类型", "Kind")} value={data.kind} />
                    </div>
                    {data.node ? (
                        <ContextCard title={t("节点上下文", "Node context")}>
                            <Detail label={t("主机名", "Hostname")} value={data.node.hostname} />
                            <Detail
                                label={t("Agent", "Agent")}
                                value={`${data.node.agentId} · v${data.node.agentVersion}`}
                            />
                            <Detail
                                label={t("最后在线", "Last seen")}
                                value={formatDateTime(data.node.lastSeenAt)}
                            />
                            <AuthWrap code="monitor:incident:view">
                                <Button
                                    type="link"
                                    href={`/monitoring/incidents?sourceType=${data.sourceType}&sourceId=${encodeURIComponent(data.node.id)}`}
                                >
                                    {t("查看该节点事件", "View incidents for this node")}
                                </Button>
                            </AuthWrap>
                        </ContextCard>
                    ) : null}
                    {data.check ? (
                        <ContextCard title={t("服务检查上下文", "Check context")}>
                            <Detail label={t("名称", "Name")} value={data.check.name} />
                            <Detail
                                label={t("目标", "Target")}
                                value={`${data.check.host}:${data.check.port}`}
                            />
                            <Detail
                                label={t("最近状态", "Last status")}
                                value={data.check.lastStatus ?? "-"}
                            />
                            <Detail
                                label={t("连续失败", "Consecutive failures")}
                                value={data.check.consecutiveFailures}
                            />
                            <AuthWrap code="monitor:incident:view">
                                <Button
                                    type="link"
                                    href={`/monitoring/incidents?sourceType=check&sourceId=${encodeURIComponent(data.check.id)}`}
                                >
                                    {t("查看该检查事件", "View incidents for this check")}
                                </Button>
                            </AuthWrap>
                        </ContextCard>
                    ) : null}
                    <div>
                        <Typography.Text strong>
                            {t("结构化证据", "Structured evidence")}
                        </Typography.Text>
                        <pre className="mt-2 max-h-72 overflow-auto whitespace-pre-wrap rounded border border-border bg-muted p-3 text-xs">
                            {JSON.stringify(data.details, null, 2)}
                        </pre>
                    </div>
                </div>
            ) : null}
        </Drawer>
    );
}

function IncidentStatusTag({ status }: { status: Monitor.IncidentSummary["status"] }) {
    const meta = statusMeta(status);
    return (
        <Tag icon={meta.icon} color={meta.color}>
            {meta.label}
        </Tag>
    );
}

function ContextCard({ title, children }: { title: string; children: ReactNode }) {
    return (
        <div className="space-y-2 rounded border border-border p-3">
            <Typography.Text strong>{title}</Typography.Text>
            <div className="grid gap-2 text-sm sm:grid-cols-2">{children}</div>
        </div>
    );
}

function Detail({ label, value }: { label: string; value: ReactNode }) {
    return (
        <div className="min-w-0">
            <Typography.Text type="secondary">{label}</Typography.Text>
            <div className="break-words">{value}</div>
        </div>
    );
}
