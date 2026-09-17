import { describe, expect, test } from "bun:test";

const routePaths = [
    "../src/routes/index.tsx",
    "../src/routes/login.tsx",
    "../src/routes/manage/log.tsx",
    "../src/routes/manage/task.tsx",
    "../src/routes/system/menu.tsx",
    "../src/routes/system/role.tsx",
    "../src/routes/system/status.tsx",
    "../src/routes/system/module-log.tsx",
    "../src/routes/system/user.tsx",
];
const routes = await Promise.all(
    routePaths.map((path) => Bun.file(new URL(path, import.meta.url)).text()),
);

describe("console locale consistency", () => {
    test("every retained console route subscribes to the locale used by its bilingual copy", () => {
        for (const [index, source] of routes.entries()) {
            expect(source, routePaths[index]).toContain("useLocale()");
        }
    });

    test("dashboard keeps semantic module state text and the login page has one top-level heading", () => {
        expect(routes[0]).toContain('t("运行中", "Online")');
        expect(routes[0]).toContain('t("不可用", "Unavailable")');
        expect(routes[1].match(/level=\{1\}/g)).toHaveLength(1);
    });

    test("dashboard module health stays readable inside the desktop side column", () => {
        expect(routes[0]).toContain("md:grid-cols-3 xl:grid-cols-1");
    });
});
