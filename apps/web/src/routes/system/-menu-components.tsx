import { EditOutlined } from "@ant-design/icons";
import { useQueryClient } from "@tanstack/react-query";
import { Button, Form, Input, Modal, Select } from "antd";
import { useEffect, useState, type ReactNode } from "react";

import { appMessage, systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { DialogFooter } from "@/components/feedback/dialog-footer";
import { getEnableOptions, getModuleIconOptions } from "@/constant/options";
import { t } from "@/lib/i18n";

export type DisplayMenuItem = Menu.Item & {
    readOnly?: boolean;
};

export function MenuActions({
    record,
    onSuccess,
}: {
    record: DisplayMenuItem;
    onSuccess: () => void;
}) {
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
                                className="w-full"
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
                                className="w-full"
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
                    <DialogFooter
                        onCancel={() => setOpen(false)}
                        submitLabel={t("保存", "Save")}
                        submitting={submitting}
                        submitHtmlType="submit"
                    />
                </Form>
            </Modal>
        </>
    );
}
