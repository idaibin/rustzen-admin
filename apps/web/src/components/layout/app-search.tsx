import { SearchOutlined } from "@ant-design/icons";
import { AutoComplete, Button, Input, Modal } from "antd";
import { useEffect, useMemo, useState } from "react";

import { t } from "@/lib/i18n";

import type { AppRoutePath, SearchRouteItem } from "./routes";

interface AppSearchProps {
    routes: SearchRouteItem[];
    onSelect: (path: AppRoutePath) => void;
}

type SearchOption = {
    value: string;
    label: string;
    groupLabel?: string;
    searchText: string;
    routePath: AppRoutePath;
};

export const AppSearch = ({ routes, onSelect }: AppSearchProps) => {
    const [open, setOpen] = useState(false);
    const [keyword, setKeyword] = useState("");

    const groupedOptions = useMemo(() => {
        const grouped = routes.reduce<Record<string, SearchOption[]>>((groups, route) => {
            const groupLabel = route.groupLabel;
            const searchText = [route.label, route.path, route.groupLabel].join(" ").toLowerCase();
            const option = {
                value: route.path,
                label: `${route.label} · ${route.path}`,
                groupLabel,
                searchText,
                routePath: route.path,
            };
            groups[groupLabel] = groups[groupLabel] ?? [];
            groups[groupLabel].push(option);
            return groups;
        }, {});

        return Object.entries(grouped).map(([groupLabel, options]) => ({
            value: groupLabel,
            label: groupLabel,
            options: options.map((option) => ({
                ...option,
                label: (
                    <>
                        <div className="flex min-w-0 items-center gap-2">
                            <span className="min-w-0 flex-1 truncate">{option.label}</span>
                            <span className="text-xs text-muted-foreground">
                                {option.routePath}
                            </span>
                        </div>
                    </>
                ),
                value: option.routePath,
            })),
        }));
    }, [routes]);

    useEffect(() => {
        const handleShortcut = (event: globalThis.KeyboardEvent) => {
            if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
                event.preventDefault();
                setOpen(true);
            }
        };

        window.addEventListener("keydown", handleShortcut);
        return () => window.removeEventListener("keydown", handleShortcut);
    }, []);

    const selectRoute = (value: AppRoutePath) => {
        const selected = routes.find((route) => route.path === value);
        if (!selected) {
            return;
        }
        onSelect(selected.path);
        setOpen(false);
        setKeyword("");
    };

    return (
        <>
            <Button
                type="default"
                className="size-9 shrink-0 justify-center px-0 text-muted-foreground sm:w-45 sm:justify-start sm:gap-2 sm:px-3 xl:w-90"
                onClick={() => setOpen(true)}
                icon={<SearchOutlined />}
                aria-label={t("打开页面搜索", "Open page search")}
            >
                <span className="hidden min-w-0 flex-1 truncate text-left sm:inline">
                    {t("搜索", "Search")}
                </span>
                <kbd className="hidden rounded border bg-muted px-1.5 py-0.5 text-xs leading-none text-muted-foreground sm:inline">
                    ⌘ K
                </kbd>
            </Button>

            <Modal
                open={open}
                onCancel={() => setOpen(false)}
                footer={null}
                centered
                title={t("搜索页面", "Search pages")}
                width={680}
            >
                <div className="space-y-3 py-2">
                    <AutoComplete
                        autoFocus
                        className="w-full"
                        options={groupedOptions as any}
                        onSelect={(_value: string, option: unknown) => {
                            const selected = option as SearchOption;
                            selectRoute(selected.routePath);
                        }}
                        value={keyword}
                        onChange={setKeyword}
                        placeholder={t("输入页面名称或路径...", "Type a page name or path...")}
                        onKeyDown={(event) => {
                            if (event.key === "Escape") {
                                setOpen(false);
                            }
                        }}
                        filterOption={(input, option) => {
                            const typed = (input ?? "").toLowerCase();
                            const routeOption = option as SearchOption;
                            const searchText = String(routeOption?.searchText ?? "").toLowerCase();
                            return searchText.includes(typed);
                        }}
                    >
                        <Input
                            suffix={null}
                            placeholder={t("输入页面名称或路径...", "Type a page name or path...")}
                        />
                    </AutoComplete>

                    {routes.length === 0 ? (
                        <div className="rounded-sm border border-dashed p-3 text-sm text-muted-foreground">
                            {t("未找到页面。", "No pages found.")}
                        </div>
                    ) : null}
                </div>
            </Modal>
        </>
    );
};
