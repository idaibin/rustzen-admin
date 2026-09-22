import { DataState } from "@/components/feedback/data-state";

export const displayTableProps = {
    pagination: false,
    rowSelection: false,
    tableAlertOptionRender: false,
    toolBarRender: false,
} as const;

export const pagedTableProps = {
    rowSelection: false,
    tableAlertOptionRender: false,
    toolBarRender: false,
} as const;

export function tablePagination({
    current,
    pageSize,
    total,
    onChange,
    hideOnSinglePage,
    showLessItems,
}: {
    current: number;
    pageSize: number;
    total: number;
    onChange: (page: number) => void;
    hideOnSinglePage?: boolean;
    showLessItems?: boolean;
}) {
    return {
        current,
        pageSize,
        total,
        showSizeChanger: false,
        hideOnSinglePage,
        showLessItems,
        onChange,
    };
}

export function emptyTableLocale(
    title: string,
    options: { compact?: boolean; description?: string; visible?: boolean } = {},
) {
    const { compact = false, description, visible = true } = options;
    return {
        emptyText: visible ? (
            <DataState kind="empty" title={title} description={description} compact={compact} />
        ) : undefined,
    };
}
