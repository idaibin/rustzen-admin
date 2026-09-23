import { Alert, Space, Typography } from "antd";

import type { ModuleLogCleanupResult } from "@/api/system/status/module-logs";
import { t } from "@/lib/i18n";

export function CleanupResult({ result }: { result: ModuleLogCleanupResult }) {
    const resultType = result.partial ? "warning" : "success";
    return (
        <Alert
            data-testid="module-log-cleanup-result"
            type={resultType}
            showIcon
            message={
                result.partial
                    ? t("清理完成，但需要复核部分结果", "Cleanup completed with items to review")
                    : t("清理完成", "Cleanup completed")
            }
            description={
                <Space direction="vertical" size="small">
                    <Typography.Text>
                        {t(
                            `已删除 ${result.removed.length} 个文件，保留 ${result.retained.length} 个文件，失败 ${result.failures.length} 个文件。`,
                            `Removed ${result.removed.length}, retained ${result.retained.length}, failed ${result.failures.length}.`,
                        )}
                    </Typography.Text>
                    {result.failures.length ? <FailureList failures={result.failures} /> : null}
                </Space>
            }
        />
    );
}

export function FailureList({
    failures,
}: {
    failures: { module: string; fileName: string; reason: string }[];
}) {
    return (
        <ul className="m-0 list-disc space-y-1 pl-5">
            {failures.map((failure) => (
                <li key={`${failure.module}:${failure.fileName}:${failure.reason}`}>
                    <Typography.Text>
                        {failure.module} / {failure.fileName}: {failure.reason}
                    </Typography.Text>
                </li>
            ))}
        </ul>
    );
}
