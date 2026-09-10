import { expect, test } from "bun:test";
import { Tag } from "antd";

import { AuthWrap } from "@/components/auth";
import {
    MenuActions,
    MenuStatusBadge,
    MenuTypeBadge,
    type DisplayMenuItem,
} from "./-menu-components";

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
    expect(MenuActions({ record: { ...record, moduleMenuCode: null }, onSuccess: () => {} })).toBeNull();

    const action = MenuActions({ record, onSuccess: () => {} })!;
    expect(action.type).toBe(AuthWrap);
    expect(action.props.code).toBe("system:menu:update");
});

test("menu badges preserve known and unknown labels and colors", () => {
    const directory = MenuTypeBadge({ menuType: 1 });
    const button = MenuTypeBadge({ menuType: 3 });
    const unknownType = MenuTypeBadge({ menuType: 99 });
    const enabled = MenuStatusBadge({ status: 1 });
    const disabled = MenuStatusBadge({ status: 2 });
    const unknownStatus = MenuStatusBadge({ status: 99 });

    for (const badge of [directory, button, unknownType, enabled, disabled, unknownStatus]) {
        expect(badge.type).toBe(Tag);
    }
    expect([directory.props.color, directory.props.children]).toEqual(["default", "目录"]);
    expect([button.props.color, button.props.children]).toEqual(["green", "按钮"]);
    expect([unknownType.props.color, unknownType.props.children]).toEqual(["default", "未知"]);
    expect([enabled.props.color, enabled.props.children]).toEqual(["green", "启用"]);
    expect([disabled.props.color, disabled.props.children]).toEqual(["default", "禁用"]);
    expect([unknownStatus.props.color, unknownStatus.props.children]).toEqual(["default", "未知"]);
});
