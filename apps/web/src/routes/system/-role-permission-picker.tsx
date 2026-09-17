import { Button, Card, Checkbox, Input, Tree } from "antd";
import { useMemo } from "react";

import { DataState } from "@/components/feedback/data-state";
import { t } from "@/lib/i18n";

export function PermissionPicker({
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
            <Card size="small" className="h-72 overflow-auto">
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
            </Card>
        </div>
    );
}
