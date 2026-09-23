import { expect, test } from "bun:test";

import type { ReactElement, ReactNode } from "react";

import { CleanupResult, FailureList } from "./-module-log-cleanup-result";

type Node = ReactElement<Record<string, any>>;
function nodes(value: ReactNode): Node[] {
    if (!value || typeof value !== "object") return [];
    if (Array.isArray(value)) return value.flatMap(nodes);
    const node = value as Node;
    if (!node.props) return [];
    return [node, ...Object.values(node.props).flatMap((prop) => nodes(prop as ReactNode))];
}
function text(value: unknown): string {
    if (typeof value === "string" || typeof value === "number") return String(value);
    if (Array.isArray(value)) return value.map(text).join("");
    if (value && typeof value === "object" && "props" in value)
        return text((value as Node).props.children);
    return "";
}
const failures = [
    { module: "monitor", fileName: "monitor.old", reason: "changed" },
    { module: "reports", fileName: "reports.old", reason: "unreadable" },
];

test("cleanup result renders exact success and partial summaries", () => {
    const success = CleanupResult({
        result: { removed: [], retained: [], failures: [], partial: false } as never,
    });
    expect(success.props.type).toBe("success");
    expect(success.props.message).toBe("清理完成");
    expect(
        nodes(success).some((node) => (node.type as { name?: string }).name === "FailureList"),
    ).toBeFalse();

    const partial = CleanupResult({
        result: { removed: [{}, {}], retained: [{}], failures, partial: true } as never,
    });
    expect(partial.props.type).toBe("warning");
    expect(partial.props.message).toBe("清理完成，但需要复核部分结果");
    expect(text(partial.props.description)).toContain(
        "已删除 2 个文件，保留 1 个文件，失败 2 个文件。",
    );
});

test("failure list keeps every failure text and stable key", () => {
    const tree = FailureList({ failures });
    const items = nodes(tree).filter((node) => node.type === "li");
    expect(items).toHaveLength(2);
    expect(items.map((item) => item.key)).toEqual([
        "monitor:monitor.old:changed",
        "reports:reports.old:unreadable",
    ]);
    expect(items.map(text)).toEqual([
        "monitor / monitor.old: changed",
        "reports / reports.old: unreadable",
    ]);
});
