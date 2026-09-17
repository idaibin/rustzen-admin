import { expect, test } from "bun:test";

import { DeleteOutlined, DownloadOutlined, ReloadOutlined } from "@ant-design/icons";
import { Button, DatePicker, Select, Space } from "antd";
import dayjs from "dayjs";
import type { ReactElement, ReactNode } from "react";

import { ModuleLogActions, ModuleLogToolbar } from "./-module-log-controls";

type Node = ReactElement<Record<string, any>>;

function nodes(value: ReactNode): Node[] {
    if (!value || typeof value !== "object") return [];
    if (Array.isArray(value)) return value.flatMap(nodes);
    const node = value as Node;
    if (!node.props) return [];
    return [node, ...Object.values(node.props).flatMap((prop) => nodes(prop as ReactNode))];
}

function actions(props: Partial<Parameters<typeof ModuleLogActions>[0]> = {}) {
    return ModuleLogActions({
        backupPending: false,
        onBackup: () => {},
        onPreview: () => {},
        previewPending: false,
        selectedCount: 1,
        ...props,
    });
}

function actionButtons(props: Partial<Parameters<typeof ModuleLogActions>[0]> = {}) {
    return nodes(actions(props)).filter((node) => node.type === Button);
}

function toolbar(props: Partial<Parameters<typeof ModuleLogToolbar>[0]> = {}) {
    return ModuleLogToolbar({
        dateFilter: "",
        isFetching: false,
        moduleFilter: "all",
        onDateChange: () => {},
        onModuleChange: () => {},
        onRefresh: () => {},
        ...props,
    });
}

test("module log actions preserve backup disabled, pending, and callback behavior", () => {
    let backedUp = false;
    const actionTree = actions({
        onBackup: () => {
            backedUp = true;
        },
    });
    expect(actionTree.type).toBe(Space);
    expect(actionTree.props.wrap).toBeTrue();
    const enabled = nodes(actionTree).filter((node) => node.type === Button);
    const backup = enabled.find((button) => button.props["data-testid"] === "module-log-backup")!;
    expect(backup.props.disabled).toBeFalse();
    expect(backup.props.loading).toBeFalse();
    expect((backup.props.icon as Node).type).toBe(DownloadOutlined);
    expect(backup.props.children).toBe("备份选中文件");
    backup.props.onClick();
    expect(backedUp).toBeTrue();

    const disabled = actionButtons({ selectedCount: 0 }).find(
        (button) => button.props["data-testid"] === "module-log-backup",
    )!;
    expect(disabled.props.disabled).toBeTrue();

    const pending = actionButtons({ backupPending: true }).find(
        (button) => button.props["data-testid"] === "module-log-backup",
    )!;
    expect(pending.props.disabled).toBeTrue();
    expect(pending.props.loading).toBeTrue();
});

test("module log actions preserve preview loading, icon, and callback", () => {
    let previewed = false;
    const idle = actionButtons({
        onPreview: () => {
            previewed = true;
        },
    }).find((button) => button.props["data-testid"] === "module-log-cleanup-preview")!;
    expect(idle.props.loading).toBeFalse();
    expect(idle.props.disabled).toBeUndefined();
    expect((idle.props.icon as Node).type).toBe(DeleteOutlined);
    expect(idle.props.children).toBe("预览清理");
    idle.props.onClick();
    expect(previewed).toBeTrue();

    const pending = actionButtons({ previewPending: true }).find(
        (button) => button.props["data-testid"] === "module-log-cleanup-preview",
    )!;
    expect(pending.props.loading).toBeTrue();
    expect(pending.props.disabled).toBeUndefined();
});

test("module log toolbar preserves module options and filter changes", () => {
    let module = "";
    let date = "";
    const tree = toolbar({
        dateFilter: "2026-09-11",
        moduleFilter: "monitor",
        onDateChange: (value) => {
            date = value;
        },
        onModuleChange: (value) => {
            module = value;
        },
    });
    expect(tree.props.className).toBe("flex flex-wrap items-center gap-3");
    const select = nodes(tree).find((node) => node.type === Select)!;
    expect(select.props["aria-label"]).toBe("模块筛选");
    expect(select.props.className).toBe("w-36");
    expect(select.props.value).toBe("monitor");
    expect(select.props.options).toEqual([
        { label: "全部模块", value: "all" },
        { label: "admin", value: "admin" },
        { label: "monitor", value: "monitor" },
        { label: "insights", value: "insights" },
        { label: "reports", value: "reports" },
    ]);
    select.props.onChange("reports");
    expect(module).toBe("reports");

    const picker = nodes(tree).find((node) => node.type === DatePicker)!;
    expect(picker.props["aria-label"]).toBe("日志日期");
    expect(picker.props.value.format("YYYY-MM-DD")).toBe("2026-09-11");
    expect(picker.props.style).toEqual({ width: 160, maxWidth: "100%" });
    picker.props.onChange(dayjs("2026-09-12"));
    expect(date).toBe("2026-09-12");
    picker.props.onChange(null);
    expect(date).toBe("");
});

test("module log toolbar preserves refresh loading, icon, and callback", () => {
    let refreshed = false;
    const refresh = nodes(
        toolbar({
            isFetching: true,
            onRefresh: () => {
                refreshed = true;
            },
        }),
    ).find((node) => node.type === Button)!;
    expect(refresh.props.loading).toBeTrue();
    expect((refresh.props.icon as Node).type).toBe(ReloadOutlined);
    expect(refresh.props.children).toBe("刷新");
    refresh.props.onClick();
    expect(refreshed).toBeTrue();
});
