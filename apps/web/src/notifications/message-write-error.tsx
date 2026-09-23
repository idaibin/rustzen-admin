import { Alert, Button } from "antd";

import { ApiRequestError } from "@/api/request";
import { t } from "@/lib/i18n";

export type WriteFailure = "forbidden" | "missing" | "retryable";

export const classifyWriteFailure = (error: unknown): WriteFailure => {
    if (error instanceof ApiRequestError && error.status === 403) return "forbidden";
    if (error instanceof ApiRequestError && error.status === 404) return "missing";
    return "retryable";
};

export const visibleMessages = <T extends { id: string }>(
    items: T[],
    forbidden: boolean,
    hiddenIds: ReadonlySet<string>,
): T[] => (forbidden ? [] : items.filter((item) => !hiddenIds.has(item.id)));

export const MessageWriteError = ({ error, onRetry }: { error: unknown; onRetry: () => void }) => {
    if (!error) return null;
    const failure = classifyWriteFailure(error);
    return (
        <Alert
            type={failure === "retryable" ? "error" : "warning"}
            showIcon
            message={
                failure === "forbidden"
                    ? t("消息中心权限已失效", "Message center permission is no longer available")
                    : failure === "missing"
                      ? t("该消息已不可访问", "This message is no longer accessible")
                      : t("更新已读状态失败", "Read status could not be updated")
            }
            action={
                failure === "retryable" ? (
                    <Button size="small" onClick={onRetry}>
                        {t("重试", "Retry")}
                    </Button>
                ) : undefined
            }
        />
    );
};
