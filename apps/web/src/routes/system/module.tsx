import { PoweroffOutlined } from "@ant-design/icons";
import { ProTable, type ProColumns } from "@ant-design/pro-components";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Tag, Tooltip } from "antd";

import { appMessage, systemAPI } from "@/api";
import { AuthWrap } from "@/components/auth";
import { ConfirmDialog } from "@/components/feedback/confirm-dialog";
import { DataState } from "@/components/feedback/data-state";
import { PageCard } from "@/components/page/page-card";
import { localizeModuleName } from "@/lib/builtin-i18n";
import { t } from "@/lib/i18n";

export const Route = createFileRoute("/system/module")({ component: SystemModulePage });

function SystemModulePage() {
    const queryClient = useQueryClient();
    const { data, error, isPending, refetch } = useQuery({
        queryKey: ["system", "modules"],
        queryFn: systemAPI.module.list,
        refetchInterval: 10_000,
    });

    const updateEnabled = async (module: SystemModule.Item) => {
        const enabled = !module.enabled;
        const modules = await systemAPI.module.updateEnabled(module.id, enabled);
        queryClient.setQueryData(["system", "modules"], modules);
        await queryClient.invalidateQueries({
            queryKey: ["system", "modules", "navigation"],
        });
        const moduleName = localizeModuleName(module.id, module.name);
        appMessage.success(
            enabled
                ? t(`${moduleName} 已启用`, `${moduleName} enabled.`)
                : t(`${moduleName} 已禁用`, `${moduleName} disabled.`),
        );
    };

    const modules = data ?? [];

    const columns: ProColumns<SystemModule.Item>[] = [
        {
            title: t("模块", "Module"),
            key: "module",
            width: 240,
            render: (_: unknown, module: SystemModule.Item) => (
                <div>
                    <div className="font-medium">{localizeModuleName(module.id, module.name)}</div>
                    <div className="text-xs text-muted-foreground">{module.id}</div>
                </div>
            ),
        },
        {
            title: t("启用状态", "Enabled"),
            key: "enabled",
            width: 120,
            render: (_: unknown, module: SystemModule.Item) => (
                <Tag color={module.enabled ? "blue" : "default"}>
                    {module.enabled ? t("已启用", "Enabled") : t("已禁用", "Disabled")}
                </Tag>
            ),
        },
        {
            title: t("健康状态", "Health"),
            key: "health",
            width: 160,
            render: (_: unknown, module: SystemModule.Item) => <ModuleHealthTag module={module} />,
        },
        {
            title: t("操作", "Actions"),
            key: "actions",
            fixed: "right",
            width: 180,
            render: (_: unknown, module: SystemModule.Item) => (
                <AuthWrap code="system:module:update">
                    <ConfirmDialog
                        trigger={
                            <Button
                                icon={<PoweroffOutlined />}
                                type={module.enabled ? "default" : "primary"}
                                size="small"
                                danger={module.enabled}
                            >
                                {module.enabled ? t("禁用", "Disable") : t("启用", "Enable")}
                            </Button>
                        }
                        title={
                            module.enabled
                                ? t(
                                      `禁用${localizeModuleName(module.id, module.name)}`,
                                      `Disable ${localizeModuleName(module.id, module.name)}`,
                                  )
                                : t(
                                      `启用${localizeModuleName(module.id, module.name)}`,
                                      `Enable ${localizeModuleName(module.id, module.name)}`,
                                  )
                        }
                        description={
                            module.enabled
                                ? t(
                                      `禁用 ${localizeModuleName(module.id, module.name)} 并移除对应导航入口？`,
                                      `Disable ${localizeModuleName(module.id, module.name)} and remove its navigation entry?`,
                                  )
                                : t(
                                      `启用 ${localizeModuleName(module.id, module.name)} 并恢复 Manifest 同步？`,
                                      `Enable ${localizeModuleName(module.id, module.name)} and restore manifest synchronization?`,
                                  )
                        }
                        confirmLabel={module.enabled ? t("禁用", "Disable") : t("启用", "Enable")}
                        destructive={module.enabled}
                        onConfirm={() => updateEnabled(module)}
                    />
                </AuthWrap>
            ),
        },
    ];

    if (!data && isPending) {
        return (
            <PageCard
                title={t("系统模块", "System modules")}
                description={t(
                    "启用内置模块并查看当前运行状态。",
                    "Enable built-in modules and view their current runtime status.",
                )}
            >
                <DataState kind="loading" title={t("正在加载模块", "Loading modules")} />
            </PageCard>
        );
    }

    if (!data && error) {
        return (
            <PageCard
                title={t("系统模块", "System modules")}
                description={t(
                    "启用内置模块并查看当前运行状态。",
                    "Enable built-in modules and view their current runtime status.",
                )}
            >
                <DataState
                    kind="error"
                    title={t("模块加载失败", "Failed to load modules")}
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
            title={t("系统模块", "System modules")}
            description={t(
                "启用内置模块并查看当前运行状态。",
                "Enable built-in modules and view their current runtime status.",
            )}
        >
            <ProTable<SystemModule.Item>
                rowKey="id"
                columns={columns}
                dataSource={modules}
                loading={isPending}
                search={false}
                options={false}
                pagination={false}
                locale={{
                    emptyText:
                        modules.length === 0 ? (
                            <DataState kind="empty" title={t("暂无模块", "No modules")} />
                        ) : undefined,
                }}
            />
        </PageCard>
    );
}

function ModuleHealthTag({ module }: { module: SystemModule.Item }) {
    if (!module.enabled) {
        return <Tag color="default">{t("已禁用", "Disabled")}</Tag>;
    }
    if (module.available) {
        return <Tag color="green">{t("可用", "Available")}</Tag>;
    }
    if (module.compatible) {
        return (
            <Tooltip title={module.error || t("未提供错误原因", "No error details")}>
                <Tag color="orange" tabIndex={0}>
                    {t("不可用", "Unavailable")}
                </Tag>
            </Tooltip>
        );
    }
    if (module.releaseVersion) {
        return (
            <Tooltip title={module.error || t("未提供错误原因", "No error details")}>
                <Tag color="red" tabIndex={0}>
                    {t("不兼容", "Incompatible")}
                </Tag>
            </Tooltip>
        );
    }
    return (
        <Tooltip title={module.error || t("未提供错误原因", "No error details")}>
            <Tag color="red" tabIndex={0}>
                {t("未就绪", "Not ready")}
            </Tag>
        </Tooltip>
    );
}
