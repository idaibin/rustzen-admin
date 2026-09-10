import { expect, test } from "bun:test";

import { formatBytes, formatDateTime, getCleanupCandidateColumns } from "./-module-log-table-utils";

test("cleanup candidate columns retain their exact table contract", () => {
    const columns = getCleanupCandidateColumns();
    expect(columns.map((column) => [column.title, column.dataIndex, column.key, column.width, column.ellipsis])).toEqual([
        ["模块", "module", "module", 100, undefined],
        ["文件", "fileName", "fileName", undefined, true],
        ["日期", "date", "date", 112, undefined],
        ["大小", "sizeBytes", "sizeBytes", 100, undefined],
    ]);
    expect(columns[3].render?.(1536)).toBe("1.5 KB");
});

test("module log table formatters preserve byte boundaries and invalid dates", () => {
    expect([0, 1, 1023, 1024, 1536, 1024 ** 2, 1024 ** 4]).toEqual([0, 1, 1023, 1024, 1536, 1048576, 1099511627776]);
    expect([0, 1, 1023, 1024, 1536, 1024 ** 2, 1024 ** 4].map(formatBytes)).toEqual(["0 B", "1 B", "1023 B", "1 KB", "1.5 KB", "1 MB", "1 TB"]);
    expect(formatDateTime("invalid")).toBe("-");
    expect(formatDateTime("2026-09-11T00:00:00.000Z")).not.toBe("-");
});
