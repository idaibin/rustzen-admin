import { expect, test } from "bun:test";

import { formatDateTime } from "./format-date-time";
import { setLocale } from "./i18n";

const value = "2026-08-10T12:34:56Z";

test("uses the application locale for stable English and Chinese output", () => {
    const options = {
        timeZone: "Asia/Shanghai",
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
    } as const;
    setLocale("en-US");
    expect(formatDateTime(value, "Asia/Shanghai")).toBe(
        new Intl.DateTimeFormat("en-US", options).format(new Date(value)),
    );
    setLocale("zh-CN");
    expect(formatDateTime(value, "Asia/Shanghai")).toBe(
        new Intl.DateTimeFormat("zh-CN", options).format(new Date(value)),
    );
});

test("uses UTC for the no-timezone verifier path when the process is UTC", () => {
    const previous = process.env.TZ;
    process.env.TZ = "UTC";
    try {
        setLocale("en-US");
        expect(formatDateTime("2026-09-10T01:02:03Z")).toBe("09/10/2026, 01:02:03 AM");
        setLocale("zh-CN");
        expect(formatDateTime("2026-09-10T01:02:03Z")).toBe("2026/09/10 01:02:03");
    } finally {
        process.env.TZ = previous;
    }
});

test("returns a safe placeholder for invalid date or timezone input", () => {
    expect(formatDateTime("not-a-date", "Asia/Shanghai")).toBe("-");
    expect(formatDateTime(value, "Not/A-Timezone")).toBe("-");
});
