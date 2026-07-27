import {
    AlertOutlined,
    FileDoneOutlined,
    LockOutlined,
    InboxOutlined,
    LoadingOutlined,
} from "@ant-design/icons";
import { Progress } from "antd";
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

const icons = {
    loading: LoadingOutlined,
    empty: InboxOutlined,
    error: AlertOutlined,
    permission: LockOutlined,
    processing: FileDoneOutlined,
};

export function DataState({
    kind,
    title,
    description,
    action,
    progress,
    compact = false,
    className,
}: DataStateProps) {
    const Icon = icons[kind];
    const busy = kind === "loading" || kind === "processing";

    const containerClassName = [
        "flex w-full flex-col items-center justify-center text-center",
        compact
            ? "min-h-28 gap-2 px-4 py-6"
            : "min-h-64 gap-3 rounded-lg border border-dashed px-6 py-10",
        kind === "error" && "border-destructive/40 bg-destructive/5",
        kind === "permission" && "border-status-warning/40 bg-status-warning/5",
        className,
    ].filter((item): item is string => typeof item === "string");

    const iconClassName = [
        "size-8 text-muted-foreground",
        busy && "animate-spin",
        kind === "error" && "text-destructive",
        kind === "permission" && "text-status-warning",
    ].filter((item): item is string => typeof item === "string");

    return (
        <div
            className={containerClassName.join(" ")}
            role={kind === "error" || kind === "permission" ? "alert" : "status"}
            aria-live={busy ? "polite" : undefined}
            aria-busy={busy || undefined}
        >
            <Icon className={iconClassName.join(" ")} aria-hidden="true" />
            <div className="space-y-1">
                <p className="font-medium">{title}</p>
                {description ? (
                    <p className="max-w-lg text-sm text-muted-foreground">{description}</p>
                ) : null}
            </div>
            {kind === "processing" && progress !== undefined ? (
                <div className="w-full max-w-sm space-y-1">
                    <Progress percent={progress} />
                    <p className="text-xs tabular-nums text-muted-foreground">
                        {Math.round(progress)}%
                    </p>
                </div>
            ) : null}
            {action ? (
                <div className="mt-1 flex flex-wrap justify-center gap-2">{action}</div>
            ) : null}
        </div>
    );
}
