import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Input, Select, Typography } from "antd";
import { useMemo, useState } from "react";

import { systemAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { getCoreNavigationItems } from "@/components/layout/routes";
import { PageCard } from "@/components/page/page-card";
import { StatusTag } from "@/components/status-tag";
import { actionColumnWidth } from "@/components/table/action-column";
import { DataTableShell } from "@/components/table/data-table-shell";
import { displayTableProps, emptyTableLocale } from "@/components/table/table-presets";
import { getEnableOptions, getEnableStatusMeta, getMenuTypeMeta } from "@/constant/options";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

import { MenuActions, type DisplayMenuItem } from "./-menu-components";

export const Route = createFileRoute("/system/menu")({
    component: MenuPage,
});

function MenuPage() {
    const locale = useLocale();
    const [nameFilter, setNameFilter] = useState("");
    const [codeFilter, setCodeFilter] = useState("");
    const [statusFilter, setStatusFilter] = useState("all");
    const [isComposing, setIsComposing] = useState(false);
    const appliedName = useDebouncedValue(nameFilter, 300, !isComposing);
    const appliedCode = useDebouncedValue(codeFilter, 300, !isComposing);
    const { data, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["system", "menu", "inventory"],
        queryFn: systemAPI.menu.inventory,
    });
    const navigationRows = useMemo<DisplayMenuItem[]>(
        () => [
            ...getCoreNavigationItems().map((route, index) => ({
                id: -(index + 1),
                parentId: 0,
                name: route.name,
                code: route.permission ?? "",
                menuType: 2,
                sortOrder: index,
                status: 1,
                isSystem: true,
                isManual: false,
                path: route.path ?? null,
                icon: null,
                moduleId: null,
                moduleMenuCode: null,
                isActive: true,
                createdAt: "",
                updatedAt: "",
                readOnly: true,
            })),
            ...(data ?? []),
        ],
        [data, locale],
    );
    const pageError = error;
    const pagePending = isPending;
    const pageFetching = isFetching;

    const tableRows = useMemo(() => {
        const nameQuery = appliedName.trim().toLowerCase();
        const codeQuery = appliedCode.trim().toLowerCase();
        return navigationRows.filter((item) => {
            if (nameQuery && !item.name.toLowerCase().includes(nameQuery)) {
                return false;
            }
            if (codeQuery && !item.code.toLowerCase().includes(codeQuery)) {
                return false;
            }
            if (statusFilter !== "all" && String(item.status) !== statusFilter) {
                return false;
            }
            return true;
        });
    }, [appliedCode, appliedName, navigationRows, statusFilter]);

    const columns: ProColumns<DisplayMenuItem>[] = useMemo(
        () => [
            {
                title: t("名称", "Name"),
                dataIndex: "name",
                key: "name",
                width: 260,
                render: (_: unknown, row: DisplayMenuItem) => (
                    <span className="font-medium">{row.name}</span>
                ),
            },
            {
                title: t("路径", "Path"),
                dataIndex: "path",
                key: "path",
                width: 220,
                render: (_: unknown, row: DisplayMenuItem) => <span>{row.path || "-"}</span>,
            },
            {
                title: t("权限编码", "Permission code"),
                dataIndex: "code",
                key: "code",
                width: 220,
                render: (_: unknown, row: DisplayMenuItem) =>
                    row.code ? (
                        <Typography.Text code className="text-xs">
                            {row.code}
                        </Typography.Text>
                    ) : (
                        <span>-</span>
                    ),
            },
            {
                title: t("菜单类型", "Menu type"),
                dataIndex: "menuType",
                key: "menuType",
                width: 110,
                render: (_: unknown, row: DisplayMenuItem) => (
                    <StatusTag status={row.menuType} meta={getMenuTypeMeta()} />
                ),
            },
            {
                title: t("状态", "Status"),
                dataIndex: "status",
                key: "status",
                width: 110,
                render: (_: unknown, row: DisplayMenuItem) => (
                    <StatusTag status={row.status} meta={getEnableStatusMeta()} />
                ),
            },
            {
                title: t("排序", "Sort order"),
                dataIndex: "sortOrder",
                key: "sortOrder",
                width: 88,
            },
            {
                title: t("更新时间", "Updated at"),
                dataIndex: "updatedAt",
                key: "updatedAt",
                width: 180,
                render: (_: unknown, row: DisplayMenuItem) =>
                    row.updatedAt ? formatDateTime(row.updatedAt) : "-",
            },
            {
                title: t("操作", "Actions"),
                key: "actions",
                width: actionColumnWidth(1),
                fixed: "right",
                render: (_: unknown, row: DisplayMenuItem) => (
                    <MenuActions record={row} onSuccess={refresh} />
                ),
            },
        ],
        [locale],
    );

    const refresh = () => {
        void refetch();
    };

    const pageDescription = t(
        "与左侧导航和页面搜索使用同一份菜单结构；权限分配请在角色管理中完成。",
        "Uses the same menu structure as the sidebar and page search. Assign permissions in Role management.",
    );

    const filters = (
        <div className="flex max-w-full flex-nowrap items-center justify-end gap-2">
            <Input
                aria-label={t("菜单名称", "Menu name")}
                placeholder={t("菜单名称", "Menu name")}
                value={nameFilter}
                allowClear
                className="w-[168px] max-w-full"
                onChange={(event) => setNameFilter(event.target.value)}
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
            />
            <Input
                aria-label={t("权限编码", "Permission code")}
                placeholder={t("权限编码", "Permission code")}
                value={codeFilter}
                allowClear
                className="w-[180px] max-w-full"
                onChange={(event) => setCodeFilter(event.target.value)}
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
            />
            <Select
                aria-label={t("状态", "Status")}
                value={statusFilter}
                className="w-[132px] max-w-full"
                onChange={setStatusFilter}
                options={[
                    { value: "all", label: t("全部", "All") },
                    ...getEnableOptions().map((item) => ({
                        value: String(item.value),
                        label: item.label,
                    })),
                ]}
            />
        </div>
    );

    if (!data && pagePending) {
        return (
            <PageCard
                title={t("菜单管理", "Menu management")}
                description={pageDescription}
                toolbar={filters}
            >
                <DataState kind="loading" title={t("正在加载菜单", "Loading menus")} />
            </PageCard>
        );
    }

    if (!data && pageError) {
        return (
            <PageCard
                title={t("菜单管理", "Menu management")}
                description={pageDescription}
                toolbar={filters}
            >
                <DataState
                    kind="error"
                    title={t("菜单加载失败", "Failed to load menus")}
                    description={
                        pageError instanceof Error
                            ? pageError.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    action={
                        <Button type="primary" onClick={refresh}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            </PageCard>
        );
    }

    return (
        <PageCard
            title={t("菜单管理", "Menu management")}
            description={pageDescription}
            toolbar={filters}
        >
            {pageError ? (
                <DataState
                    kind="error"
                    title={t("菜单加载失败", "Failed to load menus")}
                    description={
                        pageError instanceof Error
                            ? pageError.message
                            : t("请稍后重试。", "Please try again later.")
                    }
                    action={
                        <Button type="primary" onClick={refresh}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            ) : null}
            <DataTableShell fill ariaLabel={t("系统菜单", "System menus table")}>
                <ProTable<DisplayMenuItem>
                    rowKey="id"
                    columns={columns}
                    dataSource={tableRows}
                    search={false}
                    loading={pageFetching}
                    options={false}
                    scroll={{ y: "100%" }}
                    {...displayTableProps}
                    locale={emptyTableLocale(t("暂无菜单", "No menus"), {
                        compact: true,
                        visible: tableRows.length === 0,
                    })}
                />
            </DataTableShell>
        </PageCard>
    );
}
