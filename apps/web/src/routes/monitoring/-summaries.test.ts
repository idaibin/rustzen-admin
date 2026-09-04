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

test("zero-sample daily summaries render resource ranges as an empty state", () => {
    const zeroSampleRange: Monitor.SummaryRange = { min: null, avg: null, max: null };

    expect(formatSummaryRange(zeroSampleRange, 0)).toBe("—");
    expect(formatSummaryRange({ min: 12, avg: null, max: 30 }, 2)).toBe("—");
});
