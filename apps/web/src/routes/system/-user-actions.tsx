import { EditOutlined, MoreOutlined } from "@ant-design/icons";
import { Button, Dropdown, type MenuProps } from "antd";
import { useMemo, useState } from "react";

import { appMessage, systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { ConfirmModal } from "@/components/feedback/confirm-dialog";
import { t, useLocale } from "@/lib/i18n";
import { useAuthStore } from "@/store/useAuthStore";

import { UserDialog } from "./-user-dialog";

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
export function UserActions({
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
        if (!actionConfig) return;

        await actionConfig.onConfirm();
        onSuccess();
        setPendingAction(null);
    };

    const hideActionDialog = () => {
        setPendingAction(null);
    };

    if (record.id === currentUserId || record.isSystem) {
        return null;
    }

    return (
        <div className="flex items-center gap-2">
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
            <ConfirmModal
                open={pendingAction !== null}
                onCancel={hideActionDialog}
                title={actionConfig?.title}
                description={<p>{actionConfig?.description}</p>}
                confirmLabel={actionConfig?.actionLabel ?? t("确定", "Confirm")}
                destructive={actionConfig?.destructive}
                onConfirm={executeAction}
            />
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

export function getUserActionItems(
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
