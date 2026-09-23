import { afterEach, expect, test } from "bun:test";

import { localizeModuleMenuName } from "./builtin-i18n";
import { setLocale } from "./i18n";

afterEach(() => setLocale("zh-CN"));

test("module menu translations preserve user supplied titles in both languages", () => {
    for (const locale of ["zh-CN", "en-US"] as const) {
        setLocale(locale);
        expect(localizeModuleMenuName("monitor", "overview", "我的主机总览")).toBe("我的主机总览");
        expect(localizeModuleMenuName("monitor", "overview", "My fleet")).toBe("My fleet");
        expect(localizeModuleMenuName("monitor", "overview", "概览")).toBe(
            locale === "zh-CN" ? "概览" : "Overview",
        );
        expect(localizeModuleMenuName("monitor", "overview", "Overview")).toBe(
            locale === "zh-CN" ? "概览" : "Overview",
        );
    }
});

test("reports schedule menu uses the selected locale", () => {
    for (const locale of ["zh-CN", "en-US"] as const) {
        setLocale(locale);
        expect(localizeModuleMenuName("reports", "schedules", "定时报表")).toBe(
            locale === "zh-CN" ? "定时报表" : "Scheduled reports",
        );
    }
});
