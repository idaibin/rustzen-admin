import { StatisticCard } from "@ant-design/pro-components";
import type { ReactNode } from "react";

interface MetricCardProps {
    label: ReactNode;
    value: ReactNode;
    icon?: ReactNode;
    hint?: ReactNode;
}

export function MetricCard({ label, value, icon, hint }: MetricCardProps) {
    const statisticValue = typeof value === "number" || typeof value === "string" ? value : "";

    return (
        <StatisticCard
            className="min-h-24 gap-0"
            statistic={{
                title: (
                    <span className="truncate text-sm font-medium text-muted-foreground">
                        {label}
                    </span>
                ),
                value: statisticValue,
                icon,
                description: hint ? (
                    <span className="text-xs text-muted-foreground">{hint}</span>
                ) : null,
            }}
        />
    );
}
