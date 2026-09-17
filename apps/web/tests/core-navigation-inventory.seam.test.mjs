import { describe, expect, test } from "bun:test";

import { getCoreNavigationItems } from "../src/components/layout/routes";

describe("core navigation inventory seam", () => {
    test("includes sidebar Admin routes but excludes non-navigation profile route", () => {
        const items = getCoreNavigationItems();
        const paths = items.map((item) => item.path);

        expect(paths).toEqual(
            expect.arrayContaining([
                "/",
                "/system/user",
                "/system/role",
                "/system/menu",
                "/manage/log",
                "/system/module",
                "/system/status",
                "/system/module-log",
                "/manage/task",
                "/manage/deploy",
            ]),
        );
        expect(paths).not.toContain("/profile");
        expect(items.every((item) => item.permission)).toBe(true);
    });
});
