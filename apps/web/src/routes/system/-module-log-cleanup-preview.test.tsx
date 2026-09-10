import { expect, test } from "bun:test";
import { Table } from "antd";
import type { ReactElement, ReactNode } from "react";

import type {
    ModuleLogCleanupPreview as ModuleLogCleanupPreviewData,
    ModuleLogCleanupResult,
} from "@/api/system/status/module-logs";
import { ModuleLogCleanupPreview } from "./-module-log-cleanup-preview";
import { ModuleLogCleanupPreviewSection } from "./-module-log-diagnostics";
import { formatDateTime } from "./-module-log-table-utils";

type Node = ReactElement<Record<string, any>>;

const preview: ModuleLogCleanupPreviewData = {
    cutoffDate: "2026-09-01",
    expiresAt: "2026-09-11T00:00:00.000Z",
    previewId: "preview-1",
    token: "token-1",
    candidates: [
        {
            module: "monitor",
            fileName: "monitor.2026-08-31",
            date: "2026-08-31",
            modifiedAt: "2026-08-31T00:00:00.000Z",
            sizeBytes: 1536,
        },
    ],
    failures: [{ module: "reports", fileName: "reports.2026-08-31", reason: "unreadable" }],
};
const result: ModuleLogCleanupResult = {
    previewId: "preview-1",
    removed: [],
    retained: [],
    failures: [],
    partial: false,
};

function nodes(value: ReactNode): Node[] {
    if (!value || typeof value !== "object") return [];
    if (Array.isArray(value)) return value.flatMap(nodes);
    const node = value as Node;
    if (!node.props) return [];
    return [node, ...Object.values(node.props).flatMap((prop) => nodes(prop as ReactNode))];
}

function values(value: ReactNode): unknown[] {
    if (Array.isArray(value)) return value.flatMap(values);
    if (value && typeof value === "object" && "props" in value) {
        return Object.values((value as Node).props).flatMap((prop) => values(prop as ReactNode));
    }
    return [value];
}

function renderPreview(props: Partial<Parameters<typeof ModuleLogCleanupPreview>[0]> = {}) {
    return ModuleLogCleanupPreview({
        confirmAction: null,
        error: null,
        isExpired: false,
        preview: null,
        result: null,
        ...props,
    });
}

function renderSection(props: Partial<Parameters<typeof ModuleLogCleanupPreviewSection>[0]> = {}) {
    return ModuleLogCleanupPreviewSection({
        confirmAction: <span data-testid="cleanup-confirm">confirm</span>,
        error: null,
        isExpired: false,
        isPending: false,
        preview: null,
        result: null,
        ...props,
    });
}

function sectionDisplay(props: Partial<Parameters<typeof ModuleLogCleanupPreviewSection>[0]> = {}) {
    const section = renderSection(props)!;
    const previewNode = nodes(section).find(
        (node) => (node.type as { name?: string }).name === "ModuleLogCleanupPreview",
    )!;
    const display = (previewNode.type as typeof ModuleLogCleanupPreview)(
        previewNode.props as Parameters<typeof ModuleLogCleanupPreview>[0],
    );
    return { display, previewNode, section };
}

test("cleanup preview preserves error, preview, result, and empty branches", () => {
    const error = renderPreview({ error: "failed", preview, result })!;
    expect(error.props.type).toBe("error");
    expect(error.props.message).toBe("清理预览失败");
    expect(error.props.action).toBeUndefined();

    const info = renderPreview({ preview })!;
    const infoAlert = nodes(info).find((node) => node.props.type === "info")!;
    expect(infoAlert.props.message).toBe(`预览有效至 ${formatDateTime(preview.expiresAt)}`);
    expect(infoAlert.props.description).toBe(
        "仅处理 2026-09-01 之前的固定模块日志；当前文件不会删除。",
    );

    const expired = renderPreview({ isExpired: true, preview })!;
    const warning = nodes(expired).find((node) => node.props.type === "warning")!;
    expect(warning.props.message).toBe("预览已过期，请重新生成。");

    const fallback = renderPreview({ result })!;
    expect((fallback.type as { name?: string }).name).toBe("CleanupResult");
    expect(renderPreview()).toBeNull();
});

test("cleanup preview preserves candidate table and confirmation slot order", () => {
    const tree = renderPreview({
        confirmAction: <span data-testid="cleanup-confirm">confirm</span>,
        preview: { ...preview, failures: [] },
    })!;
    const table = nodes(tree).find((node) => node.type === Table)!;
    expect(table.props["data-testid"]).toBe("module-log-cleanup-candidates");
    expect(table.props.dataSource).toEqual(preview.candidates);
    expect(table.props.columns.map((column: { key: string }) => column.key)).toEqual([
        "module",
        "fileName",
        "date",
        "sizeBytes",
    ]);
    expect(table.props.rowKey(preview.candidates[0])).toBe("monitor:2026-08-31");
    expect(table.props.size).toBe("small");
    expect(table.props.pagination).toBeFalse();
    expect(table.props.scroll).toEqual({ x: 520 });

    const wrapper = nodes(tree).find((node) => node.props.className === "space-y-3")!;
    const children = wrapper.props.children as ReactNode[];
    const tableIndex = children.findIndex((child) => nodes(child).some((node) => node.type === Table));
    const confirmIndex = children.findIndex(
        (child) => nodes(child).some((node) => node.props["data-testid"] === "cleanup-confirm"),
    );
    expect(confirmIndex).toBeGreaterThan(tableIndex);
});

test("cleanup preview keeps empty candidates free of confirmation and zero text", () => {
    const tree = renderPreview({
        confirmAction: <span data-testid="cleanup-confirm">confirm</span>,
        preview: { ...preview, candidates: [], failures: [] },
    })!;
    const empty = nodes(tree).find((node) => node.props.kind === "empty")!;
    expect(empty.props.title).toBe("没有符合条件的日志");
    expect(nodes(tree).some((node) => node.props["data-testid"] === "cleanup-confirm")).toBeFalse();
    expect(values(tree)).not.toContain(0);
});

test("parent composition keeps processing independent of preview, result, and error", () => {
    const pendingPreviewSection = sectionDisplay({ isPending: true, preview });
    const pendingPreview = [...nodes(pendingPreviewSection.section), ...nodes(pendingPreviewSection.display)];
    expect(pendingPreview.some((node) => node.props.kind === "processing")).toBeTrue();
    expect(pendingPreview.some((node) => node.type === Table)).toBeTrue();
    expect(pendingPreview.some((node) => node.props["data-testid"] === "cleanup-confirm")).toBeTrue();

    const pendingResultSection = sectionDisplay({ isPending: true, result });
    const pendingResult = [...nodes(pendingResultSection.section), ...nodes(pendingResultSection.display)];
    expect(pendingResult.some((node) => node.props.kind === "processing")).toBeTrue();
    expect(pendingResult.some((node) => (node.type as { name?: string }).name === "CleanupResult")).toBeTrue();

    const pendingErrorSection = sectionDisplay({ error: "failed", isPending: true, preview });
    const pendingError = [...nodes(pendingErrorSection.section), ...nodes(pendingErrorSection.display)];
    expect(pendingError.some((node) => node.props.kind === "processing")).toBeTrue();
    expect(pendingErrorSection.previewNode.props.error).toBe("failed");
    expect(pendingErrorSection.previewNode.props.confirmAction).toBeNull();
    expect(pendingError.some((node) => node.props["data-testid"] === "cleanup-confirm")).toBeFalse();
});
