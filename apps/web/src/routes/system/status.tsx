import {
    CloudServerOutlined,
    DatabaseOutlined,
    HddOutlined,
    ReloadOutlined,
} from "@ant-design/icons";
import { ProCard } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Badge, Button, Card, Progress, Statistic, Tag, Typography } from "antd";

import { systemAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageHeader } from "@/components/page/page-header";
import { t, useLocale } from "@/lib/i18n";

export const Route = createFileRoute("/system/status")({
    component: SystemStatusPage,
});

function SystemStatusPage() {
    useLocale();
    const { data, isError, isLoading, refetch } = useQuery({
        queryKey: ["system", "status"],
        queryFn: systemAPI.status.overview,
        refetchInterval: 30 * 1000,
    });

    if (isLoading && !data) {
        return <DataState kind="loading" title={t("正在加载系统状态", "Loading system status")} />;
    }

    return (
        <div className="flex h-full min-h-0 flex-col gap-4 overflow-y-auto">
            <PageHeader
                title={t("系统状态", "System status")}
                description={t(
                    "存储和本地资源遥测数据每 30 秒刷新一次。",
                    "Storage and local resource telemetry refresh every 30 seconds.",
                )}
                actions={
                    <span className="text-sm text-muted-foreground">
                        {t("采集时间：", "Collected at: ")}
                        {data?.collectedAt ? formatDateTime(data.collectedAt) : "-"}
                    </span>
                }
            />

            {data ? (
                <StatusGrid
                    storage={data.storage}
                    modules={data.modules ?? []}
                    resource={data.resource}
                />
            ) : isError ? (
                <DataState
                    kind="error"
                    title={t("系统状态加载失败", "Failed to load system status")}
                    description={t(
                        "请检查 Admin 服务日志和本地资源读取权限后重试。",
                        "Check the Admin service logs and local resource permissions, then try again.",
                    )}
                    action={
                        <Button icon={<ReloadOutlined />} onClick={() => void refetch()}>
                            {t("重新加载", "Reload")}
                        </Button>
                    }
                />
            ) : null}
        </div>
    );
}

function StatusGrid({
    storage,
    modules,
    resource,
}: {
    storage: SystemStatus.StorageStatus;
    modules: SystemStatus.ModuleDatabaseStatus[];
    resource: SystemStatus.LocalResourceStatus;
}) {
    return (
        <div className="grid items-stretch gap-4 xl:grid-cols-[minmax(0,7fr)_minmax(300px,5fr)]">
            <StorageCard storage={storage} modules={modules} />
            <ResourceCard resource={resource} />
        </div>
    );
}

function StorageCard({
    storage,
    modules,
}: {
    storage: SystemStatus.StorageStatus;
    modules: SystemStatus.ModuleDatabaseStatus[];
}) {
    const directories = [...storage.directories].sort((a, b) => b.sizeBytes - a.sizeBytes);
    const moduleRows = [...(modules ?? [])].sort(
        (a, b) => (b.database?.totalBytes ?? -1) - (a.database?.totalBytes ?? -1),
    );
    const databaseFiles = [
        { label: t("主库", "Main database"), value: storage.database.mainBytes },
        { label: "WAL", value: storage.database.walBytes },
        { label: "SHM", value: storage.database.shmBytes },
    ].sort((a, b) => b.value - a.value);

    return (
        <ProCard
            title={t("存储", "Storage")}
            subTitle={t(
                "SQLite 存储及运行目录分布。",
                "SQLite storage and runtime directory distribution.",
            )}
            className="min-w-0"
        >
            <div className="flex h-full flex-col gap-6">
                <div className="grid items-center gap-6 md:grid-cols-[minmax(9rem,auto)_minmax(0,1fr)]">
                    <div>
                        <div className="mb-3 text-sm text-muted-foreground">
                            <DatabaseOutlined className="mr-2" />
                            {t("SQLite 总计", "SQLite total")}
                        </div>
                        <Statistic
                            value={formatBytes(storage.database.totalBytes)}
                            styles={{ content: { fontSize: 28 } }}
                        />
                    </div>
                    <div className="grid gap-5 sm:grid-cols-3">
                        {databaseFiles.map((file) => (
                            <StorageBreakdownItem
                                key={file.label}
                                label={file.label}
                                value={file.value}
                                total={storage.database.totalBytes}
                            />
                        ))}
                    </div>
                </div>

                <Card size="small" className="flex min-h-0 flex-1 flex-col">
                    <div className="mb-4 flex items-start justify-between gap-4">
                        <div>
                            <Typography.Text strong>
                                {t("模块数据库", "Module databases")}
                            </Typography.Text>
                            <Typography.Paragraph type="secondary">
                                {t(
                                    "监控 / 埋点 / 自动化 · 各自独立 SQLite，模块自报",
                                    "Monitor / Insights / Reports · separate SQLite, self-reported",
                                )}
                            </Typography.Paragraph>
                        </div>
                    </div>
                    <div className="flex flex-1 flex-col justify-around">
                        {moduleRows.map((row) => (
                            <div
                                key={row.module}
                                className="flex flex-wrap items-center gap-x-4 gap-y-1 py-2"
                            >
                                <Badge
                                    status={row.available ? "success" : "default"}
                                    text={
                                        <span className="font-semibold">
                                            {moduleName(row.module)}
                                        </span>
                                    }
                                />
                                <span className="ms-auto w-[220px] text-right font-mono text-xs text-muted-foreground">
                                    {row.module}.db
                                </span>
                                <span className="w-24 text-right font-semibold tabular-nums">
                                    {row.database
                                        ? formatBytes(row.database.totalBytes)
                                        : t("不可用", "Unavailable")}
                                </span>
                                <span className="w-32 text-right text-xs tabular-nums text-muted-foreground">
                                    {row.database
                                        ? `WAL ${formatBytes(row.database.walBytes)}`
                                        : `${t("上次", "Last")} ${formatDateTime(row.collectedAt)}`}
                                </span>
                            </div>
                        ))}
                    </div>
                </Card>

                <div className="flex flex-wrap items-baseline justify-between gap-2 text-xs text-muted-foreground">
                    <span>
                        <Typography.Text strong className="text-xs">
                            {t("目录占用", "Directory usage")}
                        </Typography.Text>
                        {"　"}
                        {directories
                            .map(
                                (item) =>
                                    `${item.label} ${formatBytes(item.sizeBytes)}${
                                        item.errorMessage ? `（${item.errorMessage}）` : ""
                                    }`,
                            )
                            .join(" · ")}
                    </span>
                    <span>{t("host 级汇总", "host-level summary")}</span>
                </div>
            </div>
        </ProCard>
    );
}

