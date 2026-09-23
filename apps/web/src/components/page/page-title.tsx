import type { ReactNode } from "react";

interface PageTitleProps {
    children: ReactNode;
    className: string;
    level?: 1 | 2;
}

export function PageTitle({ children, className, level = 1 }: PageTitleProps) {
    const Title = level === 1 ? "h1" : "h2";
    return <Title className={className}>{children}</Title>;
}
