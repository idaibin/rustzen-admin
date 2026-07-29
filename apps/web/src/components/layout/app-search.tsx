import { SearchOutlined } from "@ant-design/icons";
import { Button, Empty, Flex, Input, Menu, Modal } from "antd";
import type { MenuProps } from "antd";
import { useEffect, useMemo, useState, type KeyboardEvent } from "react";

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
                icon: route.icon,
                label: route.label,
            })),
        }));
    }, [filteredRoutes]);

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
            const activeRoute = filteredRoutes[activeIndex];
            if (activeRoute) {
                selectRoute(activeRoute);
            }
        }
    };

    return (
        <>
            <Button
                type="text"
                icon={<SearchOutlined />}
                onClick={() => setOpen(true)}
                aria-label={t("打开页面搜索", "Open page search")}
            >
                {t("搜索", "Search")}
            </Button>

            <Modal
                open={open}
                onCancel={closeSearch}
                footer={null}
                title={t("搜索页面", "Search pages")}
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
                    />

                    {filteredRoutes.length === 0 ? (
                        <Empty
                            image={Empty.PRESENTED_IMAGE_SIMPLE}
                            description={t("未找到页面", "No pages found")}
                        />
                    ) : (
                        <div className="max-h-105 overflow-y-auto">
                            <Menu
                                items={menuItems}
                                selectedKeys={[filteredRoutes[activeIndex]?.path ?? ""]}
                                onSelect={({ key }) => {
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
