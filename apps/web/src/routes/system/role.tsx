import { DeleteOutlined, EditOutlined, PlusOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
    Button,
    Checkbox,
    Form,
    Input,
    Modal,
    Popconfirm,
    Select,
    Tag,
    Tree,
    type FormProps,
} from "antd";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { menuQueryOptions } from "@/api/system/menu/query-options";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { getEnableOptions } from "@/constant/options";
import {
    localizeBuiltInMenuName,
    localizeBuiltInRoleDescription,
    localizeBuiltInRoleName,
} from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t } from "@/lib/i18n";

const OWNER_ROLE_CODE = "owner";
const BUILTIN_ROLE_CODES = new Set([OWNER_ROLE_CODE, "admin", "viewer"]);
const PAGE_SIZE = 20;

export const Route = createFileRoute("/system/role")({
    component: RolePage,
});

interface RoleFormValues {
    name: string;
    code: string;
    status: string;
    description: string;
}

function RolePage() {
    const [currentPage, setCurrentPage] = useState(1);
    const [roleName, setRoleName] = useState("");
    const [roleCode, setRoleCode] = useState("");
    const [status, setStatus] = useState("all");
    const [filters, setFilters] = useState({
        roleName: "",
        roleCode: "",
        status: "all",
    });
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
    });

    const rows = data?.data ?? [];
    const total = data?.total ?? 0;

    const search = (event: React.FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setCurrentPage(1);
        setFilters({
            roleName: roleName.trim(),
            roleCode: roleCode.trim(),
            status,
        });
    };

    const reset = () => {
        setRoleName("");
        setRoleCode("");
        setStatus("all");
        setCurrentPage(1);
        setFilters({ roleName: "", roleCode: "", status: "all" });
    };

    const refresh = () => {
        void refetch();
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
            title: t("更新时间", "Updated at"),
            key: "updatedAt",
            width: 220,
            render: (_: unknown, row: Role.Item) => formatDateTime(row.updatedAt),
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 160,
            align: "right",
            render: (_: unknown, row: Role.Item) => (
                <RoleActions record={row} onSuccess={refresh} />
            ),
        },
    ];

    if (!data && isPending) {
        return (
            <PageCard
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
            toolbar={
                <form className="grid gap-3 md:grid-cols-4" onSubmit={search}>
                    <Input
                        aria-label={t("角色名称", "Role name")}
                        value={roleName}
                        placeholder={t("角色名称", "Role name")}
                        onChange={(event) => setRoleName(event.target.value)}
                    />
                    <Input
                        aria-label={t("角色编码", "Role code")}
                        value={roleCode}
                        placeholder={t("角色编码", "Role code")}
                        onChange={(event) => setRoleCode(event.target.value)}
                    />
                    <Select
                        className="w-full"
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
                    <div className="flex gap-2">
                        <Button type="primary" htmlType="submit" disabled={isFetching}>
                            {t("查询", "Search")}
                        </Button>
                        <Button type="default" onClick={reset} disabled={isFetching}>
                            {t("重置", "Reset")}
                        </Button>
                    </div>
                </form>
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

function RoleActions({ record, onSuccess }: { record: Role.Item; onSuccess: () => void }) {
    if (isBuiltInRoleCode(record.code)) {
        return null;
    }

    return (
        <div className="flex justify-end gap-2">
            <AuthWrap code="system:role:update">
                <RoleDialog mode="edit" record={record} onSuccess={onSuccess}>
                    <Button
                        type="text"
                        size="small"
                        aria-label={t("编辑角色", "Edit role")}
                        icon={<EditOutlined />}
                    />
                </RoleDialog>
            </AuthWrap>
            <AuthWrap code="system:role:delete">
                <Popconfirm
                    title={t("删除角色", "Delete role")}
                    description={
                        <span>
                            {t(
                                `此操作无法撤销。确定删除角色 ${record.name}？`,
                                `This action cannot be undone. Delete role ${record.name}?`,
                            )}
                        </span>
                    }
                    okText={t("删除", "Delete")}
                    cancelText={t("取消", "Cancel")}
                    onConfirm={() => onSuccessDelete(record.id, onSuccess)}
                >
                    <Button type="text" size="small" danger icon={<DeleteOutlined />} />
                </Popconfirm>
            </AuthWrap>
        </div>
    );
}

async function onSuccessDelete(id: number, onSuccess: () => void) {
    await systemAPI.role.delete(id);
    appMessage.success(t("角色已删除", "Role deleted."));
    onSuccess();
}

interface RoleDialogProps {
    record?: Partial<Role.Item>;
    mode?: "create" | "edit";
    children: ReactNode;
    onSuccess?: () => void;
}

function RoleDialog({ children, record, mode = "create", onSuccess }: RoleDialogProps) {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const [menuIds, setMenuIds] = useState<number[]>([]);
    const [permissionSearch, setPermissionSearch] = useState("");
    const [form] = Form.useForm<RoleFormValues>();
    const watchedName = Form.useWatch("name", form) ?? "";
    const watchedCode = Form.useWatch("code", form) ?? "";

    const trimmedName = String(watchedName).trim();
    const trimmedCode = String(watchedCode).trim();

    const {
        data: menuOptions,
        error: permissionError,
        isError: permissionLoadFailed,
        isPending: permissionLoading,
        isFetching: permissionRefreshing,
        refetch: refetchPermissions,
    } = useQuery({
        ...menuQueryOptions.options(),
        enabled: open,
    });

    const permissionInitialError =
        permissionLoadFailed && menuOptions === undefined ? permissionError : null;
    const permissionOptions = useMemo(
        () =>
            (menuOptions ?? [])
                .filter((option) => option.value !== 0 && isAssignableRolePermission(option.code))
                .map((option) => ({
                    value: option.value,
                    title: localizeBuiltInMenuName({
                        name: option.label,
                        code: option.code,
                        isSystem: option.isSystem,
                        moduleId: option.moduleId,
                        moduleMenuCode: option.moduleMenuCode,
                    }),
                    code: option.code,
                })),
        [menuOptions],
    );

    const permissionReady = menuOptions !== undefined && permissionOptions.length > 0;
    const filteredPermissions = useMemo(() => {
        const query = permissionSearch.trim().toLowerCase();
        if (!query) {
            return permissionOptions;
        }
        return permissionOptions.filter(
            (option) =>
                option.title.toLowerCase().includes(query) ||
                option.code.toLowerCase().includes(query),
        );
    }, [permissionOptions, permissionSearch]);
    const filteredPermissionKeys = useMemo(
        () => filteredPermissions.map((item) => String(item.value)),
        [filteredPermissions],
    );
    const selectedPermissionKeys = useMemo(() => new Set(menuIds.map(String)), [menuIds]);
    const isAllChecked =
        permissionReady &&
        permissionOptions.length > 0 &&
        filteredPermissionKeys.length > 0 &&
        filteredPermissionKeys.every((key) => selectedPermissionKeys.has(key));

    const submitDisabled =
        submitting ||
        !permissionReady ||
        trimmedName.length < 2 ||
        trimmedName.length > 50 ||
        trimmedCode.length < 2 ||
        trimmedCode.length > 50 ||
        !/^[a-zA-Z_]+$/.test(trimmedCode) ||
        menuIds.length === 0;

    useEffect(() => {
        if (!open) {
            return;
        }

        const name = record?.name ?? "";
        const code = record?.code ?? "";
        const description = record?.description ?? "";
        const status = String(record?.status ?? 1);
        const nextMenuIds = record?.menus?.map((menu) => Number(menu.value)) ?? [];

        setMenuIds(nextMenuIds);
        setPermissionSearch("");
        form.setFieldsValue({
            name,
            code,
            status,
            description,
        });
    }, [record, open, form]);

    const submit: FormProps<RoleFormValues>["onFinish"] = async (values) => {
        const trimmedName = values.name.trim();
        const trimmedCode = values.code.trim();
        const trimmedDescription = values.description?.trim() ?? "";

        if (trimmedName.length < 2 || trimmedName.length > 50) {
            appMessage.error(
                t("角色名称必须为 2-50 个字符", "The role name must be 2–50 characters."),
            );
            return;
        }
        if (trimmedCode.length < 2 || trimmedCode.length > 50) {
            appMessage.error(
                t("角色编码必须为 2-50 个字符", "The role code must be 2–50 characters."),
            );
            return;
        }
        if (!/^[a-zA-Z_]+$/.test(trimmedCode)) {
            appMessage.error(
                t(
                    "角色编码只能包含字母和下划线",
                    "The role code may contain only letters and underscores.",
                ),
            );
            return;
        }
        if (menuIds.length === 0) {
            appMessage.error(t("请至少选择一个权限", "Select at least one permission."));
            return;
        }

        setSubmitting(true);
        try {
            if (mode === "create") {
                await systemAPI.role.create({
                    name: trimmedName,
                    code: trimmedCode,
                    status: Number(values.status ?? 1),
                    description: trimmedDescription || undefined,
                    menuIds,
                });
                appMessage.success(t("角色已创建", "Role created."));
            } else if (record?.id) {
                await systemAPI.role.update(record.id, {
                    name: trimmedName,
                    code: trimmedCode,
                    status: Number(values.status ?? 1),
                    description: trimmedDescription || undefined,
                    menuIds,
                });
                appMessage.success(t("角色已更新", "Role updated."));
            }
            onSuccess?.();
            form.resetFields();
            setOpen(false);
            setMenuIds([]);
        } finally {
            setSubmitting(false);
        }
    };

    const selectedMenuKeys = useMemo(() => menuIds.map(String), [menuIds]);

    return (
        <>
            <span className="inline-flex" onClick={() => setOpen(true)}>
                {children}
            </span>
            <Modal
                open={open}
                destroyOnHidden
                onCancel={() => {
                    form.resetFields();
                    setMenuIds([]);
                    setOpen(false);
                }}
                title={
                    mode === "create" ? t("创建角色", "Create role") : t("编辑角色", "Edit role")
                }
                footer={null}
                width={900}
            >
                <Form
                    form={form}
                    layout="vertical"
                    requiredMark={false}
                    initialValues={{
                        name: record?.name ?? "",
                        code: record?.code ?? "",
                        status: String(record?.status ?? 1),
                        description: record?.description ?? "",
                    }}
                    onFinish={submit}
                >
                    <div className="grid gap-4 md:grid-cols-2">
                        <Form.Item
                            name="name"
                            label={t("角色名称", "Role name")}
                            rules={[
                                {
                                    required: true,
                                    message: t("请输入角色名称", "Enter a role name"),
                                },
                            ]}
                        >
                            <Input placeholder={t("请输入角色名称", "Enter a role name")} />
                        </Form.Item>
                        <Form.Item
                            name="code"
                            label={t("角色编码", "Role code")}
                            rules={[
                                {
                                    required: true,
                                    message: t("请输入角色编码", "Enter a role code"),
                                },
                            ]}
                        >
                            <Input placeholder={t("请输入角色编码", "Enter a role code")} />
                        </Form.Item>
                    </div>
                    <Form.Item name="status" label={t("状态", "Status")}>
                        <Select
                            options={getEnableOptions().map((item) => ({
                                value: String(item.value),
                                label: item.label,
                            }))}
                        />
                    </Form.Item>
                    <PermissionPicker
                        permissions={filteredPermissions}
                        checkedValues={selectedMenuKeys}
                        loading={permissionLoading && menuOptions === undefined}
                        error={permissionInitialError}
                        permissionReady={permissionReady}
                        permissionRefreshing={permissionRefreshing}
                        permissionSearch={permissionSearch}
                        isAllChecked={isAllChecked}
                        onCheck={(keys) => {
                            const checkedKeys = new Set(keys);
                            const nextChecked = new Set(selectedPermissionKeys);
                            for (const key of filteredPermissionKeys) {
                                if (checkedKeys.has(key)) {
                                    nextChecked.add(key);
                                } else {
                                    nextChecked.delete(key);
                                }
                            }
                            setMenuIds(Array.from(nextChecked).map((item) => Number(item)));
                        }}
                        onSearchChange={setPermissionSearch}
                        onSelectAllChange={(checked) => {
                            const nextChecked = new Set(selectedPermissionKeys);
                            if (checked) {
                                for (const key of filteredPermissionKeys) {
                                    nextChecked.add(key);
                                }
                            } else {
                                for (const key of filteredPermissionKeys) {
                                    nextChecked.delete(key);
                                }
                            }
                            setMenuIds(Array.from(nextChecked).map((item) => Number(item)));
                        }}
                        onRetry={async () => {
                            await refetchPermissions();
                        }}
                    />
                    <Form.Item
                        name="description"
                        label={t("描述", "Description")}
                        className="!mb-1"
                    >
                        <Input.TextArea
                            maxLength={200}
                            placeholder={t("请输入角色描述", "Enter a role description")}
                            showCount
                            rows={4}
                        />
                    </Form.Item>
                    <div className="flex items-center justify-end gap-2">
                        <Button
                            onClick={() => {
                                form.resetFields();
                                setMenuIds([]);
                                setOpen(false);
                            }}
                        >
                            {t("取消", "Cancel")}
                        </Button>
                        <Button
                            type="primary"
                            htmlType="submit"
                            loading={submitting}
                            disabled={submitDisabled}
                        >
                            {submitting
                                ? mode === "create"
                                    ? t("创建中", "Creating")
                                    : t("保存中", "Saving")
                                : mode === "create"
                                  ? t("创建", "Create")
                                  : t("保存", "Save")}
                        </Button>
                    </div>
                </Form>
            </Modal>
        </>
    );
}

function PermissionPicker({
    permissions,
    checkedValues,
    loading,
    error,
    permissionReady,
    permissionRefreshing,
    permissionSearch,
    isAllChecked,
    onCheck,
    onSearchChange,
    onSelectAllChange,
    onRetry,
}: {
    permissions: { value: number; title: string; code: string }[];
    checkedValues: string[];
    loading: boolean;
    error: unknown;
    permissionReady: boolean;
    permissionRefreshing: boolean;
    permissionSearch: string;
    isAllChecked: boolean;
    onCheck: (checkedValues: string[]) => void;
    onSearchChange: (value: string) => void;
    onSelectAllChange: (checked: boolean) => void;
    onRetry?: () => Promise<void> | void;
}) {
    const treeData = useMemo(
        () =>
            permissions.map((item) => ({
                title: (
                    <div className="grid gap-0.5">
                        <span>{item.title}</span>
                        <span className="text-xs text-muted-foreground">{item.code}</span>
                    </div>
                ),
                key: String(item.value),
                isLeaf: true,
            })),
        [permissions],
    );
    const permissionSet = useMemo(
        () => new Set(permissions.map((item) => String(item.value))),
        [permissions],
    );
    const visibleCheckedValues = checkedValues.filter((key) => permissionSet.has(key));

    return (
        <div className="grid gap-2">
            <div className="flex flex-wrap items-center justify-between gap-3">
                <span className="font-medium">{t("权限", "Permissions")}</span>
                {permissionReady ? (
                    <Checkbox
                        checked={isAllChecked}
                        onChange={(event) => onSelectAllChange(event.target.checked)}
                    >
                        {t("全选当前", "Select all")}
                    </Checkbox>
                ) : null}
                <span className="text-sm text-muted-foreground">
                    {t(`已选择 ${checkedValues.length} 项`, `${checkedValues.length} selected`)}
                </span>
            </div>
            <div className="h-72 overflow-auto rounded-md border p-3">
                {loading ? (
                    <DataState
                        compact
                        kind="loading"
                        title={t("正在加载权限", "Loading permissions")}
                    />
                ) : error ? (
                    <DataState
                        compact
                        kind="error"
                        title={t("权限加载失败", "Failed to load permissions")}
                        description={t(
                            "请重新加载；若仍无权限，请联系所有者。",
                            "Reload, or contact an owner if access is still unavailable.",
                        )}
                        action={
                            <Button
                                type="default"
                                onClick={() => void onRetry?.()}
                                disabled={permissionRefreshing}
                            >
                                {t("重新加载", "Reload")}
                            </Button>
                        }
                    />
                ) : !permissionReady ? (
                    <DataState
                        compact
                        kind="empty"
                        title={t("暂无可分配权限", "No permissions available to assign")}
                    />
                ) : (
                    <div className="grid gap-3">
                        <Input
                            value={permissionSearch}
                            placeholder={t("搜索权限", "Search permissions")}
                            onChange={(event) => onSearchChange(event.target.value)}
                        />
                        {permissions.length === 0 ? (
                            <div className="text-sm text-muted-foreground">
                                {t("未找到匹配权限。", "No matching permissions found.")}
                            </div>
                        ) : (
                            <Tree
                                checkable
                                checkedKeys={visibleCheckedValues}
                                treeData={treeData}
                                onCheck={(keys) => {
                                    const checked = Array.isArray(keys)
                                        ? keys
                                        : "checked" in keys
                                          ? keys.checked
                                          : [];
                                    onCheck(checked.map((item) => String(item)));
                                }}
                            />
                        )}
                    </div>
                )}
            </div>
        </div>
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

function isBuiltInRoleCode(code: string) {
    return BUILTIN_ROLE_CODES.has(code);
}

function isAssignableRolePermission(code: string) {
    const ownerOnlyRoots = ["system:module", "system:status", "manage:task", "manage:deploy"];
    if (code === "*") {
        return false;
    }
    if (ownerOnlyRoots.some((root) => code === root || code.startsWith(`${root}:`))) {
        return false;
    }
    if (!code.endsWith(":*")) {
        return true;
    }
    const wildcardPrefix = code.slice(0, -1);
    return !ownerOnlyRoots.some((root) => `${root}:`.startsWith(wildcardPrefix));
}
