import { InboxOutlined } from "@ant-design/icons";
import { Flex, Progress, Result, Spin, Typography } from "antd";
import { cn } from "cn";
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
    const containerClassName = cn(
        "flex w-full items-center justify-center",
        compact ? "min-h-28 p-4" : "min-h-64 p-6",
        className,
    );

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
                <Flex vertical align="center" gap="small" className="text-center">
                    <span className="flex size-10 items-center justify-center rounded-lg bg-[var(--metric-blue-surface)] text-lg text-[var(--metric-blue-foreground)]">
                        <InboxOutlined aria-hidden />
                    </span>
                    <Typography.Text strong>{title}</Typography.Text>
                    {description ? (
                        <Typography.Text type="secondary">{description}</Typography.Text>
                    ) : null}
                    {action}
                </Flex>
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
