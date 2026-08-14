import { DownloadOutlined, SearchOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Select, Tag } from "antd";
import { useEffect, useMemo, useState } from "react";

import { manageAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { t } from "@/lib/i18n";
import { useLocalStore } from "@/store/useLocalStore";

export const Route = createFileRoute("/manage/log")({
    component: LogPage,
});

const DEFAULT_ACTION = "AUTH_LOGIN";
const ALL_ACTION = "all";
const PAGE_SIZE = 20;

function LogPage() {
    const actionOptions = [
        { label: t("全部", "All"), value: ALL_ACTION },
        { label: t("登录", "Sign-in"), value: DEFAULT_ACTION },
        { label: "GET", value: "HTTP_GET" },
        { label: "POST", value: "HTTP_POST" },
        { label: "PUT", value: "HTTP_PUT" },
        { label: "DELETE", value: "HTTP_DELETE" },
    ];

    const [savedActionType, setActionType] = useLocalStore("log-action", DEFAULT_ACTION);
    const actionType = savedActionType || DEFAULT_ACTION;
    const selectedAction = actionType === ALL_ACTION ? undefined : actionType;
    const [searchInput, setSearchInput] = useState("");
    const [searchKeyword, setSearchKeyword] = useState("");
    const [currentPage, setCurrentPage] = useState(1);

    const params = useMemo<Log.QueryParams>(
        () => ({
            current: currentPage,
            pageSize: PAGE_SIZE,
            action: selectedAction,
            search: searchKeyword || undefined,
        }),
        [currentPage, searchKeyword, selectedAction],
    );
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["manage", "log", params],
        queryFn: () => manageAPI.log.list(params),
    });
    const rows = data?.data ?? [];
    const total = data?.total ?? 0;

    useEffect(() => {
        if (data === undefined || isFetching) {
            return;
        }
        const lastPage = Math.max(1, Math.ceil(total / PAGE_SIZE));
        if (currentPage > lastPage) {
            setCurrentPage(lastPage);
        }
    }, [currentPage, data, isFetching, total]);

    const updateAction = (value: string) => {
        setActionType(value);
        setCurrentPage(1);
    };

    const submitSearch = () => {
        setSearchKeyword(searchInput.trim());
        setCurrentPage(1);
    };

    const clearSearch = () => {
        setSearchInput("");
        setSearchKeyword("");
        setCurrentPage(1);
    };

    const exportButton = (
        <AuthWrap code="manage:log:export">
            <Button
                icon={<DownloadOutlined />}
                onClick={() => {
                    void manageAPI.log.export(params);
                }}
            >
                {t("导出", "Export")}
            </Button>
        </AuthWrap>
    );

    const logToolbar = (
        <div className="flex flex-wrap items-center gap-3">
            <Select
                aria-label={t("操作类型", "Action type")}
                className="w-36"
                value={actionType}
                options={actionOptions}
                onChange={updateAction}
            />
            <div className="flex w-full items-center gap-2 sm:w-auto">
                <Input.Search
                    prefix={<SearchOutlined />}
                    aria-label={t("搜索用户或 IP", "Search by user or IP")}
                    value={searchInput}
                    placeholder={t("搜索用户或 IP", "Search by user or IP")}
                    style={{ width: "100%", minWidth: 220 }}
                    onChange={(event) => {
                        const value = event.target.value;
                        setSearchInput(value);
                        if (!value) {
                            setSearchKeyword("");
                            setCurrentPage(1);
                        }
                    }}
                    onSearch={submitSearch}
                />
                <Button type="default" onClick={submitSearch}>
                    {t("查询", "Search")}
                </Button>
                {searchKeyword ? (
                    <Button type="default" onClick={clearSearch}>
                        {t("清除", "Clear")}
                    </Button>
                ) : null}
            </div>
        </div>
    );

    const columns: ProColumns<Log.Item>[] = [
        {
            title: "ID",
            dataIndex: "id",
            key: "id",
            width: 70,
        },
        {
            title: t("用户", "User"),
            dataIndex: "username",
            key: "username",
            render: (_: unknown, row: Log.Item) => row.username || t("匿名用户", "Anonymous user"),
        },
        {
            title: t("操作", "Action"),
            key: "action",
            render: (_: unknown, row: Log.Item) => <ActionBadge action={row.action} />,
            width: 100,
        },
        {
            title: t("描述", "Description"),
            key: "description",
            ellipsis: true,
            render: (_: unknown, row: Log.Item) => operationDescription(row.description),
        },
        {
            title: t("状态", "Status"),
            key: "status",
            render: (_: unknown, row: Log.Item) => <StatusBadge status={row.status} />,
            width: 90,
        },
        {
            title: t("IP 地址", "IP address"),
            dataIndex: "ipAddress",
            key: "ipAddress",
            width: 150,
            render: (_: unknown, row: Log.Item) => row.ipAddress || "-",
        },
        {
            title: t("耗时", "Duration"),
            key: "durationMs",
            width: 96,
            render: (_: unknown, row: Log.Item) => formatDuration(row.durationMs),
        },
        {
            title: t("创建时间", "Created at"),
            dataIndex: "createdAt",
            key: "createdAt",
            width: 180,
        },
    ];

    if (!rows.length && isPending) {
        return (
            <PageCard
                title={t("日志", "Logs")}
                description={t(
                    "审计管理服务中的登录和 HTTP 操作记录。",
                    "Audit sign-in and HTTP operations in the admin service.",
                )}
                actions={exportButton}
                toolbar={logToolbar}
            >
                <DataState kind="loading" title={t("正在加载日志", "Loading logs")} />
            </PageCard>
        );
    }

    if (!rows.length && error) {
        return (
            <PageCard
                title={t("日志", "Logs")}
                description={t(
                    "审计管理服务中的登录和 HTTP 操作记录。",
                    "Audit sign-in and HTTP operations in the admin service.",
                )}
                actions={exportButton}
                toolbar={logToolbar}
            >
                <DataState
                    kind="error"
                    title={t("日志加载失败", "Failed to load logs")}
                    description={
                        error instanceof Error
                            ? error.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    action={
                        <Button type="primary" onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </PageCard>
        );
    }

    if (total === 0) {
        return (
            <PageCard
                title={t("日志", "Logs")}
                description={t(
                    "审计管理服务中的登录和 HTTP 操作记录。",
                    "Audit sign-in and HTTP operations in the admin service.",
                )}
                actions={exportButton}
                toolbar={logToolbar}
            >
                <DataState kind="empty" title={t("暂无日志", "No logs")} />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("日志", "Logs")}
            description={t(
                "审计管理服务中的登录和 HTTP 操作记录。",
                "Audit sign-in and HTTP operations in the admin service.",
            )}
            actions={exportButton}
            toolbar={logToolbar}
        >
            <DataTableShell ariaLabel={t("操作日志", "Operation logs table")}>
                {error ? (
                    <DataState
                        kind="error"
                        title={t("日志刷新失败", "Failed to refresh logs")}
                        description={
                            error instanceof Error
                                ? error.message
                                : t("请稍后重试。", "Please try again later.")
                        }
                        action={
                            <Button type="primary" onClick={() => void refetch()}>
                                {t("重新加载", "Reload")}
                            </Button>
                        }
                        compact
                    />
                ) : null}
                <ProTable<Log.Item>
                    rowKey="id"
                    columns={columns}
                    dataSource={rows}
                    loading={isFetching}
                    search={false}
                    options={false}
                    pagination={{
                        current: currentPage,
                        pageSize: PAGE_SIZE,
                        total,
                        showSizeChanger: false,
                        onChange: (page) => {
                            setCurrentPage(page);
                        },
                    }}
                    locale={{
                        emptyText: <DataState kind="empty" title={t("暂无日志", "No logs")} />,
                    }}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                />
            </DataTableShell>
        </PageCard>
    );
}

const ActionBadge = ({ action }: { action: string }) => {
    const color = action === "AUTH_LOGIN" ? "blue" : "gold";
    return <Tag color={color}>{action}</Tag>;
};

const StatusBadge = ({ status }: { status: string }) => {
    const isSuccess = status === "SUCCESS";
    return (
        <Tag color={isSuccess ? "green" : "red"}>
            {isSuccess ? t("成功", "Success") : t("失败", "Failed")}
        </Tag>
    );
};

const formatDuration = (durationMs?: number) => {
    if (!durationMs) return "-";
    return `${durationMs}ms`;
};

const operationDescription = (description?: string | null) => {
    if (!description) return "-";
    return description === "User login successful"
        ? t("用户登录成功", "User signed in successfully")
        : description;
};
