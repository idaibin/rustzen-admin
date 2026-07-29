import { Empty, Flex, Progress, Result, Spin, Typography } from "antd";
import type { ReactNode } from "react";

export type DataStateKind = "loading" | "empty" | "error" | "permission" | "processing";

interface DataStateProps {
    kind: DataStateKind;
    title: string;
    description?: string;
    action?: ReactNode;
    progress?: number;
    compact?: boolean;
    className?: string;
}

export function DataState({
    kind,
    title,
    description,
    action,
    progress,
    compact = false,
    className,
}: DataStateProps) {
    const containerClassName = [
        "flex w-full items-center justify-center",
        compact ? "min-h-28 p-4" : "min-h-64 p-6",
        className,
    ]
        .filter((item): item is string => typeof item === "string")
        .join(" ");

    if (kind === "error" || kind === "permission") {
        return (
            <div className={containerClassName} role="alert">
                <Result
                    status={kind === "error" ? "error" : "403"}
                    title={title}
                    subTitle={description}
                    extra={action}
                />
            </div>
        );
    }

    if (kind === "empty") {
        return (
            <div className={containerClassName} role="status">
                <Empty
                    description={
                        <Flex vertical gap="small">
                            <Typography.Text strong>{title}</Typography.Text>
                            {description ? (
                                <Typography.Text type="secondary">{description}</Typography.Text>
                            ) : null}
                        </Flex>
                    }
                >
                    {action}
                </Empty>
            </div>
        );
    }

    if (kind === "processing") {
        return (
            <Flex
                className={containerClassName}
                vertical
                gap="middle"
                role="status"
                aria-live="polite"
                aria-busy="true"
            >
                <Typography.Text strong>{title}</Typography.Text>
                {description ? (
                    <Typography.Text type="secondary">{description}</Typography.Text>
                ) : null}
                {progress !== undefined ? (
                    <Progress percent={Math.round(progress)} className="max-w-sm" />
                ) : (
                    <Spin />
                )}
                {action}
            </Flex>
        );
    }

    return (
        <Flex
            className={containerClassName}
            vertical
            gap="middle"
            role="status"
            aria-live="polite"
            aria-busy="true"
        >
            <Spin size={compact ? "medium" : "large"} />
            <Typography.Text strong>{title}</Typography.Text>
            {description ? <Typography.Text type="secondary">{description}</Typography.Text> : null}
            {action}
        </Flex>
    );
}
