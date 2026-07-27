import { ReloadOutlined } from "@ant-design/icons";
import { Alert, Button } from "antd";

import { getLocale, t } from "@/lib/i18n";

interface BackgroundRefreshNoticeProps {
    updatedAt: number;
    onRetry: () => void;
}

export function BackgroundRefreshNotice({ updatedAt, onRetry }: BackgroundRefreshNoticeProps) {
    const lastUpdated = new Intl.DateTimeFormat(getLocale(), {
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    }).format(updatedAt);

    return (
        <Alert
            type="warning"
            showIcon
            role="alert"
            title={t(
                "后台刷新失败，当前继续显示上次成功数据。",
                "Background refresh failed. The last successfully loaded data remains visible.",
            )}
            description={t(
                `上次成功更新：${lastUpdated}`,
                `Last successful update: ${lastUpdated}`,
            )}
            action={
                <Button size="small" icon={<ReloadOutlined />} onClick={onRetry}>
                    {t("重试", "Retry")}
                </Button>
            }
        />
    );
}
