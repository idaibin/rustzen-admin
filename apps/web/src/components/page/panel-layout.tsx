import { cn } from "cn";
import type { ReactNode } from "react";

export function PanelBody({ children, className }: { children: ReactNode; className?: string }) {
    return (
        <div className={cn("min-h-0 flex-1 space-y-5 overflow-y-auto", className)}>{children}</div>
    );
}

export function PanelFooter({ children, className }: { children: ReactNode; className?: string }) {
    return (
        <div
            className={cn(
                "flex shrink-0 flex-wrap items-center gap-4 border-t border-border pt-4",
                className,
            )}
        >
            {children}
        </div>
    );
}
