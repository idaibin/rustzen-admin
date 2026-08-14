import { useEffect, useRef, type ReactNode } from "react";

export function DataTableShell({
    ariaLabel,
    children,
}: {
    ariaLabel: string;
    children: ReactNode;
}) {
    const shellRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const scrollRegion = shellRef.current?.querySelector<HTMLElement>(".ant-table-content");
        if (!scrollRegion) {
            return;
        }

        const syncAccessibility = () => {
            const scrollsHorizontally = scrollRegion.scrollWidth > scrollRegion.clientWidth;
            if (scrollsHorizontally) {
                scrollRegion.tabIndex = 0;
                scrollRegion.setAttribute("role", "region");
                scrollRegion.setAttribute("aria-label", ariaLabel);
                return;
            }

            scrollRegion.removeAttribute("tabindex");
            scrollRegion.removeAttribute("role");
            scrollRegion.removeAttribute("aria-label");
        };

        syncAccessibility();
        const observer = new ResizeObserver(syncAccessibility);
        observer.observe(scrollRegion);
        return () => observer.disconnect();
    }, [ariaLabel, children]);

    return (
        <div
            ref={shellRef}
            className="data-table-shell min-h-0 flex-1 overflow-x-auto overflow-y-visible"
        >
            {children}
        </div>
    );
}
