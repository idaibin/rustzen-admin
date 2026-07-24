import { PageHeader as ProPageHeader } from "@ant-design/pro-components";
import type { ReactNode } from "react";

interface PageHeaderProps {
    title: ReactNode;
    description?: ReactNode;
    actions?: ReactNode;
}

export function PageHeader({ title, description, actions }: PageHeaderProps) {
    return (
        <ProPageHeader
            className="px-1"
            title={<span className="text-xl font-semibold tracking-tight">{title}</span>}
            subTitle={
                description ? (
                    <span className="text-sm text-muted-foreground">{description}</span>
                ) : null
            }
            extra={actions ? <div className="flex shrink-0 flex-wrap gap-2">{actions}</div> : null}
        />
    );
}
