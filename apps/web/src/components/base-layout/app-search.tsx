import { ArrowRightOutlined, CloseOutlined, SearchOutlined } from "@ant-design/icons";
import { Button, Empty, Input, Modal, theme } from "antd";
import type { InputRef } from "antd/es/input";
import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";

import type { AppRoutePath, SearchRouteItem } from "./routes";

interface AppSearchProps {
    routes: SearchRouteItem[];
    onSelect: (path: AppRoutePath) => void;
}

export const AppSearch = ({ routes, onSelect }: AppSearchProps) => {
    const { token } = theme.useToken();
    const inputRef = useRef<InputRef>(null);
    const resultRefs = useRef<Array<HTMLButtonElement | null>>([]);
    const [open, setOpen] = useState(false);
    const [keyword, setKeyword] = useState("");
    const [activeIndex, setActiveIndex] = useState(0);

    const normalizedKeyword = keyword.trim().toLowerCase();
    const filteredRoutes = useMemo(() => {
        if (!normalizedKeyword) {
            return routes;
        }
        return routes.filter((route) => route.searchText.includes(normalizedKeyword));
    }, [normalizedKeyword, routes]);

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
        if (!open) {
            return;
        }

        const timer = window.setTimeout(() => inputRef.current?.focus(), 0);
        return () => window.clearTimeout(timer);
    }, [open]);

    useEffect(() => {
        setActiveIndex(0);
    }, [normalizedKeyword, open]);

    const closeSearch = () => {
        setOpen(false);
        setKeyword("");
    };

    const selectRoute = (route: SearchRouteItem) => {
        closeSearch();
        onSelect(route.path);
    };

    const focusResult = (index: number) => {
        setActiveIndex(index);
        window.setTimeout(() => resultRefs.current[index]?.focus(), 0);
    };

    const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
        if (event.key === "Tab") {
            event.preventDefault();
            if (filteredRoutes.length > 0) {
                focusResult(event.shiftKey ? filteredRoutes.length - 1 : activeIndex);
            }
            return;
        }

        if (event.key === "ArrowDown") {
            event.preventDefault();
            setActiveIndex((current) => {
                if (filteredRoutes.length === 0) {
                    return 0;
                }
                return Math.min(current + 1, filteredRoutes.length - 1);
            });
            return;
        }

        if (event.key === "ArrowUp") {
            event.preventDefault();
            setActiveIndex((current) => Math.max(current - 1, 0));
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

    const handleResultKeyDown = (event: KeyboardEvent<HTMLButtonElement>, index: number) => {
        if (event.key === "Tab") {
            event.preventDefault();
            if (filteredRoutes.length === 0) {
                return;
            }
            const nextIndex = event.shiftKey
                ? (index - 1 + filteredRoutes.length) % filteredRoutes.length
                : (index + 1) % filteredRoutes.length;
            focusResult(nextIndex);
            return;
        }

        if (event.key === "ArrowDown") {
            event.preventDefault();
            focusResult((index + 1) % filteredRoutes.length);
            return;
        }

        if (event.key === "ArrowUp") {
            event.preventDefault();
            focusResult((index - 1 + filteredRoutes.length) % filteredRoutes.length);
            return;
        }

        if (event.key === "Enter") {
            event.preventDefault();
            selectRoute(filteredRoutes[index]);
        }
    };

    return (
        <>
            <button
                type="button"
                className="flex h-8 w-44 items-center gap-2 rounded-md border border-solid px-2.5 text-left text-sm shadow-none transition"
                style={{
                    background: token.colorFillQuaternary,
                    borderColor: "transparent",
                    color: token.colorTextTertiary,
                }}
                onClick={() => setOpen(true)}
                aria-label="Open page search"
            >
                <SearchOutlined />
                <span className="min-w-0 flex-1 truncate">Search</span>
                <kbd
                    className="rounded border border-solid px-1.5 py-0.5 text-xs leading-none"
                    style={{
                        borderColor: "transparent",
                        background: token.colorFillSecondary,
                        color: token.colorTextTertiary,
                    }}
                >
                    ⌘ K
                </kbd>
            </button>

            <Modal
                open={open}
                onCancel={closeSearch}
                footer={null}
                closable={false}
                mask={{ closable: true }}
                width={680}
                styles={{ body: { padding: 0 } }}
            >
                <div
                    className="flex items-center gap-3 border-0 border-b border-solid px-4 py-3"
                    style={{ borderColor: token.colorBorderSecondary }}
                >
                    <SearchOutlined style={{ color: token.colorTextTertiary }} />
                    <Input
                        ref={inputRef}
                        tabIndex={-1}
                        variant="borderless"
                        value={keyword}
                        placeholder="Type a page name or path..."
                        onChange={(event) => setKeyword(event.target.value)}
                        onKeyDown={handleKeyDown}
                        aria-label="Search pages"
                    />
                    <Button
                        type="text"
                        icon={<CloseOutlined />}
                        aria-label="Close search"
                        tabIndex={-1}
                        onClick={closeSearch}
                    />
                </div>

                <div className="max-h-[420px] overflow-y-auto p-3">
                    {filteredRoutes.length === 0 ? (
                        <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No pages found" />
                    ) : (
                        <div role="listbox" aria-label="Search results">
                            {filteredRoutes.map((route, index) => {
                                const isActive = index === activeIndex;
                                const showGroup =
                                    index === 0 ||
                                    route.groupLabel !== filteredRoutes[index - 1]?.groupLabel;

                                return (
                                    <div key={route.path}>
                                        {showGroup ? (
                                            <div
                                                className="px-3 pb-1 pt-2 text-xs font-medium"
                                                style={{ color: token.colorTextTertiary }}
                                            >
                                                {route.groupLabel}
                                            </div>
                                        ) : null}
                                        <button
                                            type="button"
                                            role="option"
                                            aria-selected={isActive}
                                            tabIndex={isActive ? 0 : -1}
                                            ref={(element) => {
                                                resultRefs.current[index] = element;
                                            }}
                                            className="flex w-full items-center gap-3 rounded-md border-0 px-3 py-2 text-left transition"
                                            style={{
                                                background: isActive
                                                    ? token.colorFillSecondary
                                                    : "transparent",
                                                color: token.colorText,
                                            }}
                                            onMouseEnter={() => setActiveIndex(index)}
                                            onKeyDown={(event) => handleResultKeyDown(event, index)}
                                            onClick={() => selectRoute(route)}
                                        >
                                            <span
                                                className="flex size-6 items-center justify-center"
                                                style={{ color: token.colorTextSecondary }}
                                            >
                                                {route.icon}
                                            </span>
                                            <span className="min-w-0 flex-1">
                                                <span className="block truncate">
                                                    {route.label}
                                                </span>
                                                <span
                                                    className="block truncate text-xs"
                                                    style={{ color: token.colorTextTertiary }}
                                                >
                                                    {route.path}
                                                </span>
                                            </span>
                                            <ArrowRightOutlined
                                                style={{ color: token.colorTextTertiary }}
                                            />
                                        </button>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            </Modal>
        </>
    );
};
