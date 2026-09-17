import { DeleteOutlined, EditOutlined } from "@ant-design/icons";
import { Button, Popconfirm, Tooltip } from "antd";

import { appMessage, systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { t } from "@/lib/i18n";

import { deriveRoleDeletionState } from "./-role-delete-state";
import { RoleDialog } from "./-role-dialog";

const BUILTIN_ROLE_CODES = new Set(["owner", "admin", "viewer"]);

export function RoleActions({ record, onSuccess }: { record: Role.Item; onSuccess: () => void }) {
    if (isBuiltInRoleCode(record.code)) {
        return null;
    }

    const deletionState = deriveRoleDeletionState(record);
    const deletionBlockedReason = deletionState.blockedByAssignments
        ? t(
              "该角色已分配用户，移除所有分配后才能删除。",
              "This role is assigned to users. Remove all assignments before deleting it.",
          )
        : t("当前角色不可删除。", "This role cannot be deleted right now.");
    const deleteButton = (
        <Button
            type="text"
            size="small"
            danger
            disabled={deletionState.disabled}
            aria-label={
                deletionState.disabled ? deletionBlockedReason : t("删除角色", "Delete role")
            }
            title={deletionState.disabled ? deletionBlockedReason : undefined}
            icon={<DeleteOutlined />}
        />
    );

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
                {deletionState.disabled ? (
                    <Tooltip title={deletionBlockedReason}>
                        <span
                            className="inline-flex"
                            tabIndex={0}
                            aria-label={deletionBlockedReason}
                        >
                            {deleteButton}
                        </span>
                    </Tooltip>
                ) : (
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
                        {deleteButton}
                    </Popconfirm>
                )}
            </AuthWrap>
        </div>
    );
}

async function onSuccessDelete(id: number, onSuccess: () => void) {
    await systemAPI.role.delete(id);
    appMessage.success(t("角色已删除", "Role deleted."));
    onSuccess();
}

function isBuiltInRoleCode(code: string) {
    return BUILTIN_ROLE_CODES.has(code);
}
