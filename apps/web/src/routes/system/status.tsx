import {
    CloudServerOutlined,
    DatabaseOutlined,
    HddOutlined,
    ReloadOutlined,
} from "@ant-design/icons";
import { ProCard } from "@ant-design/pro-components";
import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Button, Card, Progress, Statistic, Tag, Typography } from "antd";

import { systemAPI } from "@/api";
import { DataState } from "@/components/feedback/data-state";
import { PageHeader } from "@/components/page/page-header";
import { t } from "@/lib/i18n";

import { ModuleLogDiagnostics } from "./-module-log-diagnostics";

export const Route = createFileRoute("/system/status")({
    component: SystemStatusPage,
});

function SystemStatusPage() {
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
                <>
                    <StorageCard storage={data.storage} />
                    <ResourceCard resource={data.resource} />
                    <ModuleLogDiagnostics />
                </>
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

function StorageCard({ storage }: { storage: SystemStatus.StorageStatus }) {
    const maxDirectoryBytes = Math.max(...storage.directories.map((item) => item.sizeBytes), 1);

    return (
        <div className="grid items-start gap-4 xl:grid-cols-[minmax(0,7fr)_minmax(280px,5fr)]">
            <ProCard
                title={t("存储", "Storage")}
                subTitle={t(
                    "SQLite 存储及运行目录分布。",
                    "SQLite storage and runtime directory distribution.",
                )}
            >
                <div className="space-y-6">
                    <div>
                        <div className="mb-3 text-sm text-muted-foreground">
                            <DatabaseOutlined className="mr-2" />
                            {t("SQLite 总计", "SQLite total")}
                        </div>
                        <Statistic
                            value={formatBytes(storage.database.totalBytes)}
                            styles={{ content: { fontSize: 28 } }}
                        />
                        <Tag className="mt-4">{t("SQLite 数据库", "SQLite database")}</Tag>
                        <Progress className="mt-5" percent={100} showInfo={false} />
                        <div className="mt-4 grid grid-cols-2 gap-3 text-sm text-muted-foreground">
                            <span>
                                {t("主库", "Main database")}{" "}
                                {formatBytes(storage.database.mainBytes)}
                            </span>
                            <span className="text-right">
                                WAL {formatBytes(storage.database.walBytes)}
                            </span>
                        </div>
                    </div>

                    <Card size="small">
                        <div className="mb-5 flex items-start justify-between gap-4">
                            <div>
                                <Typography.Text strong>
                                    {t("目录分布", "Directory distribution")}
                                </Typography.Text>
                                <Typography.Paragraph type="secondary">
                                    {t("按当前目录占用空间对比", "Compare current directory usage")}
                                </Typography.Paragraph>
                            </div>
                            <div className="text-sm text-muted-foreground">
                                {t(
                                    `${storage.directories.length} 项`,
                                    `${storage.directories.length} items`,
                                )}
                            </div>
                        </div>
                        <div className="grid grid-cols-1 gap-x-10 gap-y-8 md:grid-cols-2">
                            {storage.directories.map((item) => (
                                <div key={item.key}>
                                    <div className="mb-3 flex items-center justify-between gap-4">
                                        <div className="flex min-w-0 items-center gap-2">
                                            <span className="truncate font-semibold">
                                                {item.label}
                                            </span>
                                            {item.errorMessage ? (
                                                <Tag color="red">{item.errorMessage}</Tag>
                                            ) : null}
                                        </div>
                                        <div className="shrink-0 font-semibold">
                                            {formatBytes(item.sizeBytes)}
                                        </div>
                                    </div>
                                    <Progress
                                        percent={Math.round(
                                            (item.sizeBytes / maxDirectoryBytes) * 100,
                                        )}
                                        showInfo={false}
                                    />
                                </div>
                            ))}
                        </div>
                    </Card>
                </div>
            </ProCard>

            <ProCard
                title={t("数据库文件", "Database files")}
                className="min-w-0"
                extra={
                    <span className="text-xs text-muted-foreground">
                        {t("主库 / WAL / SHM", "Main / WAL / SHM")}
                    </span>
                }
            >
                <div className="grid grid-cols-1 gap-4 md:grid-cols-1">
                    <StorageBreakdownItem
                        label={t("主库", "Main database")}
                        value={storage.database.mainBytes}
                        total={storage.database.totalBytes}
                    />
                    <StorageBreakdownItem
                        label="WAL"
                        value={storage.database.walBytes}
                        total={storage.database.totalBytes}
                    />
                    <StorageBreakdownItem
                        label="SHM"
                        value={storage.database.shmBytes}
                        total={storage.database.totalBytes}
                    />
                </div>
            </ProCard>
        </div>
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
        >
            <div className="grid grid-cols-1 gap-7 lg:grid-cols-3">
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

function formatDateTime(value: string) {
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
