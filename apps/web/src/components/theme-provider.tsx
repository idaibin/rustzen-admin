import { MoonOutlined, SunOutlined } from "@ant-design/icons";
import { App as AntdApp, Button, ConfigProvider, theme as antdTheme, type ThemeConfig } from "antd";
import enUS from "antd/locale/en_US";
import zhCN from "antd/locale/zh_CN";
import { createContext, useContext, useLayoutEffect, useState, type ReactNode } from "react";

import { t, useLocale } from "@/lib/i18n";

export type Theme = "light" | "dark";

const THEME_STORAGE_KEY = "rustzen-admin-theme";

function readStoredTheme(): Theme {
    if (typeof window === "undefined") {
        return "light";
    }

    try {
        return localStorage.getItem(THEME_STORAGE_KEY) === "dark" ? "dark" : "light";
    } catch {
        return "light";
    }
}

// CSS owns the palette. Ant Design algorithms need concrete colors, not var() seeds.
function readThemeTokens(): ThemeConfig["token"] {
    if (typeof document === "undefined") return {};
    const style = getComputedStyle(document.documentElement);
    const color = (name: string) => style.getPropertyValue(`--${name}`).trim();
    return {
        colorPrimary: color("primary"),
        colorPrimaryHover: color("primary-hover"),
        colorPrimaryActive: color("primary-active"),
        colorInfo: color("status-info"),
        colorInfoText: color("status-info"),
        colorSuccess: color("status-success"),
        colorSuccessText: color("status-success"),
        colorWarning: color("status-warning"),
        colorWarningText: color("status-warning"),
        colorError: color("status-danger"),
        colorErrorText: color("status-danger"),
        colorLink: color("link"),
        colorBgBase: color("card"),
        colorBgLayout: color("background"),
        colorBgContainer: color("card"),
        colorBgElevated: color("popover"),
        colorFillTertiary: color("muted"),
        colorTextBase: color("foreground"),
        colorText: color("foreground"),
        colorTextDescription: color("muted-foreground"),
        colorTextSecondary: color("muted-foreground"),
        colorTextTertiary: color("muted-foreground"),
        colorTextPlaceholder: color("placeholder"),
        colorTextLightSolid: color("primary-foreground"),
        colorBorder: color("input"),
        colorBorderSecondary: color("border"),
    };
}

const ThemeContext = createContext<{
    theme: Theme;
    setTheme: (theme: Theme) => void;
} | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
    const [theme, setTheme] = useState<Theme>(readStoredTheme);
    const locale = useLocale();
    const [palette, setPalette] = useState(readThemeTokens);

    const antdLocale = locale === "en-US" ? enUS : zhCN;
    const algorithm = theme === "dark" ? antdTheme.darkAlgorithm : antdTheme.defaultAlgorithm;

    useLayoutEffect(() => {
        document.documentElement.classList.remove("white");
        document.documentElement.classList.toggle("dark", theme === "dark");
        setPalette(readThemeTokens());
        try {
            localStorage.setItem(THEME_STORAGE_KEY, theme);
        } catch {
            // The selected theme still applies when storage is unavailable.
        }
    }, [theme]);

    return (
        <ThemeContext value={{ theme, setTheme }}>
            <ConfigProvider
                locale={antdLocale}
                variant="filled"
                theme={{
                    algorithm,
                    token: {
                        borderRadius: 8,
                        ...palette,
                        controlHeight: 36,
                        fontSize: 14,
                        lineWidth: 1,
                        motionDurationFast: "0.12s",
                        motionDurationMid: "0.18s",
                    },
                    components: {
                        Button: {
                            borderRadius: 8,
                            controlHeight: 36,
                            fontWeight: 500,
                            primaryColor: "var(--primary-foreground)",
                            colorPrimary: "var(--primary)",
                            colorPrimaryHover: "var(--primary-hover)",
                            colorPrimaryActive: "var(--primary-active)",
                        },
                        Card: {
                            colorBgContainer: "var(--card)",
                            borderRadiusLG: 10,
                            boxShadowTertiary: "none",
                            headerFontSize: 16,
                        },
                        Input: {
                            activeShadow:
                                "0 0 0 3px color-mix(in srgb, var(--ring) 16%, transparent)",
                            borderRadius: 8,
                        },
                        Menu: {
                            itemBg: "transparent",
                            subMenuItemBg: "transparent",
                            itemColor: "var(--muted-foreground)",
                            itemSelectedBg: "var(--sidebar-accent)",
                            itemSelectedColor: "var(--primary)",
                            itemHoverBg: "var(--muted)",
                            itemHoverColor: "var(--foreground)",
                            iconSize: 14,
                            collapsedIconSize: 14,
                            itemBorderRadius: 8,
                            itemHeight: 40,
                        },
                        Select: {
                            borderRadius: 8,
                            optionSelectedBg: "var(--accent)",
                        },
                        Tag: {
                            defaultBg: "var(--muted)",
                            defaultColor: "var(--foreground)",
                            colorSuccess: "var(--status-success)",
                            colorSuccessBg: "var(--status-success-surface)",
                            colorWarning: "var(--status-warning)",
                            colorWarningBg: "var(--status-warning-surface)",
                            colorError: "var(--status-danger)",
                            colorErrorBg: "var(--status-danger-surface)",
                            colorInfo: "var(--status-info)",
                            colorInfoBg: "var(--status-info-surface)",
                        },
                        Table: {
                            borderColor: "transparent",
                            headerSplitColor: "transparent",
                            cellPaddingBlock: 13,
                            headerBg: "var(--table-header)",
                            headerColor: "var(--foreground)",
                            rowHoverBg: "var(--table-row-hover)",
                        },
                    },
                }}
            >
                <AntdApp className="flex h-full min-h-0 flex-col">{children}</AntdApp>
            </ConfigProvider>
        </ThemeContext>
    );
}

export function ThemeSwitch() {
    const context = useTheme();
    const nextTheme: Theme = context.theme === "light" ? "dark" : "light";
    const labels: Record<Theme, string> = {
        light: t("亮色", "Light"),
        dark: t("暗色", "Dark"),
    };
    const icon = context.theme === "light" ? <MoonOutlined /> : <SunOutlined />;

    return (
        <Button
            shape="circle"
            type="text"
            aria-label={t(
                `当前主题：${labels[context.theme]}。切换到${labels[nextTheme]}主题`,
                `Current theme: ${labels[context.theme]}. Switch to ${labels[nextTheme]} theme`,
            )}
            onClick={() => context.setTheme(nextTheme)}
            icon={icon}
        ></Button>
    );
}

export function useTheme() {
    const context = useContext(ThemeContext);
    if (!context) {
        throw new Error("useTheme must be used inside ThemeProvider");
    }
    return context;
}
