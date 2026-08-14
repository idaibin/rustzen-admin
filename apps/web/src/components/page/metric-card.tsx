import { Card, Statistic } from "antd";
import type { ReactNode } from "react";

export type MetricCardTone = "blue" | "green" | "violet" | "amber" | "red";

interface MetricCardProps {
    label: ReactNode;
    value: number | string;
    icon: ReactNode;
    tone: MetricCardTone;
    hint?: ReactNode;
}

export function MetricCard({ label, value, icon, tone, hint }: MetricCardProps) {
    const toneTokens = {
        blue: {
            foreground: "var(--metric-blue-foreground)",
            surface: "var(--metric-blue-surface)",
        },
        green: {
            foreground: "var(--metric-green-foreground)",
            surface: "var(--metric-green-surface)",
        },
        violet: {
            foreground: "var(--metric-violet-foreground)",
            surface: "var(--metric-violet-surface)",
        },
        amber: {
            foreground: "var(--metric-amber-foreground)",
            surface: "var(--metric-amber-surface)",
        },
        red: {
            foreground: "var(--metric-red-foreground)",
            surface: "var(--metric-red-surface)",
        },
    }[tone];

    return (
        <Card
            className="min-h-32 overflow-hidden border border-border bg-card shadow-none"
            styles={{ body: { height: "100%", padding: 20 }, root: { borderRadius: 10 } }}
        >
            <div className="flex h-full items-start gap-4">
                <div className="min-w-0 flex-1">
                    <div className="mb-3 flex items-center gap-2 text-sm font-medium text-muted-foreground">
                        <span
                            className="flex size-8 shrink-0 items-center justify-center rounded-lg text-base"
                            style={{
                                color: toneTokens.foreground,
                                background: toneTokens.surface,
                            }}
                        >
                            {icon}
                        </span>
                        <span className="truncate">{label}</span>
                    </div>
                    <Statistic
                        value={value}
                        styles={{
                            content: {
                                color: "var(--foreground)",
                                fontSize: 32,
                                fontWeight: 650,
                                lineHeight: 1.15,
                                letterSpacing: "-0.03em",
                                fontVariantNumeric: "tabular-nums",
                            },
                        }}
                    />
                    {hint ? (
                        <div className="mt-2 truncate text-xs leading-5 text-muted-foreground">
                            {hint}
                        </div>
                    ) : null}
                </div>
            </div>
        </Card>
    );
}
