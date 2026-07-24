import { Card } from "antd";
import type { ReactNode } from "react";

export function DataTableShell({ children }: { children: ReactNode }) {
    return (
        <Card
            className="min-h-0 flex-1 overflow-auto rounded-md"
            styles={{ body: { minHeight: 0, padding: 0 } }}
        >
            {children}
        </Card>
    );
}
