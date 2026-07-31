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
            divider: "var(--metric-blue-divider)",
        },
        green: {
            foreground: "var(--metric-green-foreground)",
            surface: "var(--metric-green-surface)",
            divider: "var(--metric-green-divider)",
        },
        violet: {
            foreground: "var(--metric-violet-foreground)",
            surface: "var(--metric-violet-surface)",
            divider: "var(--metric-violet-divider)",
        },
        amber: {
            foreground: "var(--metric-amber-foreground)",
            surface: "var(--metric-amber-surface)",
            divider: "var(--metric-amber-divider)",
        },
        red: {
            foreground: "var(--metric-red-foreground)",
            surface: "var(--metric-red-surface)",
            divider: "var(--metric-red-divider)",
        },
    }[tone];

    return (
        <Card
            className="min-h-44 overflow-hidden border border-border bg-card shadow-sm"
            styles={{ body: { height: "100%", padding: 0 }, root: { borderRadius: 12 } }}
        >
            <div className="flex h-full flex-col">
                <div className="flex min-h-[87px] items-center gap-3 px-6 py-4">
                    <span
                        className="flex size-12 shrink-0 items-center justify-center rounded-lg text-[26px]"
                        style={{
                            color: toneTokens.foreground,
                            background: toneTokens.surface,
                        }}
                    >
                        {icon}
                    </span>
                    <div className="min-w-0">
                        <div className="text-lg font-semibold leading-tight text-foreground">
                            {label}
                        </div>
                        {hint ? (
                            <div className="mt-1 text-xs leading-5 text-muted-foreground">
                                {hint}
                            </div>
                        ) : null}
                    </div>
                </div>
                <div
                    className="flex min-h-[87px] min-w-0 flex-1 items-center border-t px-6 py-3"
                    style={{
                        borderColor: toneTokens.divider,
                        background: toneTokens.surface,
                    }}
                >
                    <Statistic
                        value={value}
                        styles={{
                            content: {
                                color: toneTokens.foreground,
                                fontSize: 56,
                                fontWeight: 700,
                                lineHeight: 1,
                                letterSpacing: "-0.04em",
                                fontVariantNumeric: "tabular-nums",
                            },
                        }}
                    />
                </div>
            </div>
        </Card>
    );
}
