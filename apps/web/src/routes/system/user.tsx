import { PlusOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
    Avatar,
    Button,
    Input,
    Select,
    Tag,
} from "antd";
import { useMemo, useState } from "react";

import { systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { getEnableOptions } from "@/constant/options";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { useFilteredPage } from "@/hooks/use-filtered-page";
import { localizeBuiltInRoleName, localizeBuiltInUserName } from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { UserActions } from "./-user-actions";
import { UserDialog } from "./-user-dialog";

export const Route = createFileRoute("/system/user")({
    component: UserPage,
});

const PAGE_SIZE = 20;

function UserPage() {
    useLocale();
    const queryClient = useQueryClient();
    const currentUserId = useAuthStore((state) => state.userInfo?.id);
    const [username, setUsername] = useState("");
    const [status, setStatus] = useState("all");
    const [isComposing, setIsComposing] = useState(false);
    const debouncedUsername = useDebouncedValue(username.trim(), 300, !isComposing);
    const [currentPage, setCurrentPage] = useFilteredPage(
        JSON.stringify([debouncedUsername, status]),
    );
    const params = useMemo<User.QueryParams>(
        () => ({
            current: currentPage,
            pageSize: PAGE_SIZE,
            username: debouncedUsername || undefined,
            status,
        }),
        [currentPage, debouncedUsername, status],
    );

    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["system", "user", params],
        queryFn: () => systemAPI.user.list(params),
        staleTime: 0,
    });

    const rows = data?.data ?? [];
    const total = data?.total ?? 0;
    const hasData = data !== undefined;

    const refresh = () => {
        void queryClient.invalidateQueries({ queryKey: ["system", "user"] });
        void queryClient.invalidateQueries({ queryKey: ["system", "role"] });
    };

    const columns: ProColumns<User.Item>[] = [
        {
            title: t("ID", "ID"),
            dataIndex: "id",
            key: "id",
            width: 64,
            render: (_: unknown, row: User.Item) => <span className="font-medium">{row.id}</span>,
        },
        {
            title: t("头像", "Avatar"),
            key: "avatar",
            width: 72,
            render: (_: unknown, row: User.Item) => (
                <Avatar size={32} src={row.avatarUrl ?? undefined} alt={row.username}>
                    {getUserInitial(row)}
                </Avatar>
            ),
        },
        {
            title: t("用户名", "Username"),
            dataIndex: "username",
            key: "username",
            width: 128,
        },
        {
            title: t("邮箱", "Email"),
            dataIndex: "email",
            key: "email",
            width: 192,
            ellipsis: true,
        },
        {
            title: t("真实姓名", "Real name"),
            key: "realName",
            width: 144,
            render: (_: unknown, row: User.Item) =>
                row.isSystem
                    ? localizeBuiltInUserName(row.username, row.realName)
                    : row.realName || "-",
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 88,
            render: (_: unknown, row: User.Item) => <UserStatusBadge status={row.status} />,
        },
        {
            title: t("角色", "Roles"),
            key: "roles",
            width: 176,
            ellipsis: true,
            render: (_: unknown, row: User.Item) =>
                row.roles
                    .map((role) =>
                        role.isSystem ? localizeBuiltInRoleName(role.code, role.label) : role.label,
                    )
                    .join(", ") || t("-", "-"),
        },
        {
            title: t("最后登录", "Last sign-in"),
            key: "lastLoginAt",
            width: 160,
            render: (_: unknown, row: User.Item) => formatDateTime(row.lastLoginAt),
        },
        {
            title: t("更新时间", "Updated at"),
            key: "updatedAt",
            width: 160,
            render: (_: unknown, row: User.Item) => formatDateTime(row.updatedAt),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 96,
            align: "right",
            render: (_: unknown, row: User.Item) => (
                <UserActions record={row} currentUserId={currentUserId} onSuccess={refresh} />
            ),
        },
    ];

    const searchControls = (
        <div className="flex min-w-0 flex-wrap items-center gap-2">
            <Input
                aria-label={t("用户名", "Username")}
                style={{ width: 200, maxWidth: "100%" }}
                allowClear
                value={username}
                placeholder={t("用户名", "Username")}
                onChange={(event) => setUsername(event.target.value)}
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
            />
            <Select
                style={{ width: 132, maxWidth: "100%" }}
                aria-label={t("账号状态", "Account status")}
                value={status}
                onChange={(value) => {
                    setStatus(value);
                    setCurrentPage(1);
                }}
                options={[
                    { value: "all", label: t("全部状态", "All statuses") },
                    ...getEnableOptions().map((item) => ({
                        value: String(item.value),
                        label: item.label,
                    })),
                ]}
            />
        </div>
    );

    if (!data && isPending) {
        return (
            <PageCard
                toolbar={searchControls}
                title={t("用户列表", "Users")}
                description={t(
                    "管理账号、角色和账号状态。",
                    "Manage accounts, roles, and account status.",
                )}
            >
                <DataState kind="loading" title={t("正在加载用户", "Loading users")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                toolbar={searchControls}
                title={t("用户列表", "Users")}
                description={t(
                    "管理账号、角色和账号状态。",
                    "Manage accounts, roles, and account status.",
                )}
                actions={
                    <AuthWrap code="system:user:create">
                        <UserDialog mode="create" onSuccess={refresh}>
                            <Button type="primary" icon={<PlusOutlined />}>
                                {t("新建用户", "New user")}
                            </Button>
                        </UserDialog>
                    </AuthWrap>
                }
            >
                <DataState
                    kind="error"
                    title={t("用户加载失败", "Failed to load users")}
                    description={
                        error instanceof Error
                            ? error.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    action={
                        <Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>
                    }
                />
            </PageCard>
        );
    }

    return (
        <PageCard
            toolbar={searchControls}
            title={t("用户列表", "Users")}
            description={t(
                "管理账号、角色和账号状态。",
                "Manage accounts, roles, and account status.",
            )}
            className="!h-auto"
            contentClassName="!flex-none"
            actions={
                <AuthWrap code="system:user:create">
                    <UserDialog mode="create" onSuccess={refresh}>
                        <Button type="primary" icon={<PlusOutlined />}>
                            {t("新建用户", "New user")}
                        </Button>
                    </UserDialog>
                </AuthWrap>
            }
        >
            {hasData && error && !isFetching ? (
                <DataState
                    compact
                    kind="error"
                    title={t("用户加载失败", "Failed to load users")}
                    description={
                        error instanceof Error
                            ? error.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    action={
                        <Button onClick={() => void refetch()}>{t("重新加载", "Reload")}</Button>
                    }
                />
            ) : null}
            <DataTableShell ariaLabel={t("系统用户", "System users table")}>
                <ProTable<User.Item>
                    rowKey="id"
                    columns={columns}
                    dataSource={rows}
                    loading={isPending || isFetching}
                    search={false}
                    options={false}
                    scroll={{ x: 1280 }}
                    pagination={{
                        current: currentPage,
                        pageSize: PAGE_SIZE,
                        total,
                        showSizeChanger: false,
                        hideOnSinglePage: true,
                        onChange: (page) => setCurrentPage(page),
                    }}
                    locale={{
                        emptyText:
                            rows.length === 0 ? (
                                <DataState kind="empty" title={t("暂无用户", "No users")} />
                            ) : undefined,
                    }}
                />
            </DataTableShell>
        </PageCard>
    );
}

function UserStatusBadge({ status }: { status: number }) {
    const statusMeta = {
        1: { label: t("启用", "Enabled"), color: "blue" as const },
        2: { label: t("禁用", "Disabled"), color: "default" as const },
        3: { label: t("待审核", "Pending"), color: "gold" as const },
        4: { label: t("已锁定", "Locked"), color: "red" as const },
    };
    const meta = statusMeta[status as keyof typeof statusMeta] ?? {
        label: t("未知", "Unknown"),
        color: "default" as const,
    };
    return <Tag color={meta.color}>{meta.label}</Tag>;
}

function getUserInitial(record: User.Item) {
    const name = record.isSystem
        ? localizeBuiltInUserName(record.username, record.realName)
        : record.realName || record.username;
    return name.slice(0, 1).toUpperCase();
}
