import { EditOutlined, PlusOutlined, StopOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Form, Input, Modal, Select, Tag } from "antd";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { menuQueryOptions } from "@/api/system/menu/query-options";
import { AuthWrap } from "@/components/auth";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { getEnableOptions, getMenuTypeOptions, getModuleIconOptions } from "@/constant/options";
import { localizeBuiltInMenuName } from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";

export const Route = createFileRoute("/system/menu")({
    component: MenuPage,
});

type FlatMenuItem = Menu.Item & {
    depth: number;
    children?: Menu.Item[];
};

function MenuPage() {
    const locale = useLocale();
    const [current, setCurrent] = useState(1);
    const [nameFilter, setNameFilter] = useState("");
    const [codeFilter, setCodeFilter] = useState("");
    const [statusFilter, setStatusFilter] = useState("all");
    const { data, error, isPending, isFetching, refetch } = useQuery({
        queryKey: ["system", "menu"],
        queryFn: () => systemAPI.menu.list({}),
    });
    const rows = useMemo(() => flattenMenuTree(data?.data ?? []), [data?.data]);

    const tableRows = useMemo(() => {
        const nameQuery = nameFilter.trim().toLowerCase();
        const codeQuery = codeFilter.trim().toLowerCase();
        return rows.filter((item) => {
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
    }, [codeFilter, nameFilter, rows, statusFilter]);

    const resetFilters = () => {
        setNameFilter("");
        setCodeFilter("");
        setStatusFilter("all");
        setCurrent(1);
    };

    const columns: ProColumns<FlatMenuItem>[] = useMemo(
        () => [
            {
                title: t("名称", "Name"),
                dataIndex: "name",
                key: "name",
                width: 260,
                render: (_: unknown, row: FlatMenuItem) => (
                    <span className="font-medium" style={{ paddingLeft: `${row.depth * 18}px` }}>
                        {row.depth > 0 ? (
                            <span className="mr-2 text-muted-foreground">└</span>
                        ) : null}
                        {localizeBuiltInMenuName(row)}
                    </span>
                ),
            },
            {
                title: t("路径", "Path"),
                dataIndex: "path",
                key: "path",
                width: 220,
                render: (_: unknown, row: FlatMenuItem) => <span>{row.path || "-"}</span>,
            },
            {
                title: t("权限编码", "Permission code"),
                dataIndex: "code",
                key: "code",
                width: 220,
                render: (_: unknown, row: FlatMenuItem) => <Tag bordered>{row.code}</Tag>,
            },
            {
                title: t("菜单类型", "Menu type"),
                dataIndex: "menuType",
                key: "menuType",
                width: 110,
                render: (_: unknown, row: FlatMenuItem) => (
                    <MenuTypeBadge menuType={row.menuType} />
                ),
            },
            {
                title: t("状态", "Status"),
                dataIndex: "status",
                key: "status",
                width: 110,
                render: (_: unknown, row: FlatMenuItem) => <MenuStatusBadge status={row.status} />,
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
                render: (_: unknown, row: FlatMenuItem) => formatDateTime(row.updatedAt),
            },
            {
                title: t("操作", "Actions"),
                key: "actions",
                width: 128,
                fixed: "right",
                render: (_: unknown, row: FlatMenuItem) => (
                    <MenuActions record={row} onSuccess={refresh} />
                ),
            },
        ],
        [locale],
    );

    const refresh = () => {
        void refetch();
    };

    if (!rows.length && isPending) {
        return (
            <PageCard
                title={t("菜单管理", "Menu management")}
                description={t(
                    "管理路由菜单、权限编码和按钮操作。",
                    "Manage route menus, permission codes, and button actions.",
                )}
                actions={
                    <AuthWrap code="system:menu:create">
                        <MenuDialog mode="create" onSuccess={refresh}>
                            <Button type="primary" icon={<PlusOutlined />}>
                                {t("新建菜单", "New menu")}
                            </Button>
                        </MenuDialog>
                    </AuthWrap>
                }
                toolbar={
                    <div className="grid gap-3 md:grid-cols-4">
                        <Input
                            aria-label={t("菜单名称", "Menu name")}
                            value={nameFilter}
                            onChange={(event) => setNameFilter(event.target.value)}
                            placeholder={t("菜单名称", "Menu name")}
                        />
                        <Input
                            aria-label={t("权限编码", "Permission code")}
                            value={codeFilter}
                            onChange={(event) => setCodeFilter(event.target.value)}
                            placeholder={t("权限编码", "Permission code")}
                        />
                        <Select
                            value={statusFilter}
                            className="w-full"
                            onChange={setStatusFilter}
                            options={[
                                { value: "all", label: t("全部", "All") },
                                ...getEnableOptions().map((item) => ({
                                    value: String(item.value),
                                    label: item.label,
                                })),
                            ]}
                            aria-label={t("状态", "Status")}
                        />
                        <Button type="default" onClick={resetFilters}>
                            {t("重置", "Reset")}
                        </Button>
                    </div>
                }
            >
                <DataState kind="loading" title={t("正在加载菜单", "Loading menus")} />
            </PageCard>
        );
    }

    if (!rows.length && error) {
        return (
            <PageCard
                title={t("菜单管理", "Menu management")}
                description={t(
                    "管理路由菜单、权限编码和按钮操作。",
                    "Manage route menus, permission codes, and button actions.",
                )}
                actions={
                    <AuthWrap code="system:menu:create">
                        <MenuDialog mode="create" onSuccess={refresh}>
                            <Button type="primary" icon={<PlusOutlined />}>
                                {t("新建菜单", "New menu")}
                            </Button>
                        </MenuDialog>
                    </AuthWrap>
                }
                toolbar={
                    <div className="grid gap-3 md:grid-cols-4">
                        <Input
                            aria-label={t("菜单名称", "Menu name")}
                            value={nameFilter}
                            onChange={(event) => setNameFilter(event.target.value)}
                            placeholder={t("菜单名称", "Menu name")}
                        />
                        <Input
                            aria-label={t("权限编码", "Permission code")}
                            value={codeFilter}
                            onChange={(event) => setCodeFilter(event.target.value)}
                            placeholder={t("权限编码", "Permission code")}
                        />
                        <Select
                            value={statusFilter}
                            className="w-full"
                            onChange={setStatusFilter}
                            options={[
                                { value: "all", label: t("全部", "All") },
                                ...getEnableOptions().map((item) => ({
                                    value: String(item.value),
                                    label: item.label,
                                })),
                            ]}
                            aria-label={t("状态", "Status")}
                        />
                        <Button type="default" onClick={resetFilters}>
                            {t("重置", "Reset")}
                        </Button>
                    </div>
                }
            >
                <DataState
                    kind="error"
                    title={t("菜单加载失败", "Failed to load menus")}
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

    return (
        <PageCard
            title={t("菜单管理", "Menu management")}
            description={t(
                "管理路由菜单、权限编码和按钮操作。",
                "Manage route menus, permission codes, and button actions.",
            )}
            actions={
                <AuthWrap code="system:menu:create">
                    <MenuDialog mode="create" onSuccess={refresh}>
                        <Button type="primary" icon={<PlusOutlined />}>
                            {t("新建菜单", "New menu")}
                        </Button>
                    </MenuDialog>
                </AuthWrap>
            }
            toolbar={
                <form
                    className="grid gap-3 md:grid-cols-4"
                    onSubmit={(event) => {
                        event.preventDefault();
                        setCurrent(1);
                    }}
                >
                    <Input
                        aria-label={t("菜单名称", "Menu name")}
                        value={nameFilter}
                        onChange={(event) => setNameFilter(event.target.value)}
                        placeholder={t("菜单名称", "Menu name")}
                    />
                    <Input
                        aria-label={t("权限编码", "Permission code")}
                        value={codeFilter}
                        onChange={(event) => setCodeFilter(event.target.value)}
                        placeholder={t("权限编码", "Permission code")}
                    />
                    <Select
                        value={statusFilter}
                        className="w-full"
                        onChange={setStatusFilter}
                        options={[
                            { value: "all", label: t("全部", "All") },
                            ...getEnableOptions().map((item) => ({
                                value: String(item.value),
                                label: item.label,
                            })),
                        ]}
                        aria-label={t("状态", "Status")}
                    />
                    <div className="flex gap-2">
                        <Button type="default" onClick={resetFilters} disabled={isFetching}>
                            {t("重置", "Reset")}
                        </Button>
                        <Button type="primary" htmlType="submit" disabled={isFetching}>
                            {t("查询", "Search")}
                        </Button>
                    </div>
                </form>
            }
        >
            {error ? (
                <DataState
                    kind="error"
                    title={t("菜单加载失败", "Failed to load menus")}
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
            ) : null}
            <DataTableShell>
                <ProTable<FlatMenuItem>
                    rowKey="id"
                    columns={columns}
                    dataSource={tableRows}
                    search={false}
                    loading={isFetching}
                    options={false}
                    pagination={{
                        current,
                        pageSize: 20,
                        total: tableRows.length,
                        showSizeChanger: false,
                        onChange: (page) => setCurrent(page),
                    }}
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

function MenuActions({ record, onSuccess }: { record: Menu.Item; onSuccess: () => void }) {
    const canEdit = !record.moduleId || Boolean(record.moduleMenuCode);

    return (
        <div className="flex justify-end gap-2">
            {canEdit ? (
                <AuthWrap code="system:menu:update">
                    <MenuDialog mode="edit" initialValues={record} onSuccess={onSuccess}>
                        <Button
                            type="text"
                            icon={<EditOutlined />}
                            aria-label={t("编辑菜单", "Edit menu")}
                        />
                    </MenuDialog>
                </AuthWrap>
            ) : null}
            {!record.isSystem && !record.moduleId ? (
                <AuthWrap code="system:menu:delete">
                    <DisableMenuDialog record={record} onSuccess={onSuccess} />
                </AuthWrap>
            ) : null}
        </div>
    );
}

interface MenuDialogProps {
    initialValues?: Partial<Menu.Item>;
    mode?: "create" | "edit";
    children: ReactNode;
    onSuccess?: () => void;
}

const MenuDialog = ({ children, initialValues, mode = "create", onSuccess }: MenuDialogProps) => {
    const queryClient = useQueryClient();
    const isModuleOwned = Boolean(initialValues?.moduleId);
    const [open, setOpen] = useState(false);
    const [parentId, setParentId] = useState("0");
    const [name, setName] = useState("");
    const [code, setCode] = useState("");
    const [menuType, setMenuType] = useState("1");
    const [status, setStatus] = useState("1");
    const [sortOrder, setSortOrder] = useState("0");
    const [icon, setIcon] = useState("");
    const [submitting, setSubmitting] = useState(false);
    const { data: menuOptions = [] } = useQuery({
        ...menuQueryOptions.options(),
        enabled: open && !isModuleOwned,
    });
    const selectableParents = useMemo(
        () => menuOptions.filter((option) => option.value !== initialValues?.id),
        [initialValues?.id, menuOptions],
    );

    useEffect(() => {
        if (open) {
            setParentId(String(initialValues?.parentId ?? 0));
            setName(initialValues?.name ?? "");
            setCode(initialValues?.code ?? "");
            setMenuType(String(initialValues?.menuType ?? 1));
            setStatus(String(initialValues?.status ?? 1));
            setSortOrder(String(initialValues?.sortOrder ?? 0));
            setIcon(initialValues?.icon ?? "");
        }
    }, [initialValues, open]);

    const submit = async () => {
        const trimmedName = name.trim();
        const trimmedCode = code.trim();
        const parsedSortOrder = Number(sortOrder);

        if (!trimmedName) {
            appMessage.error(t("请输入菜单名称", "Enter a menu name."));
            return;
        }
        if (!trimmedCode) {
            appMessage.error(t("请输入权限编码", "Enter a permission code."));
            return;
        }
        if (!Number.isInteger(parsedSortOrder) || parsedSortOrder < 0) {
            appMessage.error(
                t("排序必须是非负整数", "The sort order must be a non-negative integer."),
            );
            return;
        }

        const payload = {
            parentId: Number(parentId),
            name: trimmedName,
            code: trimmedCode,
            menuType: Number(menuType),
            sortOrder: parsedSortOrder,
            status: Number(status),
            icon: icon || null,
        };

        setSubmitting(true);
        try {
            if (mode === "create") {
                await systemAPI.menu.create(payload);
                appMessage.success(t("菜单已创建", "Menu created."));
            } else if (initialValues?.id) {
                await systemAPI.menu.update(initialValues.id, payload);
                appMessage.success(t("菜单已更新", "Menu updated."));
            }
            if (isModuleOwned) {
                await queryClient.invalidateQueries({
                    queryKey: ["system", "modules", "navigation"],
                });
            }
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
                title={
                    mode === "create" ? t("创建菜单", "Create menu") : t("编辑菜单", "Edit menu")
                }
                width={760}
            >
                <p className="mb-4 text-sm text-muted-foreground">
                    {isModuleOwned
                        ? t(
                              "覆盖标题、图标、排序和可见性。模块标识仍与清单保持同步。",
                              "Override the title, icon, sort order, and visibility. The module identifier remains synchronized with the manifest.",
                          )
                        : t(
                              "配置菜单层级、权限编码和显示顺序。",
                              "Configure the menu hierarchy, permission code, and display order.",
                          )}
                </p>
                <Form layout="vertical" onFinish={submit}>
                    <div className="grid gap-2">
                        <label htmlFor="menu-parent">{t("上级菜单", "Parent menu")}</label>
                        <Select
                            id="menu-parent"
                            value={parentId}
                            onChange={setParentId}
                            disabled={isModuleOwned}
                            style={{ width: "100%" }}
                            options={[
                                { value: "0", label: t("顶级菜单", "Top menu") },
                                ...selectableParents.map((item) => ({
                                    value: String(item.value),
                                    label: item.label,
                                })),
                            ]}
                        />
                    </div>
                    <div className="grid gap-2 md:grid-cols-2">
                        <Form.Item label={t("菜单名称", "Menu name")} required>
                            <Input
                                id="menu-name"
                                value={name}
                                placeholder={t("请输入菜单名称", "Enter a menu name")}
                                onChange={(event) => setName(event.target.value)}
                                disabled={false}
                            />
                        </Form.Item>
                        <Form.Item label={t("权限编码", "Permission code")} required>
                            <Input
                                id="menu-code"
                                value={code}
                                placeholder={t(
                                    "请输入权限编码（如 system:menu:list）",
                                    "Enter a permission code (for example, system:menu:list)",
                                )}
                                onChange={(event) => setCode(event.target.value)}
                                disabled={isModuleOwned}
                            />
                        </Form.Item>
                    </div>
                    {isModuleOwned && initialValues?.path ? (
                        <Form.Item label={t("路由路径", "Route path")}>
                            <Input id="menu-path" value={initialValues.path} disabled />
                        </Form.Item>
                    ) : null}
                    <div className="grid gap-2 md:grid-cols-2">
                        <div className="grid gap-2">
                            <label htmlFor="menu-type">{t("类型", "Type")}</label>
                            <Select
                                id="menu-type"
                                value={menuType}
                                onChange={setMenuType}
                                disabled={isModuleOwned}
                                style={{ width: "100%" }}
                                options={getMenuTypeOptions().map((item) => ({
                                    value: String(item.value),
                                    label: item.label,
                                }))}
                            />
                        </div>
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
                    </div>
                    {isModuleOwned ? (
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
                    ) : null}
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
                            {mode === "create" ? t("创建", "Create") : t("保存", "Save")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
};

function DisableMenuDialog({ record, onSuccess }: { record: Menu.Item; onSuccess: () => void }) {
    const submit = async () => {
        await systemAPI.menu.delete(record.id);
        appMessage.success(t("菜单已禁用", "Menu disabled."));
        onSuccess();
    };

    return (
        <ConfirmDialog
            trigger={
                <Button
                    type="text"
                    icon={<StopOutlined />}
                    aria-label={t("禁用菜单", "Disable menu")}
                />
            }
            title={t("禁用菜单", "Disable menu")}
            description={t(
                `菜单 ${record.name} 将被禁用，是否继续？`,
                `Menu ${record.name} will be disabled. Continue?`,
            )}
            confirmLabel={t("禁用", "Disable")}
            destructive
            onConfirm={submit}
        />
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

function flattenMenuTree(items: Menu.Item[], depth = 0): FlatMenuItem[] {
    return items.flatMap((item) => {
        const { children, ...rest } = item;
        return [{ ...rest, depth }, ...flattenMenuTree(children ?? [], depth + 1)];
    });
}
