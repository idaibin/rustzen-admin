import { ProCard } from "@ant-design/pro-components";
import type { ReactNode } from "react";

interface PageCardProps {
    title: ReactNode;
    description?: ReactNode;
    actions?: ReactNode;
    toolbar?: ReactNode;
    children: ReactNode;
    className?: string;
    contentClassName?: string;
}

export function PageCard({
    title,
    description,
    actions,
    toolbar,
    children,
    className,
    contentClassName,
}: PageCardProps) {
    const rootClassName = ["flex h-full min-h-0 flex-col overflow-hidden", className].filter(
        (item): item is string => typeof item === "string",
    );
    const contentClass = ["flex min-h-0 flex-1 flex-col gap-4", contentClassName].filter(
        (item): item is string => typeof item === "string",
    );

    return (
        <ProCard
            className={rootClassName.join(" ")}
            title={title}
            subTitle={
                description ? (
                    <div className="mt-1 text-sm text-muted-foreground">{description}</div>
                ) : null
            }
            extra={actions ? <div className="flex shrink-0 flex-wrap gap-2">{actions}</div> : null}
            headerBordered
        >
            <div className="flex min-h-0 flex-1 flex-col gap-4 pt-4">
                {toolbar ? (
                    <div className="rounded-md border bg-muted/35 p-2">{toolbar}</div>
                ) : null}
                <div className={contentClass.join(" ")}>{children}</div>
            </div>
        </ProCard>
    );
}