function StorageBreakdownItem({
    label,
    value,
    total,
}: {
    label: string;
    value: number;
    total: number;
}) {
    const percent = total > 0 ? Math.round((value / total) * 100) : 0;

    return (
        <div>
            <div className="mb-2 flex items-center justify-between gap-4">
                <span className="font-medium text-muted-foreground">{label}</span>
                <span className="font-semibold">{formatBytes(value)}</span>
            </div>
            <Progress percent={percent} showInfo={false} />
        </div>
    );
}

function ResourceCard({ resource }: { resource: SystemStatus.LocalResourceStatus }) {
    return (
        <ProCard
            title={t("本地资源", "Local resources")}
            subTitle={t("CPU、内存和磁盘使用情况。", "CPU, memory, and disk usage.")}
            className="min-w-0"
        >
            <div className="grid h-full grid-cols-1 content-between gap-6">
                <ResourceMetric
                    icon={<CloudServerOutlined />}
                    title="CPU"
                    detail={t(`${resource.cpu.cores} 核`, `${resource.cpu.cores} cores`)}
                    percent={resource.cpu.usagePercent}
                    status={getTagStatus(resource.cpu.usagePercent)}
                />
                <ResourceMetric
                    icon={<DatabaseOutlined />}
                    title={t("内存", "Memory")}
                    detail={`${formatBytes(resource.memory.usedBytes)} / ${formatBytes(resource.memory.totalBytes)}`}
                    percent={resource.memory.usagePercent}
                    status={getTagStatus(resource.memory.usagePercent)}
                />
                <ResourceMetric
                    icon={<HddOutlined />}
                    title={t("磁盘", "Disk")}
                    detail={`${formatBytes(resource.disk.usedBytes)} / ${formatBytes(resource.disk.totalBytes)}`}
                    percent={resource.disk.usagePercent}
                    status={getTagStatus(resource.disk.usagePercent)}
                />
            </div>
        </ProCard>
    );
}

function moduleName(module: string) {
    switch (module) {
        case "monitor":
            return t("监控", "Monitor");
        case "insights":
            return t("埋点", "Insights");
        case "reports":
            return t("自动化", "Reports");
        default:
            return module;
    }
}

function getTagStatus(percent: number) {
    if (percent >= 90) {
        return { color: "red", text: t("高", "High") };
    }
    if (percent >= 70) {
        return { color: "orange", text: t("中", "Medium") };
    }
    return { color: "green", text: t("正常", "Normal") };
}

function ResourceMetric({
    icon,
    title,
    detail,
    percent,
    status,
}: {
    icon: React.ReactNode;
    title: string;
    detail: string;
    percent: number;
    status: { color: string; text: string };
}) {
    return (
        <div>
            <div className="mb-3 flex items-start justify-between gap-4">
                <div className="mb-3 flex items-center gap-2 font-semibold">
                    <span className="text-muted-foreground">{icon}</span>
                    <span>{title}</span>
                </div>
                <div className="text-right text-muted-foreground">{detail}</div>
            </div>
            <Statistic
                value={formatPercent(percent)}
                suffix="%"
                precision={1}
                styles={{ content: { fontSize: 20 } }}
            />
            <div className="mt-3 flex w-full items-center gap-3">
                <Progress
                    className="min-w-0 flex-1"
                    percent={clampPercent(percent)}
                    showInfo={false}
                />
                <Tag color={status.color}>{status.text}</Tag>
            </div>
        </div>
    );
}

function clampPercent(value: number) {
    return Math.max(0, Math.min(100, Number(value.toFixed(1))));
}

function formatPercent(value: number) {
    return `${Number(value.toFixed(1))}`;
}

function formatBytes(bytes: number) {
    if (!bytes) {
        return "0 B";
    }

    const units = ["B", "KB", "MB", "GB", "TB"] as const;
    let value = bytes;
    let unitIndex = 0;
    while (value >= 1024 && unitIndex < units.length - 1) {
        value /= 1024;
        unitIndex += 1;
    }

    const precision = unitIndex === 0 ? 0 : 1;
    return `${Number(value.toFixed(precision))} ${units[unitIndex]}`;
}

function formatDateTime(value: string | null | undefined) {
    if (!value) {
        return "-";
    }
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return "-";
    }
    const pad = (part: number) => part.toString().padStart(2, "0");
    return (
        [date.getFullYear(), pad(date.getMonth() + 1), pad(date.getDate())].join("-") +
        ` ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
    );
}
