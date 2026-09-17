import { PlusOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Select, Tag } from "antd";
import { useMemo, useState } from "react";

import { systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { actionColumnWidth } from "@/components/table/action-column";
import { getEnableOptions } from "@/constant/options";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { useFilteredPage } from "@/hooks/use-filtered-page";
import {
    localizeBuiltInMenuName,
    localizeBuiltInRoleDescription,
    localizeBuiltInRoleName,
} from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { RoleActions } from "./-role-actions";
import { RoleDialog } from "./-role-dialog";

const PAGE_SIZE = 20;

export const Route = createFileRoute("/system/role")({
    component: RolePage,
});

function RolePage() {
    useLocale();
    const queryClient = useQueryClient();
    const [roleName, setRoleName] = useState("");
    const [roleCode, setRoleCode] = useState("");
    const [status, setStatus] = useState("all");
    const [isComposing, setIsComposing] = useState(false);
    const appliedName = useDebouncedValue(roleName.trim(), 300, !isComposing);
    const appliedCode = useDebouncedValue(roleCode.trim(), 300, !isComposing);
    const filters = useMemo(
        () => ({ roleName: appliedName, roleCode: appliedCode, status }),
        [appliedName, appliedCode, status],
    );
    const [currentPage, setCurrentPage] = useFilteredPage(JSON.stringify(filters));
    const params = useMemo<Role.QueryParams>(
        () => ({
            current: currentPage,
            pageSize: PAGE_SIZE,
            roleName: filters.roleName || undefined,
            roleCode: filters.roleCode || undefined,
            status: filters.status,
        }),
        [currentPage, filters],
    );

    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["system", "role", params],
        queryFn: () => systemAPI.role.list(params),
        staleTime: 0,
    });

    const rows = data?.data ?? [];
    const total = data?.total ?? 0;

    const refresh = () => {
        void queryClient.invalidateQueries({ queryKey: ["system", "role"] });
        void queryClient.invalidateQueries({ queryKey: ["system", "user"] });
        void queryClient.invalidateQueries({ queryKey: ["system", "roles", "options"] });
    };

    const columns: ProColumns<Role.Item>[] = [
        {
            title: t("ID", "ID"),
            dataIndex: "id",
            key: "id",
            width: 70,
            render: (_: unknown, row: Role.Item) => <span className="font-medium">{row.id}</span>,
        },
        {
            title: t("角色名称", "Role name"),
            key: "name",
            width: 170,
            render: (_: unknown, row: Role.Item) => (
                <span>{localizeBuiltInRoleName(row.code, row.name)}</span>
            ),
        },
        {
            title: t("角色编码", "Role code"),
            dataIndex: "code",
            key: "code",
            width: 170,
            render: (_: unknown, row: Role.Item) => <Tag color="blue">{row.code}</Tag>,
        },
        {
            title: t("描述", "Description"),
            key: "description",
            width: 260,
            ellipsis: true,
            render: (_: unknown, row: Role.Item) =>
                localizeBuiltInRoleDescription(row.code, row.description) || "-",
        },
        {
            title: t("状态", "Status"),
            key: "status",
            width: 120,
            render: (_: unknown, row: Role.Item) => <RoleStatusBadge status={row.status} />,
        },
        {
            title: t("权限", "Permissions"),
            key: "permissions",
            width: 160,
            render: (_: unknown, row: Role.Item) =>
                row.menus?.length ? (
                    <span
                        title={row.menus
                            .map((menu) =>
                                localizeBuiltInMenuName({
                                    name: menu.label,
                                    code: String(menu.value),
                                    isSystem: false,
                                    moduleId: null,
                                    moduleMenuCode: null,
                                }),
                            )
                            .join(", ")}
                    >
                        {t(`${row.menus.length} 项权限`, `${row.menus.length} permissions`)}
                    </span>
                ) : (
                    <span className="text-muted-foreground">{t("暂无权限", "No permissions")}</span>
                ),
        },
        {
            title: t("已分配用户", "Assigned users"),
            dataIndex: "assignedUserCount",
            key: "assignedUserCount",
            width: 150,
            render: (_: unknown, row: Role.Item) => row.assignedUserCount,
        },
        {
            title: t("更新时间", "Updated at"),
            key: "updatedAt",
            width: 220,
            render: (_: unknown, row: Role.Item) => formatDateTime(row.updatedAt),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: actionColumnWidth(2),
            render: (_: unknown, row: Role.Item) => (
                <RoleActions record={row} onSuccess={refresh} />
            ),
        },
    ];

    const searchControls = (
        <div className="flex max-w-full flex-nowrap items-center justify-end gap-2">
            <Input
                allowClear
                className="w-[168px] max-w-full"
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
                aria-label={t("角色名称", "Role name")}
                value={roleName}
                placeholder={t("角色名称", "Role name")}
                onChange={(event) => setRoleName(event.target.value)}
            />
            <Input
                allowClear
                className="w-[168px] max-w-full"
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
                aria-label={t("角色编码", "Role code")}
                value={roleCode}
                placeholder={t("角色编码", "Role code")}
                onChange={(event) => setRoleCode(event.target.value)}
            />
            <Select
                className="w-[132px] max-w-full"
                aria-label={t("角色状态", "Role status")}
                value={status}
                onChange={setStatus}
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
                title={t("角色管理", "Role management")}
                description={t(
                    "管理角色定义和权限分配。",
                    "Manage role definitions and permission assignments.",
                )}
            >
                <DataState kind="loading" title={t("正在加载角色", "Loading roles")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                toolbar={searchControls}
                title={t("角色管理", "Role management")}
                description={t(
                    "管理角色定义和权限分配。",
                    "Manage role definitions and permission assignments.",
                )}
                actions={
                    <AuthWrap code="system:role:create">
                        <RoleDialog mode="create" onSuccess={refresh}>
                            <Button type="primary" icon={<PlusOutlined />}>
                                {t("新建角色", "New role")}
                            </Button>
                        </RoleDialog>
                    </AuthWrap>
                }
            >
                <DataState
                    kind="error"
                    title={t("角色加载失败", "Failed to load roles")}
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
            title={t("角色管理", "Role management")}
            description={t(
                "管理角色定义和权限分配。",
                "Manage role definitions and permission assignments.",
            )}
            actions={
                <AuthWrap code="system:role:create">
                    <RoleDialog mode="create" onSuccess={refresh}>
                        <Button type="primary" icon={<PlusOutlined />}>
                            {t("新建角色", "New role")}
                        </Button>
                    </RoleDialog>
                </AuthWrap>
            }
        >
            <ProTable<Role.Item>
                rowKey="id"
                columns={columns}
                dataSource={rows}
                loading={isPending || isFetching}
                search={false}
                options={false}
                pagination={{
                    current: currentPage,
                    pageSize: PAGE_SIZE,
                    total,
                    showSizeChanger: false,
                    onChange: (page) => setCurrentPage(page),
                }}
                locale={{
                    emptyText:
                        rows.length === 0 ? (
                            <DataState kind="empty" title={t("暂无角色", "No roles")} />
                        ) : undefined,
                }}
            />
        </PageCard>
    );
}

function RoleStatusBadge({ status }: { status: number }) {
    const statusMeta = {
        1: { label: t("启用", "Enabled"), color: "blue" as const },
        2: { label: t("禁用", "Disabled"), color: "default" as const },
    };
    const meta = statusMeta[status as keyof typeof statusMeta] ?? {
        label: t("未知", "Unknown"),
        color: "default" as const,
    };
    return <Tag color={meta.color}>{meta.label}</Tag>;
}
