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

    test("the login page has one top-level heading", () => {
        expect(routes[1].match(/level=\{1\}/g)).toHaveLength(1);
    });
});
