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
            title={title}
            subTitle={description}
            extra={actions ? [actions] : undefined}
        />
    );
}
