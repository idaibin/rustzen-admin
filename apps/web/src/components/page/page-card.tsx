import { ProCard } from "@ant-design/pro-components";
import { cn } from "cn";
import type { ReactNode } from "react";

import { PageHeader } from "./page-header";

interface PageCardProps {
    title: ReactNode;
    description?: ReactNode;
    actions?: ReactNode;
    toolbar?: ReactNode;
    children: ReactNode;
    className?: string;
    contentClassName?: string;
    headingLevel?: 1 | 2;
}

export function PageCard({
    title,
    description,
    actions,
    toolbar,
    children,
    className,
    contentClassName,
    headingLevel = 1,
}: PageCardProps) {
    const rootClassName = cn("page-card flex h-full min-h-0 flex-col gap-5", className);
    const contentClass = cn("flex min-h-0 flex-1 flex-col gap-4", contentClassName);

    return (
        <section className={rootClassName}>
            <PageHeader
                title={title}
                description={description}
                actions={
                    toolbar || actions ? (
                        <div className="flex max-w-full flex-wrap items-center justify-end gap-2">
                            {toolbar}
                            {actions}
                        </div>
                    ) : undefined
                }
                headingLevel={headingLevel}
            />
            <ProCard
                className="page-card-content min-h-0 flex-1"
                variant="borderless"
                styles={{
                    root: {
                        background: "transparent",
                        display: "flex",
                        flex: 1,
                        flexDirection: "column",
                        minHeight: 0,
                    },
                    body: {
                        display: "flex",
                        minHeight: 0,
                        flex: 1,
                        flexDirection: "column",
                        padding: 0,
                    },
                }}
            >
                <div className="flex min-h-0 flex-1 flex-col gap-4">
                    <div className={contentClass}>{children}</div>
                </div>
            </ProCard>
        </section>
    );
}
