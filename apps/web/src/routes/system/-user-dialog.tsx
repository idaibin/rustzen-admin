import { useQuery } from "@tanstack/react-query";
import { Button, Checkbox, Form, Input, Modal, Select, type FormProps } from "antd";
import { useEffect, useState, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { getEnableOptions } from "@/constant/options";
import { localizeBuiltInRoleName } from "@/lib/builtin-i18n";
import { t } from "@/lib/i18n";

interface UserDialogValues {
    username: string;
    email: string;
    realName: string;
    password: string;
    status: string;
    roleIds: number[];
}
export const UserDialog = ({
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
                        <DialogFooter
                            onCancel={() => {
                                form.resetFields();
                                setOpen(false);
                            }}
                            submitLabel={
                                submitting
                                    ? mode === "create"
                                        ? t("创建中", "Creating")
                                        : t("保存中", "Saving")
                                    : mode === "create"
                                      ? t("创建", "Create")
                                      : t("保存", "Save")
                            }
                            submitting={submitting}
                            submitDisabled={submitDisabled}
                            submitHtmlType="submit"
                        />
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
