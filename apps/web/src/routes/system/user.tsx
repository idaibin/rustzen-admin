import { EditOutlined, MoreOutlined, PlusOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import {
    Avatar,
    Button,
    Checkbox,
    Dropdown,
    Form,
    Input,
    Modal,
    Select,
    Tag,
    type FormProps,
    type MenuProps,
} from "antd";
import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { DataTableShell } from "@/components/table/data-table-shell";
import { getEnableOptions } from "@/constant/options";
import { localizeBuiltInRoleName, localizeBuiltInUserName } from "@/lib/builtin-i18n";
import { formatDateTime } from "@/lib/format-date-time";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

export const Route = createFileRoute("/system/user")({
    component: UserPage,
});

const PAGE_SIZE = 20;

const formatResetPassword = (date = new Date()) => {
    const year = date.getFullYear() % 100;
    const month = date.getMonth() + 1;
    const day = date.getDate();
    return `${String(year).padStart(2, "0")}${String(month).padStart(2, "0")}${String(day).padStart(
        2,
        "0",
    )}`;
};

function getResetPassword(username: string) {
    const trimmedUsername = username.trim();
    if (!trimmedUsername) {
        return `User@${formatResetPassword(new Date())}`;
    }
    const normalized = trimmedUsername.charAt(0).toUpperCase() + trimmedUsername.slice(1);
    return `${normalized}@${formatResetPassword(new Date())}`;
}

interface UserDialogValues {
    username: string;
    email: string;
    realName: string;
    password: string;
    status: string;
    roleIds: number[];
}

function UserPage() {
    const currentUserId = useAuthStore((state) => state.userInfo?.id);
    const [currentPage, setCurrentPage] = useState(1);
    const [username, setUsername] = useState("");
    const [email, setEmail] = useState("");
    const [realName, setRealName] = useState("");
    const [status, setStatus] = useState("all");
    const [filters, setFilters] = useState({
        username: "",
        email: "",
        realName: "",
        status: "all",
    });
    const params = useMemo<User.QueryParams>(
        () => ({
            current: currentPage,
            pageSize: PAGE_SIZE,
            username: filters.username || undefined,
            email: filters.email || undefined,
            realName: filters.realName || undefined,
            status: filters.status,
        }),
        [currentPage, filters],
    );

    const { data, error, isFetching, isPending, refetch } = useQuery({
        queryKey: ["system", "user", params],
        queryFn: () => systemAPI.user.list(params),
    });

    const rows = data?.data ?? [];
    const total = data?.total ?? 0;
    const hasData = data !== undefined;

    const search = (event: FormEvent<HTMLFormElement>) => {
        event.preventDefault();
        setCurrentPage(1);
        setFilters({
            username: username.trim(),
            email: email.trim(),
            realName: realName.trim(),
            status,
        });
    };

    const reset = () => {
        setUsername("");
        setEmail("");
        setRealName("");
        setStatus("all");
        setCurrentPage(1);
        setFilters({ username: "", email: "", realName: "", status: "all" });
    };

    const refresh = () => {
        void refetch();
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

    if (!data && isPending) {
        return (
            <PageCard
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
            toolbar={
                <form className="grid gap-3 md:grid-cols-5" onSubmit={search}>
                    <Input
                        aria-label={t("用户名", "Username")}
                        value={username}
                        placeholder={t("用户名", "Username")}
                        onChange={(event) => setUsername(event.target.value)}
                    />
                    <Input
                        aria-label={t("邮箱", "Email")}
                        value={email}
                        placeholder={t("邮箱", "Email")}
                        onChange={(event) => setEmail(event.target.value)}
                    />
                    <Input
                        aria-label={t("真实姓名", "Real name")}
                        value={realName}
                        placeholder={t("真实姓名", "Real name")}
                        onChange={(event) => setRealName(event.target.value)}
                    />
                    <Select
                        className="w-full"
                        aria-label={t("账号状态", "Account status")}
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
                        <Button type="default" disabled={isFetching} onClick={reset}>
                            {t("重置", "Reset")}
                        </Button>
                    </div>
                </form>
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

function UserActions({
    record,
    currentUserId,
    onSuccess,
}: {
    record: User.Item;
    currentUserId?: number;
    onSuccess: () => void;
}) {
    const locale = useLocale();
    const hasStatusPermission = useAuthStore((state) =>
        state.checkPermissions("system:user:status"),
    );
    const hasPasswordPermission = useAuthStore((state) =>
        state.checkPermissions("system:user:password"),
    );
    const hasDeletePermission = useAuthStore((state) =>
        state.checkPermissions("system:user:delete"),
    );
    const [pendingAction, setPendingAction] = useState<UserActionType | null>(null);
    const [confirmingAction, setConfirmingAction] = useState(false);
    const actionItems = useMemo<NonNullable<MenuProps["items"]>>(
        () =>
            getUserActionItems(
                record,
                hasStatusPermission,
                hasPasswordPermission,
                hasDeletePermission,
            ),
        [
            record.status,
            record.username,
            hasStatusPermission,
            hasPasswordPermission,
            hasDeletePermission,
            locale,
        ],
    );
    const actionConfig = useMemo<UserActionConfig | null>(() => {
        if (!pendingAction) return null;

        if (pendingAction === "status") {
            return {
                title:
                    record.status === 1
                        ? t("禁用用户", "Disable user")
                        : t("启用用户", "Enable user"),
                description:
                    record.status === 1
                        ? t(`确定禁用用户 ${record.username}？`, `Disable user ${record.username}?`)
                        : t(`确定启用用户 ${record.username}？`, `Enable user ${record.username}?`),
                actionLabel: record.status === 1 ? t("禁用", "Disable") : t("启用", "Enable"),
                onConfirm: async () => {
                    await systemAPI.user.status(record.id, record.status === 1 ? 2 : 1);
                },
            };
        }

        if (pendingAction === "password") {
            return {
                title: t("重置密码", "Reset password"),
                description: t(
                    `确定重置用户 ${record.username} 的密码吗？`,
                    `Reset the password for user ${record.username}?`,
                ),
                actionLabel: t("重置密码", "Reset password"),
                onConfirm: async () => {
                    const password = getResetPassword(record.username);
                    await systemAPI.user.password(record.id, password);
                    appMessage.success(
                        t(`密码已重置为 ${password}`, `Password reset to ${password}`),
                    );
                },
            };
        }

        return {
            title: t("删除用户", "Delete user"),
            description: t(
                `确定删除用户 ${record.username}？此操作无法撤销。`,
                `Delete user ${record.username}? This action cannot be undone.`,
            ),
            actionLabel: t("删除用户", "Delete user"),
            destructive: true,
            onConfirm: async () => {
                await systemAPI.user.delete(record.id);
            },
        };
    }, [pendingAction, record.id, record.status, record.username, locale]);

    const executeAction = async () => {
        if (!actionConfig || confirmingAction) return;

        setConfirmingAction(true);
        try {
            await actionConfig.onConfirm();
            onSuccess();
            setPendingAction(null);
        } finally {
            setConfirmingAction(false);
        }
    };

    const hideActionDialog = () => {
        if (confirmingAction) return;
        setPendingAction(null);
    };

    if (record.id === currentUserId || record.isSystem) {
        return null;
    }

    return (
        <div className="flex justify-end gap-2">
            <AuthWrap code="system:user:update">
                <UserDialog mode="edit" initialValues={record} onSuccess={onSuccess}>
                    <Button
                        type="text"
                        size="small"
                        aria-label={t("编辑用户", "Edit user")}
                        icon={<EditOutlined />}
                    />
                </UserDialog>
            </AuthWrap>
            {actionItems.length > 0 ? (
                <Dropdown
                    menu={{
                        items: actionItems,
                        onClick: (event) => {
                            setPendingAction(event.key as UserActionType);
                        },
                    }}
                    trigger={["click"]}
                >
                    <Button
                        type="text"
                        size="small"
                        aria-label={t("更多用户操作", "More user actions")}
                        icon={<MoreOutlined />}
                    />
                </Dropdown>
            ) : null}
            <Modal
                open={pendingAction !== null}
                onCancel={hideActionDialog}
                footer={null}
                title={actionConfig?.title}
                destroyOnHidden
            >
                <p>{actionConfig?.description}</p>
                <div className="mt-4 flex items-center justify-end gap-2">
                    <Button type="default" onClick={hideActionDialog}>
                        {t("取消", "Cancel")}
                    </Button>
                    <Button
                        type="primary"
                        danger={actionConfig?.destructive}
                        loading={confirmingAction}
                        onClick={() => void executeAction()}
                    >
                        {actionConfig?.actionLabel ?? t("确定", "Confirm")}
                    </Button>
                </div>
            </Modal>
        </div>
    );
}

type UserActionType = "status" | "password" | "delete";
type UserActionConfig = {
    title: string;
    description: string;
    actionLabel: string;
    destructive?: boolean;
    onConfirm: () => Promise<void>;
};

function getUserActionItems(
    record: User.Item,
    hasStatusPermission: boolean,
    hasPasswordPermission: boolean,
    hasDeletePermission: boolean,
): NonNullable<MenuProps["items"]> {
    const items: NonNullable<MenuProps["items"]> = [];

    if (hasStatusPermission) {
        items.push({
            key: "status",
            label: <span>{record.status === 1 ? t("禁用", "Disable") : t("启用", "Enable")}</span>,
        });
    }

    if (hasPasswordPermission) {
        items.push({
            key: "password",
            label: <span>{t("重置密码", "Reset password")}</span>,
        });
    }

    if (hasDeletePermission) {
        items.push({
            key: "delete",
            label: <span>{t("删除用户", "Delete user")}</span>,
        });
    }

    return items;
}

const UserDialog = ({
    children,
    initialValues,
    mode = "create",
    onSuccess,
}: {
    initialValues?: Partial<User.Item>;
    mode?: "create" | "edit";
    children: ReactNode;
    onSuccess?: () => void;
}) => {
    const [open, setOpen] = useState(false);
    const [submitting, setSubmitting] = useState(false);
    const [form] = Form.useForm<UserDialogValues>();

    const {
        data: roleOptions,
        error: roleError,
        isError: roleLoadFailed,
        isPending: roleLoading,
        isFetching: roleRefreshing,
        refetch: refetchRoles,
    } = useQuery({
        queryKey: ["system", "roles", "options"],
        queryFn: systemAPI.role.options,
        enabled: open,
    });

    const roleInitialError = roleLoadFailed && roleOptions === undefined ? roleError : null;
    const rolePermissionDenied =
        roleInitialError instanceof Response &&
        (roleInitialError.status === 401 || roleInitialError.status === 403);

    const roleReady = roleOptions !== undefined && roleOptions.length > 0;
    const submitDisabled = submitting || !roleReady;

    useEffect(() => {
        if (!open) {
            return;
        }

        form.setFieldsValue({
            username: initialValues?.username ?? "",
            email: initialValues?.email ?? "",
            realName: initialValues?.realName ?? "",
            password: "",
            status: String(initialValues?.status ?? 1),
            roleIds: initialValues?.roles?.map((role) => role.value) ?? [],
        });
    }, [initialValues, open, form]);

    const submit: FormProps<UserDialogValues>["onFinish"] = async (values) => {
        const trimmedUsername = values.username.trim();
        const trimmedEmail = values.email.trim();
        const trimmedRealName = values.realName.trim();
        const trimmedPassword = (values.password ?? "").trim();
        const roleIds = (values.roleIds ?? []).map((item) => Number(item));

        if (mode === "create" && trimmedUsername.length < 3) {
            appMessage.error(
                t("用户名至少需要 3 个字符", "The username must be at least 3 characters."),
            );
            return;
        }
        if (!trimmedEmail || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmedEmail)) {
            appMessage.error(t("请输入有效的邮箱", "Enter a valid email address."));
            return;
        }
        if (!trimmedRealName) {
            appMessage.error(t("请输入真实姓名", "Enter the real name."));
            return;
        }
        if (mode === "create" && trimmedPassword.length < 6) {
            appMessage.error(
                t("密码至少需要 6 个字符", "The password must be at least 6 characters."),
            );
            return;
        }
        if (roleIds.length === 0) {
            appMessage.error(t("请至少选择一个角色", "Select at least one role."));
            return;
        }

        setSubmitting(true);
        try {
            if (mode === "create") {
                await systemAPI.user.create({
                    username: trimmedUsername,
                    email: trimmedEmail,
                    password: trimmedPassword,
                    realName: trimmedRealName,
                    status: Number(values.status ?? 1),
                    roleIds,
                });
                appMessage.success(t("用户已创建", "User created."));
            } else if (initialValues?.id) {
                await systemAPI.user.update(initialValues.id, {
                    email: trimmedEmail,
                    realName: trimmedRealName,
                    roleIds,
                });
                appMessage.success(t("用户已更新", "User updated."));
            }
            onSuccess?.();
            form.resetFields();
            setOpen(false);
        } finally {
            setSubmitting(false);
        }
    };

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
                    setOpen(false);
                }}
                title={
                    mode === "create" ? t("创建用户", "Create user") : t("编辑用户", "Edit user")
                }
                footer={null}
            >
                <Form
                    form={form}
                    layout="vertical"
                    requiredMark={false}
                    onFinish={submit}
                    initialValues={{
                        username: initialValues?.username ?? "",
                        email: initialValues?.email ?? "",
                        realName: initialValues?.realName ?? "",
                        status: String(initialValues?.status ?? 1),
                        roleIds: initialValues?.roles?.map((role) => role.value) ?? [],
                    }}
                >
                    <Form.Item
                        name="username"
                        label={t("用户名", "Username")}
                        rules={[{ required: true, message: t("请输入用户名", "Enter a username") }]}
                    >
                        <Input
                            placeholder={t("请输入用户名", "Enter a username")}
                            disabled={mode === "edit"}
                        />
                    </Form.Item>
                    <Form.Item
                        name="email"
                        label={t("邮箱", "Email")}
                        rules={[
                            { required: true, message: t("请输入邮箱", "Enter an email address") },
                        ]}
                    >
                        <Input placeholder={t("请输入邮箱", "Enter an email address")} />
                    </Form.Item>
                    <Form.Item
                        name="realName"
                        label={t("真实姓名", "Real name")}
                        rules={[
                            { required: true, message: t("请输入真实姓名", "Enter the real name") },
                        ]}
                    >
                        <Input placeholder={t("请输入真实姓名", "Enter the real name")} />
                    </Form.Item>
                    {mode === "create" && (
                        <Form.Item
                            name="password"
                            label={t("密码", "Password")}
                            rules={[
                                { required: true, message: t("请输入密码", "Enter a password") },
                            ]}
                        >
                            <Input.Password placeholder={t("请输入密码", "Enter a password")} />
                        </Form.Item>
                    )}
                    {mode === "create" && (
                        <Form.Item name="status" label={t("状态", "Status")}>
                            <Select
                                options={getEnableOptions().map((item) => ({
                                    value: String(item.value),
                                    label: item.label,
                                }))}
                            />
                        </Form.Item>
                    )}
                    <Form.Item
                        name="roleIds"
                        label={t("角色", "Roles")}
                        rules={[
                            {
                                required: true,
                                type: "array",
                                min: 1,
                                message: t("请至少选择一个角色", "Select at least one role."),
                            },
                        ]}
                    >
                        <RolePicker
                            options={roleOptions ?? []}
                            loading={roleLoading && roleOptions === undefined}
                            error={roleInitialError}
                            permissionDenied={rolePermissionDenied}
                            onRetry={async () => {
                                await refetchRoles();
                            }}
                            retrying={roleRefreshing}
                        />
                    </Form.Item>
                    <Form.Item className="!mb-0">
                        <div className="flex items-center justify-end gap-2">
                            <Button
                                onClick={() => {
                                    form.resetFields();
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
                    </Form.Item>
                </Form>
            </Modal>
        </>
    );
};

function RolePicker({
    options,
    loading,
    error,
    permissionDenied,
    onRetry,
    retrying,
    value,
    onChange,
}: {
    options: Role.OptionItem[];
    loading: boolean;
    error: unknown;
    permissionDenied: boolean;
    onRetry?: () => Promise<void> | void;
    retrying: boolean;
    value?: Array<number | string>;
    onChange?: (value: Array<number | string>) => void;
}) {
    const normalizeRoleIds = (rawValues: Array<number | string>) =>
        rawValues.map((item) => Number(item)).filter((item) => Number.isFinite(item));

    if (loading) {
        return <DataState compact kind="loading" title={t("正在加载角色", "Loading roles")} />;
    }

    if (error) {
        return (
            <DataState
                compact
                kind={permissionDenied ? "permission" : "error"}
                title={
                    permissionDenied
                        ? t(
                              "角色加载失败或无权限",
                              "Failed to load roles or insufficient permission",
                          )
                        : t("角色加载失败", "Failed to load roles")
                }
                description={t(
                    "角色加载失败，请重试；若仍无权限，请联系管理员。",
                    "Failed to load roles. Retry, or contact an owner if access is still unavailable.",
                )}
                action={
                    <Button type="default" onClick={() => void onRetry?.()} disabled={retrying}>
                        {t("重新加载", "Reload")}
                    </Button>
                }
            />
        );
    }

    if (options.length === 0) {
        return (
            <DataState
                compact
                kind="empty"
                title={t("暂无可分配角色", "No roles available to assign")}
            />
        );
    }

    return (
        <Checkbox.Group
            value={normalizeRoleIds(value ?? [])}
            options={options.map((role) => ({
                value: role.value,
                label: role.isSystem ? localizeBuiltInRoleName(role.code, role.label) : role.label,
            }))}
            onChange={(values) => onChange?.(normalizeRoleIds(values as Array<number | string>))}
        />
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
