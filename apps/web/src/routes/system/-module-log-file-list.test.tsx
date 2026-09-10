import { expect, test } from "bun:test";
import { ProTable } from "@ant-design/pro-components";
import { Button } from "antd";
import type { ReactElement, ReactNode } from "react";

import { DataTableShell } from "@/components/table/data-table-shell";
import type { ModuleLogFile } from "@/api/system/status/module-logs";
import { ModuleLogFileList } from "./-module-log-file-list";
import { getModuleLogColumns } from "./-module-log-columns";

type Node = ReactElement<Record<string, any>>;

const files: ModuleLogFile[] = [
    {
        module: "monitor",
        date: "2026-09-11",
        fileName: "monitor.2026-09-11",
        sizeBytes: 1536,
        modifiedAt: "2026-09-11T00:00:00.000Z",
        readable: true,
        active: true,
    },
];
const unreadable = { ...files[0], module: "reports", readable: false };
const columns = getModuleLogColumns(() => {});

function nodes(value: ReactNode): Node[] {
    if (!value || typeof value !== "object") return [];
    if (Array.isArray(value)) return value.flatMap(nodes);
    const node = value as Node;
    if (!node.props) return [];
    return [node, ...Object.values(node.props).flatMap((prop) => nodes(prop as ReactNode))];
}

function render(props: Partial<Parameters<typeof ModuleLogFileList>[0]> = {}) {
    return ModuleLogFileList({
        columns,
        error: null,
        files: [],
        isFetching: false,
        isPending: false,
        onReload: () => {},
        onSelectionChange: () => {},
        selectedKeys: [],
        ...props,
    });
}

test("module log file list keeps stale rows and their refresh warning", () => {
    const tree = render({ error: "network failed", files });
    const warning = nodes(tree).find((node) => node.props.type === "warning")!;
    expect(warning.props.message).toBe("日志列表刷新失败，仍显示上次结果");
    expect(warning.props.description).toBe("network failed");
    expect(nodes(tree).some((node) => node.type === ProTable)).toBeTrue();

    const emptyError = render({ error: "", files });
    const emptyWarning = nodes(emptyError).find((node) => node.props.type === "warning")!;
    expect(emptyWarning.props.description).toBe("");
    expect(nodes(emptyError).some((node) => node.type === ProTable)).toBeTrue();
});

test("module log file list keeps loading, error reload, and empty states distinct", () => {
    const loading = nodes(render({ isPending: true })).find((node) => node.props.kind === "loading")!;
    expect(loading.props.title).toBe("正在加载模块日志");
    const pendingError = nodes(render({ error: "failed", isPending: true }));
    expect(pendingError.some((node) => node.props.kind === "loading")).toBeTrue();
    expect(pendingError.some((node) => node.props.kind === "error")).toBeFalse();

    let reloaded = false;
    const error = nodes(render({ error: "failed", onReload: () => { reloaded = true; } })).find(
        (node) => node.props.kind === "error",
    )!;
    expect(error.props.title).toBe("模块日志加载失败");
    expect(error.props.description).toBe("failed");
    const reload = error.props.action as ReactElement<{
        children: string;
        onClick: () => void;
        type: string;
    }>;
    expect(reload.type).toBe(Button);
    expect(reload.props.type).toBe("primary");
    expect(reload.props.children).toBe("重新加载");
    reload.props.onClick();
    expect(reloaded).toBeTrue();

    const emptyError = nodes(render({ error: "" })).find((node) => node.props.kind === "error")!;
    expect(emptyError.props.description).toBe("");

    const empty = nodes(render()).find((node) => node.props.kind === "empty")!;
    expect(empty.props.title).toBe("暂无模块日志文件");
    expect(empty.props.description).toBe(
        "仅展示 admin、monitor、insights、reports 四个固定模块的日志文件。",
    );
});

test("module log file list preserves table metadata and selection callbacks", () => {
    let selectedKeys: React.Key[] = [];
    let selectedRows: ModuleLogFile[] = [];
    const tree = render({
        files,
        isFetching: true,
        onSelectionChange: (keys, rows) => {
            selectedKeys = keys;
            selectedRows = rows;
        },
        selectedKeys: ["monitor:2026-09-11"],
    });
    const shell = nodes(tree).find((node) => node.type === DataTableShell)!;
    expect(shell.props.ariaLabel).toBe("模块日志文件");
    const table = nodes(tree).find((node) => node.type === ProTable)!;
    expect(table.props.rowKey(files[0])).toBe("monitor:2026-09-11");
    expect(table.props.columns).toBe(columns);
    expect(table.props.dataSource).toBe(files);
    expect(table.props.loading).toBeTrue();
    expect(table.props.search).toBeFalse();
    expect(table.props.options).toBeFalse();
    expect(table.props.pagination).toBeFalse();
    expect(table.props.toolBarRender).toBeFalse();
    expect(table.props.tableAlertOptionRender).toBeFalse();
    expect(table.props.rowSelection.selectedRowKeys).toEqual(["monitor:2026-09-11"]);
    table.props.rowSelection.onChange(["monitor:2026-09-11"], files);
    expect(selectedKeys).toEqual(["monitor:2026-09-11"]);
    expect(selectedRows).toEqual(files);
    expect(table.props.rowSelection.getCheckboxProps(files[0])).toEqual({ disabled: false });
    expect(table.props.rowSelection.getCheckboxProps(unreadable)).toEqual({ disabled: true });

    const cell = table.props.rowSelection.renderCell(false, files[0], 0, <span>checkbox</span>);
    expect(cell.props["data-testid"]).toBe("module-log-select-monitor-2026-09-11");
    expect(cell.props.children.props.children).toBe("checkbox");
    expect(table.props.locale.emptyText.props.title).toBe("暂无日志");
});
