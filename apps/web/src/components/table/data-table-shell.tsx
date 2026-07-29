import type { ReactNode } from "react";

export function DataTableShell({ children }: { children: ReactNode }) {
    return <div className="min-h-0 flex-1 overflow-auto">{children}</div>;
}
