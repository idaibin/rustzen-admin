import { expect, test } from "bun:test";

import { dedupeModuleNavigation } from "./module-navigation";

const item = (code: string, permission: string): SystemModule.NavigationItem => ({
    module: "reports",
    moduleName: "Reports",
    code,
    title: code === "schedules" ? "定时报表" : "模板",
    path: "/reports/templates",
    icon: "file-text",
    sortOrder: code === "schedules" ? 230 : 210,
    permission,
});

test("keeps the schedule menu for a schedule-only user", () => {
    const result = dedupeModuleNavigation([item("schedules", "reports:schedule:view")]);

    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({ code: "schedules", permission: "reports:schedule:view" });
});

test("dual report permissions produce one stable route using the specific schedule label", () => {
    const result = dedupeModuleNavigation([
        item("templates", "reports:flow:view"),
        item("schedules", "reports:schedule:view"),
    ]);

    expect(result).toHaveLength(1);
    expect(result[0]).toMatchObject({
        code: "schedules",
        title: "定时报表",
        permission: "reports:schedule:view",
    });
});
