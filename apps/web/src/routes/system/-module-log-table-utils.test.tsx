import { expect, test } from "bun:test";

import { getCleanupCandidateColumns } from "./-module-log-table-utils";

test("cleanup candidate columns retain their exact table contract", () => {
    const columns = getCleanupCandidateColumns();
    expect(
        columns.map((column) => [
            column.title,
            column.dataIndex,
            column.key,
            column.width,
            column.ellipsis,
        ]),
    ).toEqual([
        ["模块", "module", "module", 100, undefined],
        ["文件", "fileName", "fileName", undefined, true],
        ["日期", "date", "date", 112, undefined],
        ["大小", "sizeBytes", "sizeBytes", 100, undefined],
    ]);
    expect(columns[3].render?.(1536)).toBe("1.5 KB");
});
