import { expect, test } from "bun:test";

import type { ReactElement } from "react";

import { navigationIcon, type NavigationIconKey } from "./navigation-icons";

const destinations: NavigationIconKey[] = [
    "/",
    "/monitoring",
    "/monitoring/overview",
    "/monitoring/nodes",
    "/monitoring/incidents",
    "/monitoring/summaries",
    "/analytics",
    "/analytics/overview",
    "/analytics/details",
    "/reports",
    "/reports/templates",
    "/reports/runs",
    "/system",
    "/system/user",
    "/system/role",
    "/system/menu",
    "/manage/log",
    "/manage",
    "/system/module",
    "/system/status",
    "/system/module-log",
    "/manage/task",
    "/manage/deploy",
];

test("every sidebar destination has an independent Lucide icon", () => {
    const iconTypes = destinations.map(
        (destination) => (navigationIcon(destination) as ReactElement).type,
    );

    expect(new Set(iconTypes).size).toBe(destinations.length);
});
