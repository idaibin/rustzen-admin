import { useEffect, useRef, type ReactNode } from "react";

export function DataTableShell({
    ariaLabel,
    children,
    fill = false,
    testId,
}: {
    ariaLabel: string;
    children: ReactNode;
    fill?: boolean;
    testId?: string;
}) {
    const shellRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const scrollRegion = shellRef.current?.querySelector<HTMLElement>(
            ".ant-table-body, .ant-table-content",
        );
        if (!scrollRegion) {
            return;
        }

        const syncAccessibility = () => {
            shellRef.current?.style.setProperty(
                "--table-viewport-width",
                `${scrollRegion.clientWidth}px`,
            );
            const scrolls =
                scrollRegion.scrollWidth > scrollRegion.clientWidth ||
                scrollRegion.scrollHeight > scrollRegion.clientHeight;
            if (scrolls) {
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
            data-testid={testId}
            className={
                fill
                    ? "data-table-shell data-table-shell-fill flex min-h-0 flex-1 flex-col overflow-hidden"
                    : "data-table-shell shrink-0 overflow-hidden"
            }
        >
            {children}
        </div>
    );
}
