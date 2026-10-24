import { afterEach, beforeEach, expect, test } from "bun:test";

import { renderToStaticMarkup } from "react-dom/server";

import { getLocale, setLocale } from "@/lib/i18n";

import { PermissionPicker } from "./-role-permission-picker";

let previousLocale = getLocale();

beforeEach(() => {
    previousLocale = getLocale();
});

afterEach(() => setLocale(previousLocale));

const commonProps = {
    permissions: [{ value: 17, title: "User access", code: "system:user:list" }],
    checkedValues: ["17"],
    loading: false,
    error: null,
    permissionReady: true,
    permissionRefreshing: false,
    permissionSearch: "user",
    isAllChecked: true,
    onCheck: () => {},
    onSearchChange: () => {},
    onSelectAllChange: () => {},
    onRetry: () => {},
};

test("background refresh failure keeps cached permission choices and selection visible with retry", () => {
    setLocale("en-US");
    const html = renderToStaticMarkup(
        <PermissionPicker
            {...commonProps}
            refreshError
            updatedAt={new Date("2026-07-27T10:11:12Z").getTime()}
        />,
    );

    expect(html).toContain("Background refresh failed.");
    expect(html).toContain("Retry");
    expect(html).toContain("1 selected");
    expect(html).toContain('value="user"');
    expect(html).toContain("User access");
    expect(html).toContain("system:user:list");
    expect(html).toContain('aria-checked="true"');
    expect(html).toContain('role="alert"');
});

test("initial permission failure and successful empty data remain distinct", () => {
    setLocale("en-US");
    const initialFailure = renderToStaticMarkup(
        <PermissionPicker
            {...commonProps}
            permissions={[]}
            checkedValues={[]}
            error={new Error("initial request failed")}
            permissionReady={false}
            refreshError={false}
            updatedAt={0}
        />,
    );
    const successfulEmpty = renderToStaticMarkup(
        <PermissionPicker
            {...commonProps}
            permissions={[]}
            checkedValues={[]}
            permissionReady={false}
            refreshError={false}
            updatedAt={0}
        />,
    );

    expect(initialFailure).toContain("Failed to load permissions");
    expect(initialFailure).toContain("Reload");
    expect(initialFailure).not.toContain("No permissions available to assign");
    expect(successfulEmpty).toContain("No permissions available to assign");
    expect(successfulEmpty).not.toContain("Failed to load permissions");
});
