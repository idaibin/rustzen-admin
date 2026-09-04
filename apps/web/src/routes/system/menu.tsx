import { EditOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Input, Modal, Select, Tag, Typography } from "antd";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { getCoreNavigationItems } from "@/components/layout/routes";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { getEnableOptions, getModuleIconOptions } from "@/constant/options";
import { useDebouncedValue } from "@/hooks/use-debounced-value";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

export const Route = createFileRoute("/system/menu")({
    component: MenuPage,
});

type DisplayMenuItem = Menu.Item & {
    readOnly?: boolean;
};

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
                    <MenuTypeBadge menuType={row.menuType} />
                ),
            },
            {
                title: t("状态", "Status"),
                dataIndex: "status",
                key: "status",
                width: 110,
                render: (_: unknown, row: DisplayMenuItem) => (
                    <MenuStatusBadge status={row.status} />
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
                width: 128,
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
        <div className="flex max-w-full flex-wrap items-center justify-end gap-2">
            <Input
                aria-label={t("菜单名称", "Menu name")}
                placeholder={t("菜单名称", "Menu name")}
                value={nameFilter}
                allowClear
                style={{ width: 168, maxWidth: "100%" }}
                onChange={(event) => setNameFilter(event.target.value)}
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
            />
            <Input
                aria-label={t("权限编码", "Permission code")}
                placeholder={t("权限编码", "Permission code")}
                value={codeFilter}
                allowClear
                style={{ width: 180, maxWidth: "100%" }}
                onChange={(event) => setCodeFilter(event.target.value)}
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
            />
            <Select
                aria-label={t("状态", "Status")}
                value={statusFilter}
                style={{ width: 132, maxWidth: "100%" }}
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
                    pagination={false}
                    scroll={{ y: "100%" }}
                    toolBarRender={false}
                    tableAlertOptionRender={false}
                    rowSelection={false}
                    locale={{
                        emptyText:
                            tableRows.length === 0 ? (
                                <DataState kind="empty" title={t("暂无菜单", "No menus")} compact />
                            ) : undefined,
                    }}
                />
            </DataTableShell>
        </PageCard>
    );
}

function MenuActions({ record, onSuccess }: { record: DisplayMenuItem; onSuccess: () => void }) {
    if (record.readOnly || !record.moduleId || !record.moduleMenuCode) {
        return null;
    }

    return (
        <AuthWrap code="system:menu:update">
            <ModuleMenuDialog record={record} onSuccess={onSuccess}>
                <Button
                    type="text"
                    icon={<EditOutlined />}
                    aria-label={t("编辑导航菜单", "Edit navigation menu")}
                />
            </ModuleMenuDialog>
        </AuthWrap>
    );
}

interface ModuleMenuDialogProps {
    record: Menu.Item;
    children: ReactNode;
    onSuccess?: () => void;
}

