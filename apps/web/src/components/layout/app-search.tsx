import { SearchOutlined } from "@ant-design/icons";
import { Button, Empty, Flex, Input, Menu, Modal } from "antd";
import type { MenuProps } from "antd";
import { useEffect, useId, useMemo, useState, type KeyboardEvent } from "react";

import { t } from "@/lib/i18n";

import type { AppRoutePath, SearchRouteItem } from "./routes";

interface AppSearchProps {
    routes: SearchRouteItem[];
    onSelect: (path: AppRoutePath) => void;
}

export const AppSearch = ({ routes, onSelect }: AppSearchProps) => {
    const [open, setOpen] = useState(false);
    const [keyword, setKeyword] = useState("");
    const [activeIndex, setActiveIndex] = useState(0);
    const resultsId = useId();

    const filteredRoutes = useMemo(() => {
        const normalizedKeyword = keyword.trim().toLowerCase();
        if (!normalizedKeyword) {
            return routes;
        }
        return routes.filter((route) => route.searchText.includes(normalizedKeyword));
    }, [keyword, routes]);

    const menuItems = useMemo<MenuProps["items"]>(() => {
        const groups = new Map<string, SearchRouteItem[]>();
        filteredRoutes.forEach((route) => {
            groups.set(route.groupLabel, [...(groups.get(route.groupLabel) ?? []), route]);
        });
        return Array.from(groups, ([groupLabel, groupRoutes]) => ({
            type: "group" as const,
            label: groupLabel,
            children: groupRoutes.map((route) => ({
                key: route.path,
                id: `${resultsId}-${encodeURIComponent(route.path)}`,
                role: "option",
                icon: route.icon,
                label: `${route.label} · ${route.path}`,
            })),
        }));
    }, [filteredRoutes, resultsId]);

    const activeRoute = filteredRoutes[Math.min(activeIndex, filteredRoutes.length - 1)];
    useEffect(() => {
        if (open && activeRoute) {
            document
                .getElementById(`${resultsId}-${encodeURIComponent(activeRoute.path)}`)
                ?.scrollIntoView({ block: "nearest" });
        }
    }, [open, activeRoute, resultsId]);

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

    useEffect(() => {
        setActiveIndex(0);
    }, [keyword, open]);

    const closeSearch = () => {
        setOpen(false);
        setKeyword("");
    };

    const selectRoute = (route: SearchRouteItem) => {
        closeSearch();
        onSelect(route.path);
    };

    const handleInputKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
        // Enter chooses an IME candidate before it can choose a search result.
        if (event.nativeEvent.isComposing || event.keyCode === 229) return;
        if (event.key === "Escape") {
            closeSearch();
            return;
        }

        if (event.key === "ArrowDown") {
            event.preventDefault();
            setActiveIndex((current) =>
                filteredRoutes.length === 0 ? 0 : (current + 1) % filteredRoutes.length,
            );
            return;
        }

        if (event.key === "ArrowUp") {
            event.preventDefault();
            setActiveIndex((current) =>
                filteredRoutes.length === 0
                    ? 0
                    : (current - 1 + filteredRoutes.length) % filteredRoutes.length,
            );
            return;
        }

        if (event.key === "Enter") {
            event.preventDefault();
            if (activeRoute) {
                selectRoute(activeRoute);
            }
        }
    };

    return (
        <>
            <Button
                color="default"
                variant="filled"
                icon={<SearchOutlined />}
                className="w-9 justify-center sm:w-[180px] sm:justify-start"
                onClick={() => setOpen(true)}
                aria-label={t("打开页面搜索", "Open page search")}
                aria-keyshortcuts="Control+K Meta+K"
            >
                <span className="hidden min-w-0 flex-1 items-center gap-2 sm:flex">
                    <span>{t("搜索页面", "Search pages")}</span>
                    <span className="ml-auto text-xs text-muted-foreground">Ctrl K</span>
                </span>
            </Button>

            <Modal
                open={open}
                onCancel={closeSearch}
                footer={null}
                title={t("搜索页面", "Search pages")}
                width={560}
                destroyOnHidden
            >
                <Flex vertical gap="small">
                    <Input
                        allowClear
                        autoFocus
                        prefix={<SearchOutlined />}
                        value={keyword}
                        placeholder={t("输入页面名称或路径...", "Type a page name or path...")}
                        onChange={(event) => setKeyword(event.target.value)}
                        onKeyDown={handleInputKeyDown}
                        aria-label={t("搜索页面", "Search pages")}
                        role="combobox"
                        aria-autocomplete="list"
                        aria-expanded={open}
                        aria-controls={resultsId}
                        aria-activedescendant={
                            activeRoute
                                ? `${resultsId}-${encodeURIComponent(activeRoute.path)}`
                                : undefined
                        }
                    />

                    {filteredRoutes.length === 0 ? (
                        <div
                            id={resultsId}
                            role="listbox"
                            aria-label={t("页面搜索结果", "Page search results")}
                        >
                            <Empty
                                image={Empty.PRESENTED_IMAGE_SIMPLE}
                                description={t("未找到页面", "No pages found")}
                            />
                        </div>
                    ) : (
                        <div className="max-h-80 overflow-y-auto">
                            <Menu
                                id={resultsId}
                                role="listbox"
                                aria-label={t("页面搜索结果", "Page search results")}
                                items={menuItems}
                                selectedKeys={[activeRoute?.path ?? ""]}
                                onClick={({ key }) => {
                                    const route = filteredRoutes.find((item) => item.path === key);
                                    if (route) {
                                        selectRoute(route);
                                    }
                                }}
                            />
                        </div>
                    )}
                </Flex>
            </Modal>
        </>
    );
};
