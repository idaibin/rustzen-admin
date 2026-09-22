import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Tag } from "antd";
import { useEffect, useState } from "react";

import { manageAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { actionColumnWidth } from "@/components/table/action-column";
import { DataTableShell } from "@/components/table/data-table-shell";
import {
    emptyTableLocale,
    pagedTableProps,
    tablePagination,
} from "@/components/table/table-presets";
import { formatBytes } from "@/lib/format";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

import {
    CleanupDialog,
    DeleteVersionDialog,
    DeployVersionDialog,
    ExpireVersionDialog,
    UploadVersionDialog,
    componentLabel,
} from "./-deploy-dialogs";

export const Route = createFileRoute("/manage/deploy")({
    component: DeployPage,
});

const PAGE_SIZE = 20;

function DeployPage() {
    const [currentPage, setCurrentPage] = useState(1);
    const params: Deploy.ListParams = { current: currentPage, pageSize: PAGE_SIZE };
    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["manage", "deploy", params],
        queryFn: () => manageAPI.deploy.list(params),
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

    const refresh = () => {
        void refetch();
    };

    const columns: ProColumns<Deploy.Item>[] = [
        {
            title: t("组件", "Component"),
            dataIndex: "component",
            key: "component",
            width: 120,
            render: (_: unknown, row: Deploy.Item) => componentLabel(row.component),
        },
        {
            title: t("版本", "Version"),
            dataIndex: "version",
            key: "version",
            width: 120,
            render: (_: unknown, row: Deploy.Item) => (
                <span className="font-medium">{row.version}</span>
            ),
        },
        {
            title: t("架构", "Architecture"),
            key: "arch",
            width: 96,
            render: (_: unknown, row: Deploy.Item) => row.arch || "-",
        },
        {
            title: t("大小", "Size"),
            dataIndex: "fileSize",
            key: "fileSize",
            width: 110,
            render: (_: unknown, row: Deploy.Item) => formatBytes(row.fileSize),
        },
        {
            title: t("签名范围", "Signed scopes"),
            key: "signedScopes",
            width: 230,
            render: (_: unknown, row: Deploy.Item) => (
                <div className="flex flex-col items-start gap-1">
                    <Tag title={row.frontendHash} color="blue">
                        {t("前端", "Frontend")} {shortDigest(row.frontendHash)}
                    </Tag>
                    <Tag title={row.backendHash} color="green">
                        {t("后端", "Backend")} {shortDigest(row.backendHash)}
                    </Tag>
                </div>
            ),
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 120,
            render: (_: unknown, row: Deploy.Item) => <DeployStatusBadge record={row} />,
        },
        {
            title: t("部署人", "Deployed by"),
            dataIndex: "deployedBy",
            key: "deployedBy",
            width: 140,
            render: (_: unknown, row: Deploy.Item) => row.deployedBy || "-",
        },
        {
            title: t("部署时间", "Deployed at"),
            dataIndex: "deployedAt",
            key: "deployedAt",
            width: 190,
            render: (_: unknown, row: Deploy.Item) => formatDateTime(row.deployedAt),
        },
        {
            title: t("过期时间", "Expired at"),
            dataIndex: "expiredAt",
            key: "expiredAt",
            width: 190,
            render: (_: unknown, row: Deploy.Item) => formatDateTime(row.expiredAt),
        },
        {
            title: t("备注", "Notes"),
            dataIndex: "notes",
            key: "notes",
            width: 220,
            render: (_: unknown, row: Deploy.Item) => row.notes || "-",
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: actionColumnWidth(3),
            render: (_: unknown, row: Deploy.Item) => (
                <DeployActions record={row} onSuccess={refresh} />
            ),
        },
    ];

    if (!rows.length && isPending) {
        return (
            <PageCard
                title={t("部署版本", "Deployment versions")}
                description={t(
                    "上传签名的 rz 完整发行包并应用到四个服务。",
                    "Upload a signed complete rz release bundle and apply it to all four services.",
                )}
                actions={
                    <div className="flex flex-wrap gap-2">
                        <AuthWrap code="manage:deploy:create">
                            <UploadVersionDialog onSuccess={refresh} />
                        </AuthWrap>
                        <AuthWrap code="manage:deploy:delete">
                            <CleanupDialog onSuccess={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="loading"
                    title={t("正在加载部署版本", "Loading deployment versions")}
                />
            </PageCard>
        );
    }

    if (!rows.length && error) {
        return (
            <PageCard
                title={t("部署版本", "Deployment versions")}
                description={t(
                    "上传签名的 rz 完整发行包并应用到四个服务。",
                    "Upload a signed complete rz release bundle and apply it to all four services.",
                )}
                actions={
                    <div className="flex flex-wrap gap-2">
                        <AuthWrap code="manage:deploy:create">
                            <UploadVersionDialog onSuccess={refresh} />
                        </AuthWrap>
                        <AuthWrap code="manage:deploy:delete">
                            <CleanupDialog onSuccess={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState
                    kind="error"
                    title={t("部署版本加载失败", "Failed to load deployment versions")}
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
                title={t("部署版本", "Deployment versions")}
                description={t(
                    "上传签名的 rz 完整发行包并应用到四个服务。",
                    "Upload a signed complete rz release bundle and apply it to all four services.",
                )}
                actions={
                    <div className="flex flex-wrap gap-2">
                        <AuthWrap code="manage:deploy:create">
                            <UploadVersionDialog onSuccess={refresh} />
                        </AuthWrap>
                        <AuthWrap code="manage:deploy:delete">
                            <CleanupDialog onSuccess={refresh} />
                        </AuthWrap>
                    </div>
                }
            >
                <DataState kind="empty" title={t("暂无部署版本", "No deployment versions")} />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("部署版本", "Deployment versions")}
            description={t(
                "上传签名的 rz 完整发行包并应用到四个服务。",
                "Upload a signed complete rz release bundle and apply it to all four services.",
            )}
            actions={
                <div className="flex flex-wrap gap-2">
                    <AuthWrap code="manage:deploy:create">
                        <UploadVersionDialog onSuccess={refresh} />
                    </AuthWrap>
                    <AuthWrap code="manage:deploy:delete">
                        <CleanupDialog onSuccess={refresh} />
                    </AuthWrap>
                </div>
            }
        >
            {error ? (
                <DataState
                    kind="error"
                    title={t("部署版本刷新失败", "Failed to refresh deployment versions")}
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
            <DataTableShell ariaLabel={t("部署记录", "Deployments table")}>
                <ProTable<Deploy.Item>
                    rowKey="id"
                    columns={columns}
                    dataSource={rows}
                    loading={isFetching}
                    search={false}
                    options={false}
                    {...pagedTableProps}
                    pagination={tablePagination({
                        current: currentPage,
                        pageSize: PAGE_SIZE,
                        total,
                        onChange: (page) => {
                            setCurrentPage(page);
                        },
                    })}
                    locale={emptyTableLocale(t("暂无部署版本", "No deployment versions"))}
                />
            </DataTableShell>
        </PageCard>
    );
}

function DeployActions({ record, onSuccess }: { record: Deploy.Item; onSuccess: () => void }) {
    return (
        <div className="flex items-center gap-2">
            <AuthWrap code="manage:deploy:run">
                <DeployVersionDialog record={record} onSuccess={onSuccess} />
            </AuthWrap>
            <AuthWrap code="manage:deploy:update">
                <ExpireVersionDialog version={record} onSuccess={onSuccess} />
            </AuthWrap>
            <AuthWrap code="manage:deploy:delete">
                <DeleteVersionDialog record={record} onSuccess={onSuccess} />
            </AuthWrap>
        </div>
    );
}

function DeployStatusBadge({ record }: { record: Deploy.Item }) {
    if (record.isCurrent) {
        return <Tag color="blue">{t("当前", "Current")}</Tag>;
    }
    if (record.isExpired) {
        return <Tag color="red">{t("已过期", "Expired")}</Tag>;
    }
    if (record.isDeployed) {
        return <Tag color="green">{t("已部署", "Deployed")}</Tag>;
    }
    return <Tag>{t("已上传", "Uploaded")}</Tag>;
}

function shortDigest(value: string) {
    return `${value.slice(0, 12)}…`;
}
