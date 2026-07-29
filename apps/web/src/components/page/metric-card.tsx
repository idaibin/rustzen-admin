import { StatisticCard } from "@ant-design/pro-components";
import { theme } from "antd";
import type { ReactNode } from "react";

interface MetricCardProps {
    label: ReactNode;
    value: ReactNode;
    icon?: ReactNode;
    tone?: "primary" | "success" | "warning" | "danger" | "info";
    hint?: ReactNode;
    size?: "default" | "small";
}

export function MetricCard({ label, value, icon, tone, hint, size = "default" }: MetricCardProps) {
    const { token } = theme.useToken();
    const statisticValue = typeof value === "number" || typeof value === "string" ? value : "";
    const toneColor = tone
        ? {
              primary: token.colorPrimary,
              success: token.colorSuccess,
              warning: token.colorWarning,
              danger: token.colorError,
              info: token.colorInfo,
          }[tone]
        : undefined;

    return (
        <StatisticCard
            className="min-h-24 gap-0"
            size={size}
            statistic={{
                title: (
                    <span className="truncate text-sm font-medium text-muted-foreground">
                        {label}
                    </span>
                ),
                value: statisticValue,
                styles: toneColor ? { content: { color: toneColor } } : undefined,
                icon: icon ? (
                    <span style={toneColor ? { color: toneColor } : undefined}>{icon}</span>
                ) : null,
                description: hint ? (
                    <span className="text-xs text-muted-foreground">{hint}</span>
                ) : null,
            }}
        />
    );
}
