import { useQuery } from "@tanstack/react-query";
import { Form, Input, Modal, Select, type FormProps } from "antd";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { menuQueryOptions } from "@/api/system/menu/query-options";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { getEnableOptions } from "@/constant/options";
import { localizeBuiltInMenuName } from "@/lib/builtin-i18n";
import { t } from "@/lib/i18n";

import { PermissionPicker } from "./-role-permission-picker";

interface RoleDialogProps {
    record?: Partial<Role.Item>;
    mode?: "create" | "edit";
    children: ReactNode;
    onSuccess?: () => void;
}

interface RoleFormValues {
    name: string;
    code: string;
    status: string;
    description: string;
}

export function RoleDialog({ children, record, mode = "create", onSuccess }: RoleDialogProps) {
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
                    <DialogFooter
                        onCancel={() => {
                            form.resetFields();
                            setMenuIds([]);
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
                </Form>
            </Modal>
        </>
    );
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
