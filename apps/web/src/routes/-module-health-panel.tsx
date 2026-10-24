import {
    AppstoreOutlined,
    BarChartOutlined,
    CloudServerOutlined,
    FileTextOutlined,
} from "@ant-design/icons";
import { Button, Card, Typography, theme } from "antd";
import type { ReactNode } from "react";

import { BackgroundRefreshNotice } from "@/components/feedback/background-refresh-notice";
import { DataState } from "@/components/feedback/data-state";
import { t, useLocale } from "@/lib/i18n";

type ModuleHealth = Dashboard.ModuleHealth;

export function ModuleHealthPanel({
    data,
    dataUpdatedAt,
    error,
    isPending,
    isFetching,
    onRetry,
}: {
    data: ModuleHealth[] | undefined;
    dataUpdatedAt: number;
    error: Error | null;
    isPending: boolean;
    isFetching: boolean;
    onRetry: () => void;
}) {
    useLocale();
    const { token } = theme.useToken();

    if (data !== undefined && data.length === 0) {
        return null;
    }

    return (
        <Card className="page-panel h-full" variant="outlined" styles={{ body: { padding: 24 } }}>
            <div className="mb-6">
                <Typography.Title level={5} className="!mb-1.5">
                    {t("扩展示例", "Extension examples")}
                </Typography.Title>
                <Typography.Text type="secondary">
                    {t(
                        "已接入的扩展示例及其当前服务状态。",
                        "Connected extension examples and their current service status.",
                    )}
                </Typography.Text>
            </div>
            {isPending && data === undefined ? (
                <DataState
                    kind="loading"
                    title={t("正在加载模块状态", "Loading module status")}
                    compact
                />
            ) : error && data === undefined ? (
                <DataState
                    kind="error"
                    title={t("模块状态加载失败", "Failed to load module status")}
                    description={t(
                        "无法读取当前数据，请检查 Admin 服务后重试。",
                        "Unable to read the current data. Check the Admin service and try again.",
                    )}
                    action={
                        <Button loading={isFetching} onClick={onRetry}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                    compact
                />
            ) : (
                <>
                    <div className="grid gap-3 md:grid-cols-3 xl:grid-cols-1">
                        {(data ?? []).map((health) => {
                            const moduleMeta: Record<string, { label: string; icon: ReactNode }> = {
                                monitor: {
                                    label: t("监控", "Monitor"),
                                    icon: <CloudServerOutlined />,
                                },
                                insights: {
                                    label: t("分析", "Insights"),
                                    icon: <BarChartOutlined />,
                                },
                                reports: {
                                    label: t("自动化", "Automation"),
                                    icon: <FileTextOutlined />,
                                },
                            };
                            const meta = moduleMeta[health.module] ?? {
                                label: health.module,
                                icon: <AppstoreOutlined />,
                            };
                            return (
                                <div
                                    key={health.module}
                                    aria-label={`${meta.label}：${health.available ? t("运行中", "Online") : t("不可用", "Unavailable")}`}
                                    className="flex min-h-20 min-w-0 items-center gap-3 rounded-lg border bg-card px-3 py-3"
                                    style={{ borderColor: token.colorBorderSecondary }}
                                >
                                    <span
                                        className="flex size-9 shrink-0 items-center justify-center rounded-lg text-base"
                                        style={{
                                            color: token.colorPrimary,
                                            background: token.colorFillTertiary,
                                        }}
                                    >
                                        {meta.icon}
                                    </span>
                                    <div className="min-w-0">
                                        <Typography.Text strong ellipsis>
                                            {meta.label}
                                        </Typography.Text>
                                        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                                            <span
                                                className={`size-1.5 rounded-full ${health.available ? "bg-status-success" : "bg-status-danger"}`}
                                            />
                                            {health.available
                                                ? t("运行中", "Online")
                                                : t("不可用", "Unavailable")}
                                        </div>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                    {error ? (
                        <BackgroundRefreshNotice
                            retrying={isFetching}
                            updatedAt={dataUpdatedAt}
                            onRetry={onRetry}
                        />
                    ) : null}
                </>
            )}
        </Card>
    );
}
