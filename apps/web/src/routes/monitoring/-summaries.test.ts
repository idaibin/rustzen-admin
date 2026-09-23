import { expect, test } from "bun:test";

// The Bun suite shares one process. Some earlier tests intentionally install a
// minimal document stub; SWR (loaded through ProTable) needs the listener
// methods during module initialization. Keep the real component module import,
// but complete only the missing browser seam for this test.
const sharedDocument = globalThis.document;
if (sharedDocument && typeof sharedDocument.addEventListener !== "function") {
    Object.assign(sharedDocument, {
        addEventListener: () => undefined,
        removeEventListener: () => undefined,
    });
}

const { formatSummaryRange } = await import("./summaries");
const source = await Bun.file("src/routes/monitoring/summaries.tsx").text();

test("zero-sample daily summaries render resource ranges as an empty state", () => {
    const zeroSampleRange: Monitor.SummaryRange = { min: null, avg: null, max: null };

    expect(formatSummaryRange(zeroSampleRange, 0)).toBe("—");
    expect(formatSummaryRange({ min: 12, avg: null, max: 30 }, 2)).toBe("—");
});

test("daily summaries retain the fixed page contract while refreshing in the background", () => {
    expect(source).toContain("refetchInterval: 30_000");
    expect(source).toContain("<BackgroundRefreshNotice");
    expect(source).toContain("<Pagination");
    expect(source).toContain("{...displayTableProps}");
    expect(source).toContain('scroll={{ y: "100%" }}');
    expect(source).not.toContain("nodeId search");
});

test("narrow summaries preserve date, node, and coverage without squeezing detail columns", () => {
    expect(source).toContain('{ title: t("日期", "Date"), dataIndex: "date", width: 104 }');
    expect(source).toContain('{ title: t("节点", "Node"), dataIndex: "nodeId", ellipsis: true }');
    expect(source.match(/responsive: \["sm"\]/g)?.length).toBe(6);
});
