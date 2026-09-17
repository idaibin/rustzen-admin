import { expect, test } from "bun:test";

import { EyeOutlined } from "@ant-design/icons";

import { getModuleLogColumns } from "./-module-log-columns";

const readable = {
    module: "admin",
    date: "2026-09-11",
    fileName: "admin.2026-09-11",
    sizeBytes: 1536,
    modifiedAt: "invalid",
    readable: true,
    active: true,
};
const unreadable = { ...readable, module: "monitor", readable: false, active: false };
const render = (column: any, record: any) => column.render(null, record, 0);

test("module log columns retain metadata, status, and view-action contracts", () => {
    let opened: unknown;
    const columns = getModuleLogColumns((record) => {
        opened = record;
    });
    expect(
        columns.map((column) => [
            column.title,
            column.dataIndex,
            column.key,
            column.width,
            column.fixed,
            column.ellipsis,
        ]),
    ).toEqual([
        ["模块", "module", "module", 112, undefined, undefined],
        ["文件 / 日期", undefined, "file", undefined, undefined, true],
        ["大小", undefined, "size", 110, undefined, undefined],
        ["状态", undefined, "status", 150, undefined, undefined],
        ["修改时间", undefined, "modifiedAt", 180, undefined, undefined],
        ["操作", undefined, "actions", 64, "right", undefined],
    ]);
    const module: any = render(columns[0], readable);
    expect([module.props.code, module.props.children]).toEqual([true, "admin"]);
    const file: any = render(columns[1], readable);
    expect([file.props.direction, file.props.size]).toEqual(["vertical", 0]);
    expect(file.props.children[0].props.ellipsis).toEqual({ tooltip: readable.fileName });
    expect(file.props.children[0].props.children).toBe(readable.fileName);
    expect([file.props.children[1].props.type, file.props.children[1].props.children]).toEqual([
        "secondary",
        readable.date,
    ]);
    expect(render(columns[2], readable)).toBe("1.5 KB");
    expect(render(columns[4], readable)).toBe("-");
    const active: any = render(columns[3], readable);
    expect([active.props.children[0].props.color, active.props.children[0].props.children]).toEqual(
        ["green", "可读"],
    );
    expect([active.props.children[1].props.color, active.props.children[1].props.children]).toEqual(
        ["orange", "当前文件"],
    );
    const inactive: any = render(columns[3], unreadable);
    expect([
        inactive.props.children[0].props.color,
        inactive.props.children[0].props.children,
    ]).toEqual(["red", "不可读"]);
    expect(inactive.props.children[1]).toBeNull();
    const readAction: any = render(columns[5], readable);
    const blockedAction: any = render(columns[5], unreadable);
    for (const [action, record, disabled] of [
        [readAction, readable, false],
        [blockedAction, unreadable, true],
    ] as const) {
        expect([
            action.props["data-testid"],
            action.props.type,
            action.props.size,
            action.props.disabled,
            action.props.children,
        ]).toEqual([
            `module-log-tail-${record.module}-${record.date}`,
            "link",
            "small",
            disabled,
            "查看",
        ]);
        expect(action.props.icon.type).toBe(EyeOutlined);
    }
    readAction.props.onClick();
    expect(opened).toBe(readable);
});
