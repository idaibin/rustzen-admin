import { expect, test } from "bun:test";

import { Tag } from "antd";

import { AuthWrap } from "@/components/auth";
import { StatusTag } from "@/components/status-tag";
import { getEnableStatusMeta, getMenuTypeMeta } from "@/constant/options";

import { MenuActions, type DisplayMenuItem } from "./-menu-components";

const record = {
    id: 1,
    parentId: 0,
    name: "Monitor",
    code: "monitor:view",
    menuType: 2,
    sortOrder: 0,
    status: 1,
    isSystem: true,
    isManual: false,
    path: "/monitoring",
    icon: null,
    moduleId: "monitor",
    moduleMenuCode: "monitor-overview",
    isActive: true,
    createdAt: "",
    updatedAt: "",
} as DisplayMenuItem;

test("menu action remains limited to editable module rows", () => {
    expect(MenuActions({ record: { ...record, readOnly: true }, onSuccess: () => {} })).toBeNull();
    expect(MenuActions({ record: { ...record, moduleId: null }, onSuccess: () => {} })).toBeNull();
    expect(
        MenuActions({ record: { ...record, moduleMenuCode: null }, onSuccess: () => {} }),
    ).toBeNull();

    const action = MenuActions({ record, onSuccess: () => {} })!;
    expect(action.type).toBe(AuthWrap);
    expect(action.props.code).toBe("system:menu:update");
});

test("menu badges preserve known and unknown labels and colors", () => {
    const directory = StatusTag({ status: 1, meta: getMenuTypeMeta() });
    const button = StatusTag({ status: 3, meta: getMenuTypeMeta() });
    const unknownType = StatusTag({ status: 99, meta: getMenuTypeMeta() });
    const enabled = StatusTag({ status: 1, meta: getEnableStatusMeta() });
    const disabled = StatusTag({ status: 2, meta: getEnableStatusMeta() });
    const unknownStatus = StatusTag({ status: 99, meta: getEnableStatusMeta() });

    for (const badge of [directory, button, unknownType, enabled, disabled, unknownStatus]) {
        expect(badge.type).toBe(Tag);
    }
    expect([directory.props.color, directory.props.children]).toEqual(["default", "目录"]);
    expect([button.props.color, button.props.children]).toEqual(["green", "按钮"]);
    expect([unknownType.props.color, unknownType.props.children]).toEqual(["default", "未知"]);
    expect([enabled.props.color, enabled.props.children]).toEqual(["success", "启用"]);
    expect([disabled.props.color, disabled.props.children]).toEqual(["default", "禁用"]);
    expect([unknownStatus.props.color, unknownStatus.props.children]).toEqual(["default", "未知"]);
});
