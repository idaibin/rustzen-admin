import { Typography } from "antd";
import type { ReactNode } from "react";

import { PageTitle } from "./page-title";

interface PageHeaderProps {
    title: ReactNode;
    description?: ReactNode;
    actions?: ReactNode;
    headingLevel?: 1 | 2;
}

export function PageHeader({ title, description, actions, headingLevel = 1 }: PageHeaderProps) {
    return (
        <header className="page-header flex shrink-0 flex-wrap items-center justify-between gap-4">
            <div className="flex min-w-[min(100%,20rem)] flex-1 flex-wrap items-baseline gap-x-3 gap-y-1 md:flex-nowrap">
                <PageTitle className="page-title shrink-0" level={headingLevel}>
                    {title}
                </PageTitle>
                {description ? (
                    <Typography.Text
                        type="secondary"
                        className="min-w-0 md:truncate"
                        title={typeof description === "string" ? description : undefined}
                    >
                        {description}
                    </Typography.Text>
                ) : null}
            </div>
            {actions ? (
                <div className="ml-auto flex min-w-0 max-w-full flex-wrap justify-end gap-2">
                    {actions}
                </div>
            ) : null}
        </header>
    );
}
