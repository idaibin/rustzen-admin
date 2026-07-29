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
            subTitle={description}
            extra={actions}
            headerBordered
        >
            <div className="flex min-h-0 flex-1 flex-col gap-4 pt-4">
                {toolbar}
                <div className={contentClass.join(" ")}>{children}</div>
            </div>
        </ProCard>
    );
}