function ModuleMenuDialog({ children, record, onSuccess }: ModuleMenuDialogProps) {
    const queryClient = useQueryClient();
    const [open, setOpen] = useState(false);
    const [name, setName] = useState("");
    const [status, setStatus] = useState("1");
    const [sortOrder, setSortOrder] = useState("0");
    const [icon, setIcon] = useState("");
    const [submitting, setSubmitting] = useState(false);

    useEffect(() => {
        if (open) {
            setName(record.name);
            setStatus(String(record.status));
            setSortOrder(String(record.sortOrder));
            setIcon(record.icon ?? "");
        }
    }, [open, record]);

    const submit = async () => {
        const trimmedName = name.trim();
        const parsedSortOrder = Number(sortOrder);

        if (!trimmedName) {
            appMessage.error(t("请输入菜单名称", "Enter a menu name."));
            return;
        }
        if (!Number.isInteger(parsedSortOrder) || parsedSortOrder < 0) {
            appMessage.error(
                t("排序必须是非负整数", "The sort order must be a non-negative integer."),
            );
            return;
        }

        setSubmitting(true);
        try {
            await systemAPI.menu.update(record.id, {
                name: trimmedName,
                sortOrder: parsedSortOrder,
                status: Number(status),
                icon: icon || null,
            });
            appMessage.success(t("导航菜单已更新", "Navigation menu updated."));
            await queryClient.invalidateQueries({
                queryKey: ["system", "menu", "inventory"],
            });
            onSuccess?.();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <>
            <span
                onClick={(event) => {
                    event.stopPropagation();
                    event.preventDefault();
                    setOpen(true);
                }}
            >
                {children}
            </span>
            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                destroyOnHidden
                footer={null}
                title={t("编辑导航菜单", "Edit navigation menu")}
                width={640}
            >
                <p className="mb-4 text-sm text-muted-foreground">
                    {t(
                        "仅覆盖导航标题、图标、排序和可见性；路由与权限编码仍由模块清单维护。",
                        "Override only the navigation title, icon, order, and visibility. The module manifest continues to own the route and permission code.",
                    )}
                </p>
                <Form layout="vertical" onFinish={submit}>
                    <div className="grid gap-2 md:grid-cols-2">
                        <Form.Item label={t("菜单名称", "Menu name")} required>
                            <Input
                                id="menu-name"
                                value={name}
                                placeholder={t("请输入菜单名称", "Enter a menu name")}
                                onChange={(event) => setName(event.target.value)}
                            />
                        </Form.Item>
                        <Form.Item label={t("权限编码", "Permission code")} required>
                            <Input id="menu-code" value={record.code} disabled />
                        </Form.Item>
                    </div>
                    {record.path ? (
                        <Form.Item label={t("路由路径", "Route path")}>
                            <Input id="menu-path" value={record.path} disabled />
                        </Form.Item>
                    ) : null}
                    <div className="grid gap-2 md:grid-cols-2">
                        <div className="grid gap-2">
                            <label htmlFor="menu-status">{t("状态", "Status")}</label>
                            <Select
                                id="menu-status"
                                value={status}
                                onChange={setStatus}
                                style={{ width: "100%" }}
                                options={getEnableOptions().map((item) => ({
                                    value: String(item.value),
                                    label: item.label,
                                }))}
                            />
                        </div>
                        <div className="grid gap-2">
                            <label htmlFor="menu-icon">{t("图标", "Icon")}</label>
                            <Select
                                id="menu-icon"
                                value={icon || undefined}
                                onChange={setIcon}
                                allowClear
                                style={{ width: "100%" }}
                                options={getModuleIconOptions().map((item) => ({
                                    value: item.value,
                                    label: item.label,
                                }))}
                                placeholder={t("请选择图标", "Select an icon")}
                            />
                        </div>
                    </div>
                    <div className="grid gap-2">
                        <Form.Item label={t("排序", "Sort order")}>
                            <Input
                                id="menu-sort-order"
                                type="number"
                                min={0}
                                step={1}
                                value={sortOrder}
                                placeholder={t("请输入排序", "Enter a sort order")}
                                onChange={(event) => setSortOrder(event.target.value)}
                            />
                        </Form.Item>
                    </div>
                    <div className="mt-4 flex justify-end gap-2">
                        <Button type="default" onClick={() => setOpen(false)}>
                            {t("取消", "Cancel")}
                        </Button>
                        <Button type="primary" htmlType="submit" loading={submitting}>
                            {t("保存", "Save")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function MenuTypeBadge({ menuType }: { menuType: number }) {
    const menuTypeMeta = {
        1: { label: t("目录", "Directory"), color: "default" as const },
        2: { label: t("菜单", "Menu"), color: "blue" as const },
        3: { label: t("按钮", "Button"), color: "green" as const },
    };
    const meta = menuTypeMeta[menuType as keyof typeof menuTypeMeta] ?? {
        label: t("未知", "Unknown"),
        color: "default" as const,
    };

    return <Tag color={meta.color}>{meta.label}</Tag>;
}

function MenuStatusBadge({ status }: { status: number }) {
    const statusMeta = {
        1: { label: t("启用", "Enabled"), color: "green" as const },
        2: { label: t("禁用", "Disabled"), color: "default" as const },
    };
    const meta = statusMeta[status as keyof typeof statusMeta] ?? {
        label: t("未知", "Unknown"),
        color: "default" as const,
    };

    return <Tag color={meta.color}>{meta.label}</Tag>;
}
